// Copyright (C) 2024 Alvaro Gaiotti
// See end of file for license information

//! A partially played state: the input to double-dummy solving.
//!
//! A [`Position`] is a *snapshot*, not a mutable cursor. It is a pure function
//! of a `Deal`, a [`PlayRecord`] prefix and a [`Strain`], so "rewinding" is
//! prefix derivation ([`Position::after_play`]) rather than an undo stack.
//! Keeping it a value type is what lets it be serialized, compared and handed
//! to the solver unchanged.
//!
//! Deriving a position always needs the strain to fill [`Position::trump`], and
//! a non-empty prefix needs it again to name the winner of a completed trick.
//! Only the opening position needs an opening leader, which the position's own
//! fields cannot name; [`Position::from_deal_prefix`] therefore takes it as an
//! explicit input and [`Position::from_deal_prefix_with_contract`] reads it
//! from a [`Contract`]. [`Position::new`] is the raw constructor for callers
//! that already hold the parts.

use crate::prelude::*;

/// Why a [`Position`] cannot be built from its parts or derived from a deal.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum PositionError {
    /// The board carries no contract, so the strain, the opening leader or the
    /// winner of a completed trick cannot be determined.
    MissingContract,
    /// The requested play prefix is longer than the recorded play.
    PrefixOutOfRange {
        /// How many plays the caller asked for.
        requested: usize,
        /// How many plays the board records.
        recorded: usize,
    },
    /// The table holds more than the three cards of an incomplete trick.
    TableTooLong {
        /// How many cards the caller put on the table.
        len: usize,
    },
    /// A hand holds more than the thirteen cards of a full bridge hand.
    HandTooLong {
        /// The offending seat.
        seat: Seat,
        /// How many cards that seat holds.
        len: u8,
    },
    /// A card appears twice across the table and hands.
    DuplicatedCard(Card),
    /// The table is not the trailing incomplete trick: the number of cards
    /// played does not leave exactly `table` cards on the table.
    TableNotTrailing {
        /// How many cards the hands imply have been played, `52 - |hands|`.
        played: usize,
        /// How many cards are actually on the table.
        table: usize,
    },
    /// The table's plays are not consecutive in rotation.
    TableSeatsNotConsecutive {
        /// The seat that played out of turn.
        seat: Seat,
        /// The seat that should have played.
        expected: Seat,
    },
    /// A prefix card is not in the hand of the seat recorded as having played
    /// it.
    PrefixCardNotHeld {
        /// The offending play.
        play: Play,
    },
    /// A prefix card is played more than once.
    PrefixCardPlayedTwice(Card),
}

impl std::fmt::Display for PositionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MissingContract => {
                write!(f, "the board has no contract to derive a position from")
            }
            Self::PrefixOutOfRange {
                requested,
                recorded,
            } => write!(
                f,
                "requested a position after {requested} plays, but only {recorded} are recorded"
            ),
            Self::TableTooLong { len } => write!(
                f,
                "the table holds {len} cards, but an incomplete trick has at most 3"
            ),
            Self::HandTooLong { seat, len } => {
                write!(f, "{seat} holds {len} cards, expected at most 13")
            }
            Self::DuplicatedCard(card) => write!(f, "card {card} appears more than once"),
            Self::TableNotTrailing { played, table } => write!(
                f,
                "{played} cards have been played but the table holds {table}, not {played} % 4"
            ),
            Self::TableSeatsNotConsecutive { seat, expected } => write!(
                f,
                "the table plays out of rotation: {seat} played where {expected} should have"
            ),
            Self::PrefixCardNotHeld { play } => write!(
                f,
                "{} is recorded as playing {}, which the deal does not give {0}",
                play.seat, play.card
            ),
            Self::PrefixCardPlayedTwice(card) => {
                write!(f, "card {card} is played more than once in the prefix")
            }
        }
    }
}

impl std::error::Error for PositionError {}

/// The winner of a complete four-card trick under `trump`.
///
/// This is the canonical trick-winner rule: a card can only win by being a
/// trump or by following the led suit; among cards of the same class the higher
/// rank wins, and any trump beats any non-trump. `trick` must be a complete
/// trick, led by its first play.
#[must_use]
pub fn trick_winner(trick: &[Play], trump: Strain) -> Seat {
    debug_assert_eq!(trick.len(), 4, "a completed trick has four plays");
    let led = trick[0].card.suit();
    let trump = Suit::try_from(trump).ok();
    // Rank a play by `(is_trump, effective_rank)`: discards rank zero, so they
    // can never outrank a card that followed suit or a trump.
    let strength = |play: &Play| {
        let is_trump = Some(play.card.suit()) == trump;
        let rank = if is_trump || play.card.suit() == led {
            play.card.rank()
        } else {
            0
        };
        (is_trump, rank)
    };
    trick
        .iter()
        .max_by_key(|play| strength(play))
        .map_or(trick[0].seat, |play| play.seat)
}

