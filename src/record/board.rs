// Copyright (C) 2024 Alvaro Gaiotti
// See end of file for license information

//! A recorded board: a deal plus everything a file says about it.
//!
//! [`Board`] is the unit a LIN or PBN file describes. It is the import
//! boundary, so its fields are private and construction is fallible: the
//! constructor cross-checks the parts that are present. Completeness is not
//! required — a deal-only, auction-only or claimed board is valid — and play
//! legality (revokes) is deliberately not validated, because a revoke is a
//! rules event rather than a data-integrity error.

use crate::prelude::*;
use crate::record::{Auction, AuctionError, Deal, PlayRecord};

/// Why the parts of a recorded board do not fit together.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum BoardError {
    /// The auction is not a legal four-seat rotation.
    Auction(AuctionError),
    /// A stored contract does not match the one the auction produced.
    ContractDisagrees {
        /// The contract the caller stored.
        stored: Contract,
        /// The contract derived from the auction (`None` if passed out).
        derived: Option<Contract>,
    },
    /// The board's vulnerability does not match the contract's.
    VulnerabilityDisagrees {
        /// The contract's declarer-side vulnerability.
        contract: Vulnerable,
        /// What the board's vulnerability says for the declarer.
        board: Vulnerable,
    },
    /// The play record has more than 52 cards.
    TooManyPlays {
        /// How many cards the record holds.
        len: usize,
    },
    /// A played card is not in the hand of the seat that played it.
    ForeignCard {
        /// The seat that played it.
        seat: Seat,
        /// The offending card.
        card: Card,
    },
    /// The same card was played more than once.
    DuplicatePlay(Card),
}

impl std::fmt::Display for BoardError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Auction(error) => write!(f, "illegal auction: {error}"),
            Self::ContractDisagrees { stored, derived } => match derived {
                Some(derived) => write!(
                    f,
                    "stored contract {stored} disagrees with the auction's {derived}"
                ),
                None => write!(f, "stored contract {stored} but the auction passed out"),
            },
            Self::VulnerabilityDisagrees { contract, board } => write!(
                f,
                "contract vulnerability ({contract:?}) disagrees with the board's ({board:?})"
            ),
            Self::TooManyPlays { len } => write!(f, "play record holds {len} cards, expected ≤ 52"),
            Self::ForeignCard { seat, card } => {
                write!(f, "{seat} played {card}, which is not in their hand")
            }
            Self::DuplicatePlay(card) => write!(f, "card {card} was played more than once"),
        }
    }
}

impl std::error::Error for BoardError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Auction(error) => Some(error),
            Self::ContractDisagrees { .. }
            | Self::VulnerabilityDisagrees { .. }
            | Self::TooManyPlays { .. }
            | Self::ForeignCard { .. }
            | Self::DuplicatePlay(_) => None,
        }
    }
}

impl From<AuctionError> for BoardError {
    fn from(value: AuctionError) -> Self {
        Self::Auction(value)
    }
}

/// A deal together with its dealer, vulnerability, auction, contract and play.
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Board {
    deal: Deal,
    number: Option<u8>,
    dealer: Seat,
    vulnerability: Vulnerability,
    auction: Option<Auction>,
    contract: Option<Contract>,
    play: Option<PlayRecord>,
    claim: Option<u8>,
}

