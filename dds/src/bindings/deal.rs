use crate::SeqError;

use super::{
    ddsffi::{boards, deal, dealPBN},
    AsDDSContract, AsRawDDS, Mode, RawDDSRef, RawDDSRefMut, Solutions, Target, MAXNOOFBOARDS,
};
use core::{
    convert::Into,
    ffi::{c_char, c_int},
    fmt::Display,
};

#[derive(Debug, RawDDSRef, Default)]
pub struct DDSCurrTrickSuit(#[raw] [c_int; 3]);

#[derive(Debug, RawDDSRef, Default)]
pub struct DDSCurrTrickRank(#[raw] [c_int; 3]);

#[allow(clippy::exhaustive_enums)]
/// How DDS encodes suits
pub enum DDSSuitEncoding {
    Spades = 0,
    Hearts = 1,
    Diamonds = 2,
    Clubs = 3,
    NoTrump = 4,
}

// See https://stackoverflow.com/questions/28028854/how-do-i-match-enum-values-with-an-integer
/// Macro for quick implementation of the `TryFrom` trait for a type
macro_rules! impl_tryfrom_dds_suit {
    ($($from:ty),*) => {
        $(impl core::convert::TryFrom<$from> for DDSSuitEncoding {
            type Error = DDSDealConstructionError;

            #[inline]
            fn try_from(value: $from) -> Result<Self, Self::Error> {
                match value {
            0 => Ok(Self::Spades),
            1 => Ok(Self::Hearts),
            2 => Ok(Self::Diamonds),
            3 => Ok(Self::Clubs),
            4 => Ok(Self::NoTrump),
            _ => Err(Self::Error::TrumpUnconvertable),
               }
            }
        })*
    };
}

impl core::convert::TryFrom<i32> for DDSSuitEncoding {
    type Error = DDSDealConstructionError;

    #[inline]
    fn try_from(value: i32) -> Result<Self, Self::Error> {
        match value {
            0i32 => Ok(Self::Spades),
            1i32 => Ok(Self::Hearts),
            2i32 => Ok(Self::Diamonds),
            3i32 => Ok(Self::Clubs),
            4i32 => Ok(Self::NoTrump),
            _ => Err(Self::Error::TrumpUnconvertable),
        }
    }
}

impl_tryfrom_dds_suit!(u8, u16, u32, usize);
impl_tryfrom_dds_suit!(i8, i16, isize);

#[allow(clippy::exhaustive_enums)]
/// How DDS encodes seat.
#[derive(Debug, Default)]
pub enum DDSHandEncoding {
    #[default]
    North = 0,
    East = 1,
    South = 2,
    West = 3,
}

/// Macro for implementing `TryFrom` from integer to [`DDSHandEncoding`]
macro_rules! impl_tryfrom_dds_hand {
    ($($from:ty),*) => {
        $(impl core::convert::TryFrom<$from> for DDSHandEncoding {
            type Error = DDSDealConstructionError;

            #[inline]
            fn try_from(value: $from) -> Result<Self, Self::Error> {
                match value {
                    0 => Ok(Self::North),
                    1 => Ok(Self::East),
                    2 => Ok(Self::South),
                    3 => Ok(Self::West),
                    _ => Err(Self::Error::TrumpUnconvertable),
                }
            }
        })*
    };
}

impl core::convert::TryFrom<i32> for DDSHandEncoding {
    type Error = DDSDealConstructionError;

    #[inline]
    fn try_from(value: i32) -> Result<Self, Self::Error> {
        match value {
            0i32 => Ok(Self::North),
            1i32 => Ok(Self::East),
            2i32 => Ok(Self::South),
            3i32 => Ok(Self::West),
            _ => Err(Self::Error::TrumpUnconvertable),
        }
    }
}
impl_tryfrom_dds_hand!(u8, u16, u32, usize, i8, i16, isize);

/// This is how DDS represents a "binary deal":
/// a array of arrays of u32, basing the order on the [`DDSHandEncoding`]
#[derive(Debug, Default, RawDDSRef)]
pub struct DDSDealRepr(#[raw] [[u32; 4]; 4]);

impl From<[[u32; 4]; 4]> for DDSDealRepr {
    #[inline]
    fn from(value: [[u32; 4]; 4]) -> Self {
        Self(value)
    }
}

impl DDSDealRepr {
    #[inline]
    #[must_use]
    pub fn new() -> Self {
        Self([[0; 4]; 4])
    }

    #[inline]
    #[must_use]
    pub fn as_slice(self) -> [[u32; 4]; 4] {
        self.0
    }
}

/// This is how DDS represents a PBN deal:
/// ae array of 80 chars.
#[derive(Debug, RawDDSRef)]
pub struct DDSDealPBNRepr(
    /// All PBN deals are 80 chars strings
    #[raw]
    [c_char; 80],
);

/// Trait for compatibility with DDS. Encodings:
/// Trump: 0 Spades, 1 Hearts, 2 Diamonds, 3 Clubs
/// Hands: 0 North, 1 East, 2 South, 3 West
pub trait AsDDSDeal {
    fn as_dds_deal(&self) -> DDSDealRepr;
}

/// This helps us build a Deal. Rough edges right now, should be refactored or improved
/// at least.
pub struct DDSDealBuilder {
    /// Trump for the deal, `None` when not set
    trump: Option<DDSSuitEncoding>,
    /// Leader for the deal, `None` when not set
    first: Option<DDSHandEncoding>,
    /// Current tricks' suits for the deal, `None` when not set
    current_trick_suit: Option<DDSCurrTrickSuit>,
    /// Current tricks' ranks for the deal, `None` when not set
    current_trick_rank: Option<DDSCurrTrickRank>,
    /// Remainig cards in the deal, exluded the one in current_trick* for the deal, `None` when not set
    remain_cards: Option<DDSDealRepr>,
}

#[non_exhaustive]
#[derive(Debug)]
pub enum DDSDealConstructionError {
    DuplicatedCard(c_int, c_int),
    CurrentTrickRankNotSet,
    CurrentTrickSuitNotSet,
    DealNotLoaded,
    TrumpNotDeclared,
    FirstNotDeclared,
    FirstUnconvertable,
    TrumpUnconvertable,
    IncorrectSequence(SeqError),
}

impl From<SeqError> for DDSDealConstructionError {
    #[inline]
    fn from(value: SeqError) -> Self {
        Self::IncorrectSequence(value)
    }
}

impl Display for DDSDealConstructionError {
    #[inline]
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match *self {
            Self::CurrentTrickRankNotSet => write!(
                formatter,
                "current trick rank is not set while current trick suit is"
            ),
            Self::CurrentTrickSuitNotSet => write!(
                formatter,
                "current trick suit is not set while current trick rank is"
            ),
            Self::DuplicatedCard(suit, rank) => {
                let card = dds_card_tuple_to_string(suit, rank);
                write!(formatter, "duplicated card: {card}")
            }
            Self::DealNotLoaded => write!(formatter, "deal not loaded"),
            Self::FirstNotDeclared => write!(formatter, "leader not declared"),
            Self::TrumpNotDeclared => write!(formatter, "trump not declared"),
            Self::FirstUnconvertable => {
                write!(
                    formatter,
                    "first cannot be converted from the value you provided"
                )
            }
            Self::TrumpUnconvertable => {
                write!(
                    formatter,
                    "trump cannot be converted from the value you provided"
                )
            }
            Self::IncorrectSequence(error) => {
                write!(formatter, "sequence has incorrect encoding:\n\t{error}")
            }
        }
    }
}

#[allow(clippy::missing_trait_methods, clippy::absolute_paths)]
impl std::error::Error for DDSDealConstructionError {}

impl Default for DDSDealBuilder {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl DDSDealBuilder {
    #[inline]
    #[must_use]
    pub fn new() -> Self {
        DDSDealBuilder {
            trump: None,
            first: None,
            current_trick_suit: None,
            current_trick_rank: None,
            remain_cards: None,
        }
    }

    #[inline]
    #[must_use]
    pub fn trump(mut self, trump: DDSSuitEncoding) -> Self {
        self.trump = Some(trump);
        self
    }

    #[inline]
    #[must_use]
    pub fn first(mut self, first: DDSHandEncoding) -> Self {
        self.first = Some(first);
        self
    }

    #[inline]
    #[must_use]
    pub fn remain_cards(mut self, remain_cards: DDSDealRepr) -> Self {
        self.remain_cards = Some(remain_cards);
        self
    }

    #[inline]
    #[must_use]
    pub fn current_trick_suit(mut self, current_trick_suit: DDSCurrTrickSuit) -> Self {
        self.current_trick_suit = Some(current_trick_suit);
        self
    }

    #[inline]
    #[must_use]
    pub fn current_trick_rank(mut self, current_trick_rank: DDSCurrTrickRank) -> Self {
        self.current_trick_rank = Some(current_trick_rank);
        self
    }

    #[allow(clippy::question_mark_used, clippy::as_conversions)]
    #[inline]
    /// Builds the `DDSDeal`.
    ///
    /// # Errors
    /// This method will return an error when one of the field was not supplied
    pub fn build(self) -> Result<DDSDeal, DDSDealConstructionError> {
        let remain_cards = self
            .remain_cards
            .ok_or(DDSDealConstructionError::DealNotLoaded)?;
        let trump = self
            .trump
            .ok_or(DDSDealConstructionError::TrumpNotDeclared)?;
        let first = self
            .first
            .ok_or(DDSDealConstructionError::FirstNotDeclared)?;
        let (current_trick_suit, current_trick_rank) =
            match (self.current_trick_suit, self.current_trick_rank) {
                (Some(suits), Some(ranks)) => Ok((suits, ranks)),
                (None, None) => Ok(Default::default()),
                (None, _) => Err(DDSDealConstructionError::CurrentTrickSuitNotSet),
                (_, None) => Err(DDSDealConstructionError::CurrentTrickRankNotSet),
            }?;
        Ok(DDSDeal {
            raw: deal {
                trump: trump as c_int,
                first: first as c_int,
                currentTrickSuit: *current_trick_suit.get_raw(),
                currentTrickRank: *current_trick_rank.get_raw(),
                remainCards: *remain_cards.get_raw(),
            },
        })
    }
}

/// A wrapper around the `deal` struct from DDS.
/// A `deal` is composed by a trump (represented with the [`DDSSuitEncoding`]),
/// the player on lead (representend with the [`DDSHandEncoding`]), the current
/// trick, represented as a pair of `[c_int;3]`, representing the current trick's card's
/// suit and rank and the remaining cards, representend with the [`DDSDealRepr`].
#[derive(RawDDSRef, RawDDSRefMut, Debug, AsRawDDS, Copy, Clone)]
pub struct DDSDeal {
    #[raw]
    /// The raw DDS `deal`
    raw: deal,
}

impl DDSDeal {
    pub fn new() -> Self {
        Self {
            raw: deal {
                trump: -1,
                first: -1,
                currentTrickSuit: [-1i32; 3],
                currentTrickRank: [-1i32; 3],
                remainCards: [[0u32; 4]; 4],
            },
        }
    }
}

/// A wrapper around DDS's [`dealPBN`].
/// See [`DDSDeal`] for reference on the fields.
#[derive(RawDDSRef, Debug)]
pub struct DDSDealPBN {
    #[raw]
    /// The raw DDS `dealPBN`
    raw: dealPBN,
}

#[allow(clippy::unreachable)]
/// Converts a tuple of ints to a `String` representing a card
fn dds_card_tuple_to_string(suit: c_int, rank: c_int) -> String {
    let rankstr = match rank {
        0b_100i32 => "2",
        0b_1000i32 => "3",
        0b_10000i32 => "4",
        0b_100000i32 => "5",
        0b_1000000i32 => "6",
        0b_10000000i32 => "7",
        0b_100000000i32 => "8",
        0b_1000000000i32 => "9",
        0b_10000000000i32 => "10",
        0b_100000000000i32 => "J",
        0b_1000000000000i32 => "Q",
        0b_10000000000000i32 => "K",
        0b_100000000000000i32 => "A",
        _ => unreachable!("sanity checks on rank not performed, i'm panicking"),
    };
    let suitstr = match suit {
        0i32 => "\u{2660}",
        1i32 => "\u{2665}",
        2i32 => "\u{25c6}",
        3i32 => "\u{2663}",
        _ => unreachable!("sanity checks on suit not performed, i'm panicking"),
    };
    let mut res = String::with_capacity(2);
    res.push_str(suitstr);
    res.push_str(rankstr);
    res
}

/// A wrapper around the [boards] struct from DDS.
/// Consists of a number of boards to be analyzed and
/// 5 arrays of length 200, representing
/// the deals, contracts, DDS `target`, `solution` and `mode` parameters
/// to be used in the analysis by DDS.
#[derive(RawDDSRef, RawDDSRefMut, Debug)]
pub struct Boards {
    #[raw]
    raw: boards,
}

impl Boards {
    #[inline]
    pub fn new<D: AsDDSDeal, C: AsDDSContract>(
        no_of_boards: i32,
        deals: &[D; MAXNOOFBOARDS],
        contracts: &[C; MAXNOOFBOARDS],
        target: &[Target; MAXNOOFBOARDS],
        solution: &[Solutions; MAXNOOFBOARDS],
        mode: &[Mode; MAXNOOFBOARDS],
    ) -> Self {
        Self {
            raw: boards::new(no_of_boards, deals, contracts, target, solution, mode),
        }
    }
}

impl boards {
    #[allow(clippy::unwrap_used)]
    /// Creates a new `boards` struct
    fn new<D: AsDDSDeal, C: AsDDSContract>(
        no_of_boards: i32,
        deals: &[D; MAXNOOFBOARDS],
        contracts: &[C; MAXNOOFBOARDS],
        target: &[Target; MAXNOOFBOARDS],
        solution: &[Solutions; MAXNOOFBOARDS],
        mode: &[Mode; MAXNOOFBOARDS],
    ) -> Self {
        let c_deals = deals
            .iter()
            .zip(contracts.iter())
            .map(|(deal, contract)| {
                let (trump, first) = contract.as_dds_contract();
                deal {
                    trump,
                    first,
                    currentTrickSuit: [0i32; 3],
                    currentTrickRank: [0i32; 3],
                    remainCards: deal.as_dds_deal().as_slice(),
                }
            })
            .collect::<Vec<deal>>()
            .try_into()
            // SAFETY: already now we can fit them
            .unwrap();
        boards {
            noOfBoards: no_of_boards,
            // SAFETY: Length if 200
            deals: c_deals,
            target: target.map(Into::into),
            solutions: solution.map(Into::into),
            mode: mode.map(Into::into),
        }
    }
}
