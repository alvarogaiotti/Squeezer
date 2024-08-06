// Copyright (C) 2024 Alvaro Gaiotti
// See end of file for license information

use squeezer_macros::RawDDSRef;

use crate::{
    bindings::MAXNOOFBOARDS,
    traits::{AsDDSContract, RawDDSRef},
    utils::{Mode, SeqError, Solutions, Target},
};
use core::{
    convert::{Into, TryFrom},
    ffi::{c_char, c_int},
    fmt::Display,
};

/// A wrapper around the `boards` struct from DDS.
/// Consists of a number of boards to be analyzed and
/// 5 arrays of length 200, representing
/// the deals, contracts, DDS [`Target`], [`Solutions`] and [`Mode`] parameters
/// to be used in the analysis by DDS.
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct Boards {
    pub no_of_boards: ::std::os::raw::c_int,
    pub deals: [DdsDeal; 200usize],
    pub target: [::std::os::raw::c_int; 200usize],
    pub solutions: [::std::os::raw::c_int; 200usize],
    pub mode: [::std::os::raw::c_int; 200usize],
}

#[allow(clippy::pedantic)]
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct BoardsPbn {
    pub no_of_boards: ::std::os::raw::c_int,
    pub deals: [DdsDealPbn; 200usize],
    pub target: [::std::os::raw::c_int; 200usize],
    pub solutions: [::std::os::raw::c_int; 200usize],
    pub mode: [::std::os::raw::c_int; 200usize],
}

/// A wrapper around the `deal` struct from DDS.
/// A `deal` is composed by a trump (represented with the [`DdsSuitEncoding`]),
/// the player on lead (representend with the [`DdsHandEncoding`]), the current
/// trick, represented as a pair of `[c_int;3]`, representing the current trick's card's
/// suit and rank and the remaining cards, representend with the [`DDSDealRepr`].
#[repr(C)]
#[derive(Debug, Copy, Clone, Default)]
pub struct DdsDeal {
    pub trump: ::std::os::raw::c_int,
    pub first: ::std::os::raw::c_int,
    pub current_trick_suit: [::std::os::raw::c_int; 3usize],
    pub current_trick_rank: [::std::os::raw::c_int; 3usize],
    pub remain_cards: [[::std::os::raw::c_uint; 4usize]; 4usize],
}
/// A wrapper around DDS's `dealPbn`.
/// See [`DdsDeal`] for reference on the fields.
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct DdsDealPbn {
    pub trump: ::std::os::raw::c_int,
    pub first: ::std::os::raw::c_int,
    pub current_trick_suit: [::std::os::raw::c_int; 3usize],
    pub current_trick_rank: [::std::os::raw::c_int; 3usize],
    pub remain_cards: [::std::os::raw::c_char; 80usize],
}

