// Copyright (C) 2024 Alvaro Gaiotti
// See end of file for license information

//! A partially played state: the input to double-dummy solving.
//!
//! A [`Position`] is a *snapshot*, not a mutable cursor. It is a pure function
//! of a [`Board`]'s deal, its [`PlayRecord`] and its [`Contract`]'s strain, so
//! "rewinding" is prefix derivation ([`Position::after_play`]) rather than an
//! undo stack. Keeping it a value type is what lets it be serialized,
//! compared and handed to the solver unchanged.
//!
//! Deriving a position needs the contract even before the first card: the
//! strain fills [`Position::trump`] and the contract names the opening leader.

use crate::prelude::*;

/// Why a [`Position`] cannot be derived from a [`Board`].
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
        }
    }
}

impl std::error::Error for PositionError {}

/// The winner of a complete four-card trick under `trump`.
///
/// A card can only win by being a trump or by following the led suit; among
/// cards of the same class the higher rank wins, and any trump beats any
/// non-trump. `trick` must be a complete trick, led by its first play.
fn trick_winner(trick: &[Play], trump: Strain) -> Seat {
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
    /// Derives the current position: the one after every recorded play.
    ///
    /// The board's claim, if any, is ignored: a claim ends the play without
    /// contributing cards, so the position reflects the plays before it.
    ///
    /// # Errors
    ///
    /// Returns [`PositionError::MissingContract`] when the board has no
    /// contract.
    pub fn from_board(board: &Board) -> Result<Self, PositionError> {
        Self::after_play(board, board.play().map_or(0, PlayRecord::len))
    }

    /// Derives the position after `n` plays, the rewind primitive.
    ///
    /// `n == play.len()` is the current position and `n == 0` is the position
    /// before the opening lead. Deriving a position whose play prefix crosses a
    /// trick boundary needs the contract's strain to name the winner of the
    /// last completed trick; the strain is needed in every case to fill
    /// [`Position::trump`].
    ///
    /// # Errors
    ///
    /// Returns [`PositionError::PrefixOutOfRange`] when `n` exceeds the
    /// recorded play, and [`PositionError::MissingContract`] when the board has
    /// no contract.
    pub fn after_play(board: &Board, n: usize) -> Result<Self, PositionError> {
        let plays: &[Play] = board.play().map_or(&[], PlayRecord::plays);
        if n > plays.len() {
            return Err(PositionError::PrefixOutOfRange {
                requested: n,
                recorded: plays.len(),
            });
        }
        let contract = board.contract().ok_or(PositionError::MissingContract)?;

        // The remaining hands are the original hands minus every card in the
        // prefix. Only the first `n` plays are removed, so a rewind to an
        // earlier prefix restores the cards played after it.
        let mut hands = [Cards::EMPTY; 4];
        for (index, hand) in board.deal().hands().iter().enumerate() {
            hands[index] = hand.cards();
        }
        let prefix = &plays[..n];
        for play in prefix {
            hands[play.seat as usize] = hands[play.seat as usize].remove(play.card);
        }

        // The table is the trailing incomplete trick, in play order.
        let table = prefix[n - n % 4..].to_vec();
        let to_play = if n == 0 {
            contract.leader()
        } else if n.is_multiple_of(4) {
            trick_winner(&prefix[n - 4..n], contract.strain())
        } else {
            prefix[n - 1].seat.next()
        };

        Ok(Self {
            trump: contract.strain(),
            to_play,
            table,
            hands,
        })
    }

    /// Builds a position directly from its parts, bypassing [`Board`]
    /// validation.
    ///
    /// This is the crate-internal seam for callers that already hold a deal
    /// and a contract and do not want to assemble a full [`Board`] just to
    /// derive an opening position — the simulation engine, whose deals come
    /// from a [`Dealer`] rather than a file. The public path stays
    /// board-derived; the caller is responsible for the parts being mutually
    /// consistent (a `table` of at most three cards drawn from `hands`, and a
    /// `to_play` that matches the table).
    #[must_use]
    pub(crate) fn from_parts(
        trump: Strain,
        to_play: Seat,
        table: Vec<Play>,
        hands: [Cards; 4],
    ) -> Self {
        Self {
            trump,
            to_play,
            table,
            hands,
        }
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