/// Inserts `card` into `seen`, returning `true` when it was absent.
fn insert_if_absent(seen: &mut Cards, card: Card) -> bool {
    if seen.has_card(card) {
        return false;
    }
    *seen = seen.insert(card);
    true
}

/// A snapshot of a partially played board.
///
/// `NoTrumps` is encoded directly in [`Strain`] rather than as an `Option`;
/// the DDS adapter maps the four suit strains and `NoTrumps` onto DDS trump
/// codes 0..4.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Position {
    trump: Strain,
    to_play: Seat,
    table: Vec<Play>,
    hands: [Cards; 4],
}

impl Position {
    /// Builds a position directly from its parts.
    ///
    /// `new` validates the shape of a snapshot past any point of the play, not
    /// a full deck: the table holds at most the three cards of an incomplete
    /// trick; every hand holds at most 13 cards; the table and hands are
    /// distinct; the played count `52 - |hands|` leaves exactly `table.len()`
    /// cards on the table; and the table's plays are consecutive in rotation.
    /// Cards played to completed tricks are in neither the table nor the hands,
    /// so the supplied set is smaller than 52. `to_play` is the caller's claim
    /// and is never recomputed.
    ///
    /// A four-card table is rejected rather than admitted as a second
    /// representation of a completed trick: that state is an empty table plus
    /// the winner to play.
    ///
    /// # Errors
    ///
    /// Returns [`PositionError::TableTooLong`], [`PositionError::HandTooLong`],
    /// [`PositionError::DuplicatedCard`], [`PositionError::TableNotTrailing`] or
    /// [`PositionError::TableSeatsNotConsecutive`] for the corresponding
    /// invariant.
    pub fn new(
        trump: Strain,
        to_play: Seat,
        table: Vec<Play>,
        hands: [Cards; 4],
    ) -> Result<Self, PositionError> {
        if table.len() > 3 {
            return Err(PositionError::TableTooLong { len: table.len() });
        }
        for (index, hand) in hands.iter().enumerate() {
            if hand.len() > 13 {
                return Err(PositionError::HandTooLong {
                    seat: Seat::from(index),
                    len: hand.len(),
                });
            }
        }

        let mut seen = Cards::EMPTY;
        for play in &table {
            if !insert_if_absent(&mut seen, play.card) {
                return Err(PositionError::DuplicatedCard(play.card));
            }
        }
        for hand in &hands {
            for card in *hand {
                if !insert_if_absent(&mut seen, card) {
                    return Err(PositionError::DuplicatedCard(card));
                }
            }
        }

        // The hands' complement is everything played; a trailing incomplete
        // trick means that count is `table.len()` more than a multiple of four.
        let played = usize::from(52 - hands.iter().map(Cards::len).sum::<u8>());
        if played % 4 != table.len() {
            return Err(PositionError::TableNotTrailing {
                played,
                table: table.len(),
            });
        }

        if let Some(first) = table.first() {
            for (offset, play) in table.iter().enumerate() {
                let expected = first.seat + offset;
                if play.seat != expected {
                    return Err(PositionError::TableSeatsNotConsecutive {
                        seat: play.seat,
                        expected,
                    });
                }
            }
        }

        Ok(Self {
            trump,
            to_play,
            table,
            hands,
        })
    }

    /// Derives the position after a recorded play prefix.
    ///
    /// `prefix` is the plays made so far, in order. The remaining hands are the
    /// deal minus the prefix, the table is the trailing incomplete trick, and
    /// `to_play` is `leader` for an empty prefix, the next seat mid-trick, or
    /// the winner of the last completed trick. `leader` is only consulted when
    /// `prefix` is empty, so a mid-play caller without a contract may pass any
    /// seat.
    ///
    /// Unlike [`Position::new`]'s caller-blessed `to_play`, the prefix has a
    /// checkable ground truth in `deal`, so every play is validated: its card
    /// must be held by the seat recorded as having played it and must not be
    /// played twice.
    ///
    /// # Errors
    ///
    /// Returns [`PositionError::PrefixCardNotHeld`] or
    /// [`PositionError::PrefixCardPlayedTwice`] for an inconsistent prefix, and
    /// propagates [`Position::new`]'s shape errors if the derived table is not
    /// a valid snapshot (for example seats that do not rotate).
    pub fn from_deal_prefix(
        deal: &Deal,
        prefix: &[Play],
        strain: Strain,
        leader: Seat,
    ) -> Result<Self, PositionError> {
        let mut seen = Cards::EMPTY;
        for play in prefix {
            if !deal[play.seat].has_card(play.card) {
                return Err(PositionError::PrefixCardNotHeld { play: *play });
            }
            if !insert_if_absent(&mut seen, play.card) {
                return Err(PositionError::PrefixCardPlayedTwice(play.card));
            }
        }

        let mut hands = deal.hands().map(|hand| hand.cards());
        for play in prefix {
            hands[play.seat as usize] = hands[play.seat as usize].remove(play.card);
        }

        let table = prefix[prefix.len() - prefix.len() % 4..].to_vec();
        let to_play = if prefix.is_empty() {
            leader
        } else if prefix.len().is_multiple_of(4) {
            trick_winner(&prefix[prefix.len() - 4..], strain)
        } else {
            prefix[prefix.len() - 1].seat.next()
        };

        Self::new(strain, to_play, table, hands)
    }