impl Board {
    /// Builds a board, validating the cross-part consistency of what is present.
    ///
    /// An empty [`PlayRecord`] is normalised to `None`: it carries no
    /// information, and LIN cannot distinguish it from no play (both write no
    /// `pc|` tags), so keeping `Some(empty)` would break the round trip.
    ///
    /// # Errors
    ///
    /// Returns [`BoardError`] when the auction is illegal, a stored contract or
    /// vulnerability disagrees with the auction, or a played card is foreign,
    /// duplicated or beyond 52 cards.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        deal: Deal,
        number: Option<u8>,
        dealer: Seat,
        vulnerability: Vulnerability,
        auction: Option<Auction>,
        contract: Option<Contract>,
        play: Option<PlayRecord>,
        claim: Option<u8>,
    ) -> Result<Self, BoardError> {
        if let Some(auction) = &auction {
            auction.validate(dealer)?;
        }

        // A stored contract must agree with the auction that produced it. A
        // missing contract is fine (the board is incomplete), and so is a
        // missing auction; only a disagreement is an error.
        if let (Some(auction), Some(stored)) = (&auction, contract) {
            let derived = auction.contract(dealer, vulnerability);
            if derived != Some(stored) {
                return Err(BoardError::ContractDisagrees { stored, derived });
            }
        }

        if let Some(stored) = contract {
            let expected = Vulnerable::new(vulnerability, stored.declarer());
            if stored.vulnerable() != expected {
                return Err(BoardError::VulnerabilityDisagrees {
                    contract: stored.vulnerable(),
                    board: expected,
                });
            }
        }

        // An empty play is no play: normalising it here keeps `Some(empty)` from
        // being a second, indistinguishable representation that the writers and
        // importer would collapse anyway.
        let play = play.filter(|play| !play.is_empty());

        if let Some(play) = &play {
            if play.len() > 52 {
                return Err(BoardError::TooManyPlays { len: play.len() });
            }
            let mut seen = Cards::EMPTY;
            for recorded in play {
                if !deal[recorded.seat].has_card(recorded.card) {
                    return Err(BoardError::ForeignCard {
                        seat: recorded.seat,
                        card: recorded.card,
                    });
                }
                if seen.has_card(recorded.card) {
                    return Err(BoardError::DuplicatePlay(recorded.card));
                }
                seen = seen.insert(recorded.card);
            }
        }

        Ok(Self {
            deal,
            number,
            dealer,
            vulnerability,
            auction,
            contract,
            play,
            claim,
        })
    }

    /// Starts a builder with the three mandatory parts of a board.
    #[must_use]
    pub const fn builder(deal: Deal, dealer: Seat, vulnerability: Vulnerability) -> BoardBuilder {
        BoardBuilder {
            deal,
            number: None,
            dealer,
            vulnerability,
            auction: None,
            contract: None,
            play: None,
            claim: None,
        }
    }

    /// The four hands.
    #[must_use]
    pub const fn deal(&self) -> &Deal {
        &self.deal
    }

    /// The board number, if known.
    #[must_use]
    pub const fn number(&self) -> Option<u8> {
        self.number
    }

    /// The dealer.
    #[must_use]
    pub const fn dealer(&self) -> Seat {
        self.dealer
    }

    /// The vulnerability.
    #[must_use]
    pub const fn vulnerability(&self) -> Vulnerability {
        self.vulnerability
    }

    /// The auction, if recorded.
    #[must_use]
    pub const fn auction(&self) -> Option<&Auction> {
        self.auction.as_ref()
    }

    /// The contract, if recorded.
    #[must_use]
    pub const fn contract(&self) -> Option<Contract> {
        self.contract
    }

    /// The play, if recorded.
    #[must_use]
    pub const fn play(&self) -> Option<&PlayRecord> {
        self.play.as_ref()
    }

    /// The number of tricks claimed, if any.
    #[must_use]
    pub const fn claim(&self) -> Option<u8> {
        self.claim
    }

    /// Consumes the board and returns its deal.
    #[must_use]
    pub fn into_deal(self) -> Deal {
        self.deal
    }
}

/// Fluent builder for [`Board`]; [`BoardBuilder::build`] runs the validation.
#[derive(Debug, Clone)]
pub struct BoardBuilder {
    deal: Deal,
    number: Option<u8>,
    dealer: Seat,
    vulnerability: Vulnerability,
    auction: Option<Auction>,
    contract: Option<Contract>,
    play: Option<PlayRecord>,
    claim: Option<u8>,
}

impl BoardBuilder {
    /// Sets the board number.
    #[must_use]
    pub const fn number(mut self, number: u8) -> Self {
        self.number = Some(number);
        self
    }

    /// Sets the auction.
    #[must_use]
    pub fn auction(mut self, auction: Auction) -> Self {
        self.auction = Some(auction);
        self
    }

    /// Sets the contract.
    #[must_use]
    pub const fn contract(mut self, contract: Contract) -> Self {
        self.contract = Some(contract);
        self
    }

    /// Sets the play record.
    #[must_use]
    pub fn play(mut self, play: PlayRecord) -> Self {
        self.play = Some(play);
        self
    }

    /// Sets the claimed trick count.
    #[must_use]
    pub const fn claim(mut self, claim: u8) -> Self {
        self.claim = Some(claim);
        self
    }

    /// Validates and builds the board.
    ///
    /// # Errors
    ///
    /// See [`Board::new`].
    pub fn build(self) -> Result<Board, BoardError> {
        Board::new(
            self.deal,
            self.number,
            self.dealer,
            self.vulnerability,
            self.auction,
            self.contract,
            self.play,
            self.claim,
        )
    }
}