#[derive(Debug, RawDDSRef, Default)]
pub struct DDSCurrTrickSuit(#[raw] [c_int; 3]);

#[derive(Debug, RawDDSRef, Default)]
pub struct DDSCurrTrickRank(#[raw] [c_int; 3]);

#[allow(clippy::exhaustive_enums)]
/// How DDS encodes suits
pub enum DdsSuitEncoding {
    Spades = 0,
    Hearts = 1,
    Diamonds = 2,
    Clubs = 3,
    NoTrump = 4,
}

// See https://stackoverflow.com/questions/28028854/how-do-i-match-enum-values-with-an-integer
/// Macro for quick implementation of the [`TryFrom`] trait for a type
macro_rules! impl_tryfrom_dds_suit {
    ($($from:ty),*) => {
        $(impl core::convert::TryFrom<$from> for DdsSuitEncoding {
            type Error = DDSDealConstructionError;

            #[inline]
            fn try_from(value: $from) -> Result<Self, Self::Error> {
                match value {
            0 => Ok(Self::Spades),
            1 => Ok(Self::Hearts),
            2 => Ok(Self::Diamonds),
            3 => Ok(Self::Clubs),
            4 => Ok(Self::NoTrump),
            _ => Err(Self::Error::TrumpUnconvertable(value.try_into().unwrap_or(-1i32))),
               }
            }
        })*
    };
}

impl TryFrom<i32> for DdsSuitEncoding {
    type Error = DDSDealConstructionError;

    #[inline]
    fn try_from(value: i32) -> Result<Self, Self::Error> {
        match value {
            0i32 => Ok(Self::Spades),
            1i32 => Ok(Self::Hearts),
            2i32 => Ok(Self::Diamonds),
            3i32 => Ok(Self::Clubs),
            4i32 => Ok(Self::NoTrump),
            _ => Err(Self::Error::TrumpUnconvertable(value)),
        }
    }
}

impl_tryfrom_dds_suit!(u8, u16, u32, usize);
impl_tryfrom_dds_suit!(i8, i16, isize);

#[allow(clippy::exhaustive_enums)]
/// How DDS encodes seat.
#[derive(Debug, Default)]
pub enum DdsHandEncoding {
    #[default]
    North = 0,
    East = 1,
    South = 2,
    West = 3,
}

/// Mcro for implementing [`TryFrom`] from integer to [`DdsHandEncoding`]
macro_rules! impl_tryfrom_dds_hand {
    ($($from:ty),*) => {
        $(impl core::convert::TryFrom<$from> for DdsHandEncoding {
            type Error = DDSDealConstructionError;

            #[inline]
            fn try_from(value: $from) -> Result<Self, Self::Error> {
                match value {
                    0 => Ok(Self::North),
                    1 => Ok(Self::East),
                    2 => Ok(Self::South),
                    3 => Ok(Self::West),
                    _ => Err(Self::Error::FirstUnconvertable(value.try_into().unwrap_or(-1i32))),
                }
            }
        })*
    };
}

impl TryFrom<i32> for DdsHandEncoding {
    type Error = DDSDealConstructionError;

    #[inline]
    fn try_from(value: i32) -> Result<Self, Self::Error> {
        match value {
            0i32 => Ok(Self::North),
            1i32 => Ok(Self::East),
            2i32 => Ok(Self::South),
            3i32 => Ok(Self::West),
            _ => Err(Self::Error::FirstUnconvertable(value)),
        }
    }
}
impl_tryfrom_dds_hand!(u8, u16, u32, usize, i8, i16, isize);

/// This is how DDS represents a "binary deal":
/// a array of arrays of `u32`, basing the order on the [`DdsHandEncoding`]
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
/// an array of 80 chars.
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

/// This helps us build a [`DdsDeal`]. Rough edges right now, should be refactored or improved
/// at least.
pub struct DDSDealBuilder {
    /// Trump for the deal, `None` when not set
    trump: Option<DdsSuitEncoding>,
    /// Leader for the deal, `None` when not set
    first: Option<DdsHandEncoding>,
    /// Current tricks' suits for the deal, `None` when not set
    current_trick_suit: Option<DDSCurrTrickSuit>,
    /// Current tricks' ranks for the deal, `None` when not set
    current_trick_rank: Option<DDSCurrTrickRank>,
    /// Remainig cards in the deal, exluded the one in `current_trick` for the deal, `None` when not set
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
    FirstUnconvertable(i32),
    TrumpUnconvertable(i32),
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
            Self::FirstUnconvertable(value) => {
                write!(
                    formatter,
                    "first cannot be converted from the value you provided: {value}",
                )
            }
            Self::TrumpUnconvertable(value) => {
                write!(
                    formatter,
                    "trump cannot be converted from the value you provided: {value}",
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
    pub fn trump(mut self, trump: DdsSuitEncoding) -> Self {
        self.trump = Some(trump);
        self
    }

    #[inline]
    #[must_use]
    pub fn first(mut self, first: DdsHandEncoding) -> Self {
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
    /// Builds the [`DdsDeal`].
    ///
    /// # Errors
    /// This method will return an error when one of the field was not supplied
    pub fn build(self) -> Result<DdsDeal, DDSDealConstructionError> {
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
        Ok(DdsDeal {
            trump: trump as c_int,
            first: first as c_int,
            current_trick_suit: *current_trick_suit.get_raw(),
            current_trick_rank: *current_trick_rank.get_raw(),
            remain_cards: *remain_cards.get_raw(),
        })
    }
}

impl DdsDeal {
    #[inline]
    #[must_use]
    pub fn new() -> Self {
        Self {
            trump: -1,
            first: -1,
            current_trick_suit: [-1i32; 3],
            current_trick_rank: [-1i32; 3],
            remain_cards: [[0u32; 4]; 4],
        }
    }
}

#[allow(clippy::unreachable)]
/// Converts a tuple of ints to a [`String`] representing a card
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

impl Boards {
    #[allow(clippy::unwrap_used, clippy::missing_panics_doc)]
    /// Creates a new [`Boards`] struct
    /// Number of deals get capped at 200
    pub fn new<D: AsDDSDeal, C: AsDDSContract>(
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
                DdsDeal {
                    trump,
                    first,
                    current_trick_suit: [0i32; 3],
                    current_trick_rank: [0i32; 3],
                    remain_cards: deal.as_dds_deal().as_slice(),
                }
            })
            .collect::<Vec<DdsDeal>>()
            .try_into()
            // SAFETY: already now we can fit them
            .unwrap();
        Boards {
            no_of_boards,
            // SAFETY: Length if 200
            deals: c_deals,
            target: target.map(Into::into),
            solutions: solution.map(Into::into),
            mode: mode.map(Into::into),
        }
    }
}

#[allow(clippy::pedantic)]
#[allow(dead_code)]
#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn bindgen_test_layout_boards() {
        assert_eq!(
            ::std::mem::size_of::<Boards>(),
            21604usize,
            concat!("Size of: ", stringify!(boards))
        );
        assert_eq!(
            ::std::mem::align_of::<Boards>(),
            4usize,
            concat!("Alignment of ", stringify!(boards))
        );
        assert_eq!(
            unsafe { &(*(::std::ptr::null::<Boards>())).no_of_boards as *const _ as usize },
            0usize,
            concat!(
                "Offset of field: ",
                stringify!(boards),
                "::",
                stringify!(noOfBoards)
            )
        );
        assert_eq!(
            unsafe { &(*(::std::ptr::null::<Boards>())).deals as *const _ as usize },
            4usize,
            concat!(
                "Offset of field: ",
                stringify!(boards),
                "::",
                stringify!(deals)
            )
        );
        assert_eq!(
            unsafe { &(*(::std::ptr::null::<Boards>())).target as *const _ as usize },
            19204usize,
            concat!(
                "Offset of field: ",
                stringify!(boards),
                "::",
                stringify!(target)
            )
        );
        assert_eq!(
            unsafe { &(*(::std::ptr::null::<Boards>())).solutions as *const _ as usize },
            20004usize,
            concat!(
                "Offset of field: ",
                stringify!(boards),
                "::",
                stringify!(solutions)
            )
        );
        assert_eq!(
            unsafe { &(*(::std::ptr::null::<Boards>())).mode as *const _ as usize },
            20804usize,
            concat!(
                "Offset of field: ",
                stringify!(boards),
                "::",
                stringify!(mode)
            )
        );
    }

    #[test]
    fn bindgen_test_layout_boards_pbn() {
        assert_eq!(
            ::std::mem::size_of::<BoardsPbn>(),
            24804usize,
            concat!("Size of: ", stringify!(boardsPBN))
        );
        assert_eq!(
            ::std::mem::align_of::<BoardsPbn>(),
            4usize,
            concat!("Alignment of ", stringify!(boardsPBN))
        );
        assert_eq!(
            unsafe { &(*(::std::ptr::null::<BoardsPbn>())).no_of_boards as *const _ as usize },
            0usize,
            concat!(
                "Offset of field: ",
                stringify!(boardsPBN),
                "::",
                stringify!(noOfBoards)
            )
        );
        assert_eq!(
            unsafe { &(*(::std::ptr::null::<BoardsPbn>())).deals as *const _ as usize },
            4usize,
            concat!(
                "Offset of field: ",
                stringify!(boardsPBN),
                "::",
                stringify!(deals)
            )
        );
        assert_eq!(
            unsafe { &(*(::std::ptr::null::<BoardsPbn>())).target as *const _ as usize },
            22404usize,
            concat!(
                "Offset of field: ",
                stringify!(boardsPBN),
                "::",
                stringify!(target)
            )
        );
        assert_eq!(
            unsafe { &(*(::std::ptr::null::<BoardsPbn>())).solutions as *const _ as usize },
            23204usize,
            concat!(
                "Offset of field: ",
                stringify!(boardsPBN),
                "::",
                stringify!(solutions)
            )
        );
        assert_eq!(
            unsafe { &(*(::std::ptr::null::<BoardsPbn>())).mode as *const _ as usize },
            24004usize,
            concat!(
                "Offset of field: ",
                stringify!(boardsPBN),
                "::",
                stringify!(mode)
            )
        );
    }

    #[test]
    fn bindgen_test_layout_deal() {
        assert_eq!(
            ::std::mem::size_of::<DdsDeal>(),
            96usize,
            concat!("Size of: ", stringify!(deal))
        );
        assert_eq!(
            ::std::mem::align_of::<DdsDeal>(),
            4usize,
            concat!("Alignment of ", stringify!(deal))
        );
        assert_eq!(
            unsafe { &(*(::std::ptr::null::<DdsDeal>())).trump as *const _ as usize },
            0usize,
            concat!(
                "Offset of field: ",
                stringify!(deal),
                "::",
                stringify!(trump)
            )
        );
        assert_eq!(
            unsafe { &(*(::std::ptr::null::<DdsDeal>())).first as *const _ as usize },
            4usize,
            concat!(
                "Offset of field: ",
                stringify!(deal),
                "::",
                stringify!(first)
            )
        );
        assert_eq!(
            unsafe { &(*(::std::ptr::null::<DdsDeal>())).current_trick_suit as *const _ as usize },
            8usize,
            concat!(
                "Offset of field: ",
                stringify!(deal),
                "::",
                stringify!(currentTrickSuit)
            )
        );
        assert_eq!(
            unsafe { &(*(::std::ptr::null::<DdsDeal>())).current_trick_rank as *const _ as usize },
            20usize,
            concat!(
                "Offset of field: ",
                stringify!(deal),
                "::",
                stringify!(currentTrickRank)
            )
        );
        assert_eq!(
            unsafe { &(*(::std::ptr::null::<DdsDeal>())).remain_cards as *const _ as usize },
            32usize,
            concat!(
                "Offset of field: ",
                stringify!(deal),
                "::",
                stringify!(remainCards)
            )
        );
    }
    #[test]
    fn bindgen_test_layout_dds_deal_pbn() {
        assert_eq!(
            ::std::mem::size_of::<DdsDealPbn>(),
            112usize,
            concat!("Size of: ", stringify!(dealPBN))
        );
        assert_eq!(
            ::std::mem::align_of::<DdsDealPbn>(),
            4usize,
            concat!("Alignment of ", stringify!(dealPBN))
        );
        assert_eq!(
            unsafe { &(*(::std::ptr::null::<DdsDealPbn>())).trump as *const _ as usize },
            0usize,
            concat!(
                "Offset of field: ",
                stringify!(dealPBN),
                "::",
                stringify!(trump)
            )
        );
        assert_eq!(
            unsafe { &(*(::std::ptr::null::<DdsDealPbn>())).first as *const _ as usize },
            4usize,
            concat!(
                "Offset of field: ",
                stringify!(dealPBN),
                "::",
                stringify!(first)
            )
        );
        assert_eq!(
            unsafe {
                &(*(::std::ptr::null::<DdsDealPbn>())).current_trick_suit as *const _ as usize
            },
            8usize,
            concat!(
                "Offset of field: ",
                stringify!(dealPBN),
                "::",
                stringify!(currentTrickSuit)
            )
        );
        assert_eq!(
            unsafe {
                &(*(::std::ptr::null::<DdsDealPbn>())).current_trick_rank as *const _ as usize
            },
            20usize,
            concat!(
                "Offset of field: ",
                stringify!(dealPBN),
                "::",
                stringify!(currentTrickRank)
            )
        );
        assert_eq!(
            unsafe { &(*(::std::ptr::null::<DdsDealPbn>())).remain_cards as *const _ as usize },
            32usize,
            concat!(
                "Offset of field: ",
                stringify!(dealPBN),
                "::",
                stringify!(remainCards)
            )
        );
    }
}