    /// Derives the position after a recorded play prefix under `contract`.
    ///
    /// The sibling of [`Position::from_deal_prefix`] for callers holding a
    /// [`Contract`]: the contract supplies the strain and the opening leader.
    ///
    /// # Errors
    ///
    /// See [`Position::from_deal_prefix`].
    pub fn from_deal_prefix_with_contract(
        deal: &Deal,
        prefix: &[Play],
        contract: Contract,
    ) -> Result<Self, PositionError> {
        Self::from_deal_prefix(deal, prefix, contract.strain(), contract.leader())
    }

    /// Derives the current position: the one after every recorded play.
    ///
    /// The board's claim, if any, is ignored: a claim ends the play without
    /// contributing cards, so the position reflects the plays before it.
    ///
    /// # Errors
    ///
    /// Returns [`PositionError::MissingContract`] when the board has no
    /// contract, and the prefix errors of [`Position::from_deal_prefix`] when
    /// the recorded play is inconsistent with the deal.
    pub fn from_board(board: &Board) -> Result<Self, PositionError> {
        Self::after_play(board, board.play().map_or(0, PlayRecord::len))
    }

    /// Derives the position after `n` plays, the rewind primitive.
    ///
    /// `n == play.len()` is the current position and `n == 0` is the position
    /// before the opening lead. [`Position::trump`] is mandatory, so the
    /// contract's strain is needed in every case; the contract also supplies the
    /// opening leader and the winner of a completed trick.
    ///
    /// # Errors
    ///
    /// Returns [`PositionError::PrefixOutOfRange`] when `n` exceeds the
    /// recorded play, [`PositionError::MissingContract`] when the board has no
    /// contract, and the prefix errors of [`Position::from_deal_prefix`] when
    /// the recorded play does not match the deal.
    pub fn after_play(board: &Board, n: usize) -> Result<Self, PositionError> {
        let plays: &[Play] = board.play().map_or(&[], PlayRecord::plays);
        if n > plays.len() {
            return Err(PositionError::PrefixOutOfRange {
                requested: n,
                recorded: plays.len(),
            });
        }
        let contract = board.contract().ok_or(PositionError::MissingContract)?;
        Self::from_deal_prefix(
            board.deal(),
            &plays[..n],
            contract.strain(),
            contract.leader(),
        )
    }

    /// The strain solved for; `NoTrumps` means no trump.
    #[must_use]
    pub const fn trump(&self) -> Strain {
        self.trump
    }

    /// The seat to play next.
    #[must_use]
    pub const fn to_play(&self) -> Seat {
        self.to_play
    }

    /// The cards already played to the current (incomplete) trick, in order.
    #[must_use]
    pub fn table(&self) -> &[Play] {
        &self.table
    }

    /// How many cards are on the table: `0..=3`.
    #[must_use]
    pub fn trick_len(&self) -> usize {
        self.table.len()
    }

    /// True only before the opening lead, i.e. when `n == 0`.
    ///
    /// A completed trick also leaves [`Position::table`] empty, so this cannot
    /// be read off `table`; instead it is true exactly when every card is still
    /// in hand.
    #[must_use]
    pub fn is_opening_lead(&self) -> bool {
        self.hands.iter().map(Cards::len).sum::<u8>() == 52
    }

    /// The remaining cards held by `seat`.
    #[must_use]
    pub fn hand(&self, seat: Seat) -> Cards {
        self.hands[seat as usize]
    }

    /// The remaining cards per seat, indexed by [`Seat`].
    #[must_use]
    pub const fn hands(&self) -> &[Cards; 4] {
        &self.hands
    }

    /// The cards `to_play` may legally play, respecting follow-suit.
    ///
    /// If the player holds the led suit, only those cards are legal; otherwise
    /// the whole hand is. The opening leader is unconstrained. This is the
    /// candidate set card-EV and lead analysis iterate.
    #[must_use]
    pub fn legal_plays(&self) -> Cards {
        let hand = self.hand(self.to_play);
        match self.table.first() {
            Some(led) => {
                let following = hand.in_suit(led.card.suit());
                if following.is_empty() {
                    hand
                } else {
                    following
                }
            }
            None => hand,
        }
    }
}

impl std::fmt::Display for Position {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{} to play, trump {}", self.to_play, self.trump)?;
        write!(f, "table:")?;
        if self.table.is_empty() {
            write!(f, " (empty)")?;
        } else {
            for play in &self.table {
                write!(f, " {} {}", play.seat, play.card)?;
            }
        }
        writeln!(f)?;
        for seat in Seat::iter() {
            writeln!(f, "{}: {}", seat, self.hand(seat))?;
        }
        Ok(())
    }
}