#[cfg(feature = "serde")]
#[derive(serde::Deserialize)]
struct BoardRepr {
    deal: Deal,
    number: Option<u8>,
    dealer: Seat,
    vulnerability: Vulnerability,
    auction: Option<Auction>,
    contract: Option<Contract>,
    play: Option<PlayRecord>,
    claim: Option<u8>,
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for Board {
    /// Deserializes the parts and re-runs [`Board::new`], so a standalone
    /// deserialized board cannot carry cross-part corruption.
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let repr = BoardRepr::deserialize(deserializer)?;
        Board::new(
            repr.deal,
            repr.number,
            repr.dealer,
            repr.vulnerability,
            repr.auction,
            repr.contract,
            repr.play,
            repr.claim,
        )
        .map_err(serde::de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use std::num::NonZeroU8;

    use super::*;
    use crate::record::{Call, RecordedCall};

    fn sample_deal() -> Deal {
        Deal::new([
            Cards::from_str("SAKQJT98765432").expect("spades hand parses"),
            Cards::from_str("HAKQJT98765432").expect("hearts hand parses"),
            Cards::from_str("DAKQJT98765432").expect("diamonds hand parses"),
            Cards::from_str("CAKQJT98765432").expect("clubs hand parses"),
        ])
        .expect("one suit per seat is a valid deal")
    }

    fn bid(level: u8, strain: Strain) -> RecordedCall {
        RecordedCall::new(Call::Bid {
            level: NonZeroU8::new(level).expect("test level is non-zero"),
            strain,
        })
    }

    fn pass() -> RecordedCall {
        RecordedCall::new(Call::Pass)
    }

    /// North 1S, East pass, South 2S, West pass, North 4S, three passes.
    fn spade_auction() -> Auction {
        Auction::new(vec![
            bid(1, Strain::Spades),
            pass(),
            bid(2, Strain::Spades),
            pass(),
            bid(4, Strain::Spades),
            pass(),
            pass(),
            pass(),
        ])
    }

    #[test]
    fn deal_only_board_is_valid() {
        let board = Board::builder(sample_deal(), Seat::North, Vulnerability::None)
            .build()
            .expect("a bare deal is a valid board");
        assert_eq!(board.number(), None);
        assert!(board.auction().is_none());
    }

    #[test]
    fn empty_play_normalises_to_no_play() {
        // `Some(empty)` and `None` are indistinguishable in LIN, so the
        // constructor collapses the former onto the latter.
        let board = Board::builder(sample_deal(), Seat::North, Vulnerability::None)
            .play(PlayRecord::new(vec![]))
            .build()
            .expect("an empty play is valid");
        assert_eq!(board.play(), None);
    }

    #[test]
    fn full_board_is_valid() {
        let contract = spade_auction()
            .contract(Seat::North, Vulnerability::None)
            .expect("auction has a contract");
        let play = PlayRecord::new(vec![
            Play::new(Seat::North, Card::S2),
            Play::new(Seat::East, Card::H2),
            Play::new(Seat::South, Card::D2),
            Play::new(Seat::West, Card::C2),
        ]);
        let board = Board::builder(sample_deal(), Seat::North, Vulnerability::None)
            .number(1)
            .auction(spade_auction())
            .contract(contract)
            .play(play)
            .claim(9)
            .build()
            .expect("parts fit together");
        assert_eq!(board.claim(), Some(9));
        assert_eq!(board.contract().expect("contract").declarer(), Seat::North);
    }

    #[test]
    fn foreign_played_card_is_rejected() {
        let play = PlayRecord::new(vec![Play::new(Seat::North, Card::H2)]);
        let error = Board::builder(sample_deal(), Seat::North, Vulnerability::None)
            .play(play)
            .build()
            .expect_err("North does not hold H2");
        assert_eq!(
            error,
            BoardError::ForeignCard {
                seat: Seat::North,
                card: Card::H2,
            }
        );
    }

    #[test]
    fn duplicate_played_card_is_rejected() {
        let play = PlayRecord::new(vec![
            Play::new(Seat::North, Card::S2),
            Play::new(Seat::North, Card::S2),
        ]);
        let error = Board::builder(sample_deal(), Seat::North, Vulnerability::None)
            .play(play)
            .build()
            .expect_err("S2 cannot be played twice");
        assert_eq!(error, BoardError::DuplicatePlay(Card::S2));
    }

    #[test]
    fn illegal_auction_is_rejected() {
        let auction = Auction::new(vec![RecordedCall::new(Call::Double)]);
        let error = Board::builder(sample_deal(), Seat::North, Vulnerability::None)
            .auction(auction)
            .build()
            .expect_err("an auction cannot open with a double");
        assert!(matches!(error, BoardError::Auction(_)));
    }

    #[test]
    fn contract_disagreeing_with_auction_is_rejected() {
        let stored = Contract::new(
            3,
            Strain::Spades,
            Seat::North,
            Vulnerable::No,
            Doubled::NotDoubled,
        );
        let error = Board::builder(sample_deal(), Seat::North, Vulnerability::None)
            .auction(spade_auction())
            .contract(stored)
            .build()
            .expect_err("stored 3S disagrees with the auction's 4S");
        assert!(matches!(error, BoardError::ContractDisagrees { .. }));
    }

    #[test]
    fn vulnerability_disagreeing_with_contract_is_rejected() {
        // The board says nobody is vulnerable, so the declarer's contract
        // cannot claim the declarer is vulnerable.
        let stored = Contract::new(
            4,
            Strain::Spades,
            Seat::North,
            Vulnerable::Yes,
            Doubled::NotDoubled,
        );
        let error = Board::builder(sample_deal(), Seat::North, Vulnerability::None)
            .contract(stored)
            .build()
            .expect_err("vulnerability disagrees");
        assert!(matches!(error, BoardError::VulnerabilityDisagrees { .. }));
    }

    #[cfg(feature = "serde")]
    #[test]
    fn board_deserialize_round_trips_through_validation() {
        let board = Board::builder(sample_deal(), Seat::North, Vulnerability::None)
            .number(7)
            .auction(spade_auction())
            .build()
            .expect("valid board");
        let json = serde_json::to_string(&board).expect("board serializes");
        let parsed: Board = serde_json::from_str(&json).expect("board deserializes");
        assert_eq!(parsed, board);
    }
}