/// A cursor over a [`Board`]'s play, deriving positions on demand.
///
/// The cursor stores only an index; every step re-derives the position from
/// the immutable board. This trades an O(52) recomputation per step for the
/// absence of mutable undo state, keeping [`Position`] a pure value type.
#[derive(Debug, Clone, Copy)]
pub struct Playthrough<'a> {
    board: &'a Board,
    index: usize,
}

impl<'a> Playthrough<'a> {
    /// Starts the cursor before the opening lead (`index == 0`).
    #[must_use]
    pub const fn new(board: &'a Board) -> Self {
        Self { board, index: 0 }
    }

    /// The position at the cursor's current index.
    ///
    /// # Errors
    ///
    /// See [`Position::after_play`].
    pub fn position(&self) -> Result<Position, PositionError> {
        Position::after_play(self.board, self.index)
    }

    /// Advances one play and returns the new position, or `None` at the end.
    ///
    /// # Errors
    ///
    /// See [`Position::after_play`].
    // The fallible result rules out `Iterator`; the name is the ticket's API.
    #[allow(clippy::should_implement_trait)]
    pub fn next(&mut self) -> Option<Result<Position, PositionError>> {
        if self.index >= self.len() {
            return None;
        }
        self.index += 1;
        Some(self.position())
    }

    /// Rewinds one play and returns the new position, or `None` at the start.
    ///
    /// # Errors
    ///
    /// See [`Position::after_play`].
    pub fn prev(&mut self) -> Option<Result<Position, PositionError>> {
        if self.index == 0 {
            return None;
        }
        self.index -= 1;
        Some(self.position())
    }

    /// Moves the cursor to `n`, clamped to the length of the recorded play.
    pub fn seek(&mut self, n: usize) {
        self.index = n.min(self.len());
    }

    /// The number of recorded plays.
    #[must_use]
    pub fn len(&self) -> usize {
        self.board.play().map_or(0, PlayRecord::len)
    }

    /// True when the board records no play.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// The cursor's current index: a play count in `0..=len`.
    #[must_use]
    pub const fn index(&self) -> usize {
        self.index
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn one_suit_deal() -> Deal {
        Deal::new([
            Hand::from_str("SAKQJT98765432").expect("spades hand parses"),
            Hand::from_str("HAKQJT98765432").expect("hearts hand parses"),
            Hand::from_str("DAKQJT98765432").expect("diamonds hand parses"),
            Hand::from_str("CAKQJT98765432").expect("clubs hand parses"),
        ])
        .expect("one suit per seat is a valid deal")
    }

    /// A deal in which every seat holds every suit, so follow-suit is testable.
    fn round_robin_deal() -> Deal {
        Deal::new([
            Hand::from_str("SA6T2 HK95 DQ84 CJ73").expect("north hand parses"),
            Hand::from_str("SJ73 HA6T2 DK95 CQ84").expect("east hand parses"),
            Hand::from_str("SQ84 HJ73 DA6T2 CK95").expect("south hand parses"),
            Hand::from_str("SK95 HQ84 DJ73 CA6T2").expect("west hand parses"),
        ])
        .expect("round robin deal is a valid partition")
    }

    fn contract(level: u8, strain: Strain, declarer: Seat) -> Contract {
        Contract::new(level, strain, declarer, Vulnerable::No, Doubled::NotDoubled)
    }

    fn board_with(deal: Deal, contract: Contract, plays: Vec<Play>) -> Board {
        Board::builder(deal, Seat::North, Vulnerability::None)
            .contract(contract)
            .play(PlayRecord::new(plays))
            .build()
            .expect("deal, contract and plays fit together")
    }

    /// The cards of every seat, copied out of a deal.
    fn full_hands(deal: &Deal) -> [Cards; 4] {
        deal.hands().map(|hand| hand.cards())
    }

    /// The hands left after removing `prefix` from `deal`, as `new` expects.
    fn hands_after(deal: &Deal, prefix: &[Play]) -> [Cards; 4] {
        let mut hands = full_hands(deal);
        for play in prefix {
            hands[play.seat as usize] = hands[play.seat as usize].remove(play.card);
        }
        hands
    }

    #[test]
    fn new_accepts_a_completed_partition_with_an_empty_table() {
        let deal = round_robin_deal();
        let hands = full_hands(&deal);
        let position = Position::new(Strain::NoTrumps, Seat::West, Vec::new(), hands)
            .expect("a full deal plus an empty table is a valid position");

        assert_eq!(position.trump(), Strain::NoTrumps);
        assert_eq!(position.to_play(), Seat::West);
        assert!(position.is_opening_lead());
        assert_eq!(position.hands(), &hands);
    }

    #[test]
    fn new_accepts_a_valid_mid_trick_position() {
        let deal = round_robin_deal();
        let table = vec![Play::new(Seat::North, Card::S2)];
        let hands = hands_after(&deal, &table);
        let position = Position::new(Strain::Spades, Seat::East, table.clone(), hands)
            .expect("a mid-trick position is valid");

        assert_eq!(position.table(), table);
        assert_eq!(position.trick_len(), 1);
        assert_eq!(position.to_play(), Seat::East);
        assert!(!position.is_opening_lead());
        assert_eq!(position.hand(Seat::North).len(), 12);
    }

    #[test]
    fn new_trusts_the_callers_to_play() {
        // A mid-trick table whose `to_play` is not the next seat: `new` never
        // recomputes it, so the caller's claim stands.
        let deal = round_robin_deal();
        let table = vec![Play::new(Seat::North, Card::S2)];
        let hands = hands_after(&deal, &table);
        let position = Position::new(Strain::Spades, Seat::South, table, hands)
            .expect("the partition is what `new` validates");

        assert_eq!(position.to_play(), Seat::South);
    }

    #[test]
    fn new_rejects_a_table_of_four_cards() {
        let deal = round_robin_deal();
        let table = vec![
            Play::new(Seat::North, Card::S2),
            Play::new(Seat::East, Card::H2),
            Play::new(Seat::South, Card::D2),
            Play::new(Seat::West, Card::C2),
        ];
        // The table bound is checked before the duplicate fold, so the
        // duplicated cards do not mask the real complaint.
        assert_eq!(
            Position::new(Strain::NoTrumps, Seat::North, table, full_hands(&deal)),
            Err(PositionError::TableTooLong { len: 4 })
        );
    }

    #[test]
    fn new_rejects_a_card_duplicated_between_table_and_hands() {
        let deal = round_robin_deal();
        let duplicated = Play::new(Seat::North, Card::S2);
        assert_eq!(
            Position::new(
                Strain::NoTrumps,
                Seat::East,
                vec![duplicated],
                full_hands(&deal),
            ),
            Err(PositionError::DuplicatedCard(Card::S2))
        );
    }

    #[test]
    fn new_accepts_a_position_past_the_third_play() {
        // A completed trick leaves its four cards in neither the table nor the
        // hands, so the supplied set is smaller than a full deck.
        let deal = round_robin_deal();
        let completed = [
            Play::new(Seat::North, Card::S2),
            Play::new(Seat::East, Card::H2),
            Play::new(Seat::South, Card::D2),
            Play::new(Seat::West, Card::C2),
        ];
        let hands = hands_after(&deal, &completed);
        let position = Position::new(Strain::NoTrumps, Seat::East, Vec::new(), hands)
            .expect("a completed-trick snapshot is a valid position");

        assert!(position.table().is_empty());
        assert!(!position.is_opening_lead());
        assert_eq!(position.to_play(), Seat::East);
        assert_eq!(position.hand(Seat::North).len(), 12);
    }

    #[test]
    fn new_rejects_a_hand_of_more_than_thirteen_cards() {
        let deal = one_suit_deal();
        let mut hands = full_hands(&deal);
        hands[Seat::North as usize] = hands[Seat::North as usize].insert(Card::H2);
        assert_eq!(
            Position::new(Strain::NoTrumps, Seat::North, Vec::new(), hands),
            Err(PositionError::HandTooLong {
                seat: Seat::North,
                len: 14,
            })
        );
    }

    #[test]
    fn new_rejects_a_table_that_is_not_the_trailing_trick() {
        // One card is missing from the hands, but the table is empty: the
        // played card belongs to no incomplete trick.
        let deal = round_robin_deal();
        let mut hands = full_hands(&deal);
        hands[Seat::North as usize] = hands[Seat::North as usize].remove(Card::S2);
        assert_eq!(
            Position::new(Strain::NoTrumps, Seat::North, Vec::new(), hands),
            Err(PositionError::TableNotTrailing {
                played: 1,
                table: 0
            })
        );
    }

    #[test]
    fn new_rejects_a_table_whose_seats_are_not_consecutive() {
        let deal = round_robin_deal();
        let table = vec![
            Play::new(Seat::North, Card::S2),
            Play::new(Seat::South, Card::D2),
        ];
        let hands = hands_after(&deal, &table);
        assert_eq!(
            Position::new(Strain::NoTrumps, Seat::West, table, hands),
            Err(PositionError::TableSeatsNotConsecutive {
                seat: Seat::South,
                expected: Seat::East,
            })
        );
    }

    #[test]
    fn from_deal_prefix_rejects_a_card_not_held_by_its_seat() {
        let deal = round_robin_deal();
        // S2 belongs to North; East cannot lead it.
        let prefix = [Play::new(Seat::East, Card::S2)];
        assert_eq!(
            Position::from_deal_prefix(&deal, &prefix, Strain::NoTrumps, Seat::North),
            Err(PositionError::PrefixCardNotHeld {
                play: Play::new(Seat::East, Card::S2),
            })
        );
    }

    #[test]
    fn from_deal_prefix_rejects_a_card_played_twice() {
        let deal = round_robin_deal();
        let prefix = [
            Play::new(Seat::North, Card::S2),
            Play::new(Seat::East, Card::S3),
            Play::new(Seat::North, Card::S2),
        ];
        assert_eq!(
            Position::from_deal_prefix(&deal, &prefix, Strain::NoTrumps, Seat::North),
            Err(PositionError::PrefixCardPlayedTwice(Card::S2))
        );
    }

    #[test]
    fn from_deal_prefix_rejects_a_table_that_is_not_consecutive() {
        // North and South cannot be the first two plays of a trick.
        let deal = round_robin_deal();
        let prefix = [
            Play::new(Seat::North, Card::S2),
            Play::new(Seat::South, Card::D2),
        ];
        assert_eq!(
            Position::from_deal_prefix(&deal, &prefix, Strain::NoTrumps, Seat::West),
            Err(PositionError::TableSeatsNotConsecutive {
                seat: Seat::South,
                expected: Seat::East,
            })
        );
    }

    #[test]
    fn from_deal_prefix_opening_lead_uses_the_explicit_leader() {
        let deal = one_suit_deal();
        for (strain, leader) in [
            (Strain::Spades, Seat::North),
            (Strain::Hearts, Seat::East),
            (Strain::Diamonds, Seat::South),
            (Strain::Clubs, Seat::West),
            (Strain::NoTrumps, Seat::North),
        ] {
            let position = Position::from_deal_prefix(&deal, &[], strain, leader)
                .expect("the empty prefix derives the opening position");

            assert_eq!(position.trump(), strain);
            assert_eq!(position.to_play(), leader, "leader under {strain}");
            assert!(position.is_opening_lead());
            assert!(position.table().is_empty());
            for seat in Seat::iter() {
                assert_eq!(position.hand(seat), deal[seat].cards());
            }
        }
    }

    #[test]
    fn from_deal_prefix_mid_trick_hands_the_turn_to_the_next_seat() {
        let deal = one_suit_deal();
        let prefix = [Play::new(Seat::North, Card::S2)];
        for strain in Strain::ALL {
            let position = Position::from_deal_prefix(&deal, &prefix, strain, Seat::West)
                .expect("a valid prefix derives a position");

            assert_eq!(position.trump(), strain);
            assert_eq!(position.to_play(), Seat::East, "after North comes East");
            assert_eq!(position.table(), &prefix);
            assert_eq!(position.hand(Seat::North).len(), 12);
        }
    }

    #[test]
    fn from_deal_prefix_completed_trick_hands_the_turn_to_the_winner() {
        let deal = one_suit_deal();
        // North leads a spade; every other seat answers in its own suit, so
        // the winner moves with the strain.
        let prefix = [
            Play::new(Seat::North, Card::S2),
            Play::new(Seat::East, Card::H2),
            Play::new(Seat::South, Card::D2),
            Play::new(Seat::West, Card::C2),
        ];
        let expected = [
            (Strain::Spades, Seat::North),
            (Strain::Hearts, Seat::East),
            (Strain::Diamonds, Seat::South),
            (Strain::Clubs, Seat::West),
            (Strain::NoTrumps, Seat::North),
        ];
        for (strain, winner) in expected {
            let position = Position::from_deal_prefix(&deal, &prefix, strain, Seat::West)
                .expect("a valid prefix derives a position");

            assert_eq!(position.trump(), strain);
            assert_eq!(position.to_play(), winner, "winner under {strain}");
            assert!(position.table().is_empty());
            assert!(!position.is_opening_lead());
        }
    }

    #[test]
    fn from_deal_prefix_with_contract_reads_strain_and_leader_from_the_contract() {
        let deal = one_suit_deal();
        // West declares, so North is the opening leader.
        let position = Position::from_deal_prefix_with_contract(
            &deal,
            &[],
            contract(3, Strain::Hearts, Seat::West),
        )
        .expect("the contract supplies the leader and strain");

        assert_eq!(position.trump(), Strain::Hearts);
        assert_eq!(position.to_play(), Seat::North);
    }

    #[test]
    fn trick_winner_rule_covers_the_edge_cases() {
        let cases = [
            // A low trump beats the highest card of the led suit.
            (
                [
                    Play::new(Seat::North, Card::HA),
                    Play::new(Seat::South, Card::S2),
                    Play::new(Seat::West, Card::H2),
                    Play::new(Seat::East, Card::H3),
                ],
                Strain::Spades,
                Seat::South,
            ),
            // An off-suit, non-trump discard never wins: the highest led card does.
            (
                [
                    Play::new(Seat::North, Card::HA),
                    Play::new(Seat::East, Card::CK),
                    Play::new(Seat::South, Card::H2),
                    Play::new(Seat::West, Card::H3),
                ],
                Strain::Diamonds,
                Seat::North,
            ),
            // No trump: the highest led card wins.
            (
                [
                    Play::new(Seat::North, Card::H10),
                    Play::new(Seat::East, Card::HA),
                    Play::new(Seat::South, Card::C2),
                    Play::new(Seat::West, Card::D5),
                ],
                Strain::NoTrumps,
                Seat::East,
            ),
            // The led suit is the trump suit: ranks decide, no ruff exists.
            (
                [
                    Play::new(Seat::North, Card::S10),
                    Play::new(Seat::East, Card::SA),
                    Play::new(Seat::South, Card::S2),
                    Play::new(Seat::West, Card::S3),
                ],
                Strain::Spades,
                Seat::East,
            ),
        ];
        for (trick, trump, expected) in cases {
            assert_eq!(trick_winner(&trick, trump), expected, "{trick:?}");
        }
    }

    #[test]
    fn opening_lead_position_has_full_hands_and_no_table() {
        let deal = one_suit_deal();
        let board = board_with(deal, contract(3, Strain::NoTrumps, Seat::West), vec![]);
        let position = Position::after_play(&board, 0).expect("a contract is present");

        assert_eq!(position.trump(), Strain::NoTrumps);
        assert_eq!(position.to_play(), Seat::North);
        assert!(position.is_opening_lead());
        assert_eq!(position.trick_len(), 0);
        assert!(position.table().is_empty());
        for seat in Seat::iter() {
            assert_eq!(position.hand(seat), deal[seat].cards());
        }
    }

    #[test]
    fn mid_trick_position_removes_played_cards_and_points_at_next_seat() {
        let deal = one_suit_deal();
        let board = board_with(
            deal,
            contract(3, Strain::NoTrumps, Seat::West),
            vec![Play::new(Seat::North, Card::S2)],
        );
        let position = Position::after_play(&board, 1).expect("a contract is present");

        assert_eq!(position.table(), &[Play::new(Seat::North, Card::S2)]);
        assert_eq!(position.trick_len(), 1);
        assert!(!position.is_opening_lead());
        assert_eq!(position.to_play(), Seat::East);
        assert_eq!(position.hand(Seat::North).len(), 12);
        assert!(!position.hand(Seat::North).has_card(Card::S2));
        assert_eq!(position.hand(Seat::East), deal[Seat::East].cards());
    }

    #[test]
    fn completed_trick_hands_the_lead_to_the_trick_winner() {
        // West leads a club; East trumps with a heart and wins.
        let deal = one_suit_deal();
        let plays = vec![
            Play::new(Seat::West, Card::C2),
            Play::new(Seat::North, Card::S2),
            Play::new(Seat::East, Card::H2),
            Play::new(Seat::South, Card::D2),
        ];
        let board = board_with(deal, contract(4, Strain::Hearts, Seat::South), plays);
        let position = Position::after_play(&board, 4).expect("a contract is present");

        assert!(position.table().is_empty());
        assert_eq!(position.trick_len(), 0);
        assert_eq!(position.to_play(), Seat::East);
        assert_eq!(position.hand(Seat::West).len(), 12);
    }

    #[test]
    fn no_trump_trick_is_won_by_the_highest_led_suit_card() {
        // The same cards with no trump: the only club (West's) wins.
        let deal = one_suit_deal();
        let plays = vec![
            Play::new(Seat::West, Card::C2),
            Play::new(Seat::North, Card::S2),
            Play::new(Seat::East, Card::H2),
            Play::new(Seat::South, Card::D2),
        ];
        let board = board_with(deal, contract(3, Strain::NoTrumps, Seat::South), plays);
        let position = Position::after_play(&board, 4).expect("a contract is present");
        assert_eq!(position.to_play(), Seat::West);
    }

    #[test]
    fn is_opening_lead_is_true_only_before_the_first_play() {
        // West leads a club; East trumps with a heart and wins.
        let deal = one_suit_deal();
        let plays = vec![
            Play::new(Seat::West, Card::C2),
            Play::new(Seat::North, Card::S2),
            Play::new(Seat::East, Card::H2),
            Play::new(Seat::South, Card::D2),
        ];
        let board = board_with(deal, contract(4, Strain::Hearts, Seat::South), plays);

        let opening = Position::after_play(&board, 0).expect("a contract is present");
        assert!(opening.is_opening_lead());
        assert_eq!(opening.trick_len(), 0);

        let mid_trick = Position::after_play(&board, 1).expect("a contract is present");
        assert!(!mid_trick.is_opening_lead());
        assert_eq!(mid_trick.trick_len(), 1);

        // The winner is on lead at the start of trick two: the table is empty
        // but this is not the opening lead.
        let next_trick = Position::after_play(&board, 4).expect("a contract is present");
        assert!(!next_trick.is_opening_lead());
        assert_eq!(next_trick.trick_len(), 0);
    }

    #[test]
    fn from_board_derives_the_current_position() {
        let plays = vec![
            Play::new(Seat::North, Card::S2),
            Play::new(Seat::East, Card::S3),
            Play::new(Seat::South, Card::S4),
        ];
        let board = board_with(
            round_robin_deal(),
            contract(3, Strain::NoTrumps, Seat::West),
            plays,
        );
        assert_eq!(
            Position::from_board(&board).expect("a contract is present"),
            Position::after_play(&board, 3).expect("a contract is present"),
        );
    }

    #[test]
    fn derivation_without_a_contract_is_an_error() {
        let board = Board::builder(one_suit_deal(), Seat::North, Vulnerability::None)
            .play(PlayRecord::new(vec![Play::new(Seat::North, Card::S2)]))
            .build()
            .expect("a bare board is valid");
        assert_eq!(
            Position::from_board(&board),
            Err(PositionError::MissingContract)
        );
    }

    #[test]
    fn after_play_beyond_the_recorded_play_is_an_error() {
        let board = board_with(
            round_robin_deal(),
            contract(3, Strain::NoTrumps, Seat::West),
            vec![Play::new(Seat::North, Card::S2)],
        );
        assert_eq!(
            Position::after_play(&board, 2),
            Err(PositionError::PrefixOutOfRange {
                requested: 2,
                recorded: 1,
            })
        );
    }

    #[test]
    fn a_claim_leaves_the_position_at_the_recorded_plays() {
        let deal = round_robin_deal();
        let plays = vec![
            Play::new(Seat::North, Card::S2),
            Play::new(Seat::East, Card::S3),
        ];
        let contract = contract(3, Strain::NoTrumps, Seat::West);
        let without_claim = Board::builder(deal, Seat::North, Vulnerability::None)
            .contract(contract)
            .play(PlayRecord::new(plays.clone()))
            .build()
            .expect("valid board");
        let with_claim = Board::builder(deal, Seat::North, Vulnerability::None)
            .contract(contract)
            .play(PlayRecord::new(plays))
            .claim(7)
            .build()
            .expect("valid board");

        assert_eq!(
            Position::from_board(&with_claim).expect("a contract is present"),
            Position::from_board(&without_claim).expect("a contract is present"),
        );
    }

    #[test]
    fn must_follow_suit_when_holding_the_led_suit() {
        let board = board_with(
            round_robin_deal(),
            contract(3, Strain::NoTrumps, Seat::West),
            vec![Play::new(Seat::North, Card::S2)],
        );
        let position = Position::after_play(&board, 1).expect("a contract is present");
        assert_eq!(position.to_play(), Seat::East);
        assert_eq!(
            position.legal_plays(),
            Cards::from_str("SJ73").expect("east's spades parse")
        );
    }

    #[test]
    fn a_void_player_may_play_any_card() {
        let deal = one_suit_deal();
        let board = board_with(
            deal,
            contract(3, Strain::NoTrumps, Seat::West),
            vec![Play::new(Seat::North, Card::S2)],
        );
        let position = Position::after_play(&board, 1).expect("a contract is present");
        assert_eq!(position.to_play(), Seat::East);
        // East is void in spades, so the whole (heart) hand is legal.
        assert_eq!(position.legal_plays(), deal[Seat::East].cards());
    }

    #[test]
    fn the_opening_leader_may_play_any_card() {
        let deal = one_suit_deal();
        let board = board_with(deal, contract(3, Strain::NoTrumps, Seat::West), vec![]);
        let position = Position::after_play(&board, 0).expect("a contract is present");
        assert_eq!(position.to_play(), Seat::North);
        assert_eq!(position.legal_plays(), deal[Seat::North].cards());
    }

    #[test]
    fn playthrough_navigation_reproduces_every_prefix() {
        let plays = vec![
            Play::new(Seat::North, Card::S2),
            Play::new(Seat::East, Card::S3),
            Play::new(Seat::South, Card::S4),
            Play::new(Seat::West, Card::S5),
            Play::new(Seat::North, Card::S6),
        ];
        let board = board_with(
            round_robin_deal(),
            contract(3, Strain::NoTrumps, Seat::West),
            plays,
        );
        let expected: Vec<_> = (0..=5)
            .map(|n| Position::after_play(&board, n).expect("a contract is present"))
            .collect();

        let mut playthrough = Playthrough::new(&board);
        assert_eq!(playthrough.len(), 5);
        assert_eq!(
            playthrough.position().expect("a contract is present"),
            expected[0]
        );

        for (n, position) in expected.iter().enumerate().skip(1) {
            let stepped = playthrough
                .next()
                .expect("not past the end")
                .expect("contract");
            assert_eq!(&stepped, position, "next should reach prefix {n}");
        }
        assert!(playthrough.next().is_none());

        for (n, position) in expected.iter().enumerate().rev().skip(1) {
            let rewound = playthrough
                .prev()
                .expect("not before the start")
                .expect("contract");
            assert_eq!(&rewound, position, "prev should reach prefix {n}");
        }
        assert!(playthrough.prev().is_none());

        playthrough.seek(2);
        assert_eq!(
            playthrough.position().expect("a contract is present"),
            expected[2]
        );
        playthrough.seek(99);
        assert_eq!(
            playthrough.position().expect("a contract is present"),
            expected[5]
        );
    }

    #[test]
    fn display_renders_the_table_and_hands() {
        let board = board_with(
            one_suit_deal(),
            contract(3, Strain::NoTrumps, Seat::West),
            vec![Play::new(Seat::North, Card::S2)],
        );
        let position = Position::after_play(&board, 1).expect("a contract is present");
        let rendered = position.to_string();
        assert!(rendered.contains("E to play, trump NT"), "{rendered}");
        assert!(rendered.contains("N 2♠"), "{rendered}");
        assert!(rendered.contains("E:"), "{rendered}");
        assert!(rendered.contains("W:"), "{rendered}");
    }
}
