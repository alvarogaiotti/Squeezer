//! # `deal` — Bridge deal and board representations for the DDS FFI layer
//!
//! This module defines the data structures that map onto the C structs expected
//! by the [Double Dummy Solver (DDS)](https://github.com/dds-bridge/dds).
//! It is the primary translation layer between idiomatic Rust types and the
//! `#[repr(C)]` layouts that DDS requires.
//!
//! ## Key types at a glance
//!
//! | Type | Role |
//! |---|---|
//! [`DdsDeal`] | A single deal in binary (bitfield) form, ready for DDS |
//! [`DdsDealBuilder`] | The ergonomic, validating constructor for [`DdsDeal`] |
//! [`DdsSuit`] | Suit / trump encoding that DDS uses (0 = ♠ … 4 = NT) |
//! [`DdsRank`] | Sequential card rank (2 … 14, where 14 = Ace) |
//! [`DdsHandEncoding`] | Seat encoding (0 = North … 3 = West) |
//! [`DDSDealRepr`] | Newtype over the 4×4 `u32` bitfield array for remaining cards |
//!
//! ## Encoding conventions (inherited from DDS)
//!
//! * **Trump:** 0 = Spades, 1 = Hearts, 2 = Diamonds, 3 = Clubs, 4 = No Trump.
//! * **Hands / Seats:** 0 = North, 1 = East, 2 = South, 3 = West.
//! * **Remaining cards** are stored as a `[[u32; 4]; 4]` bitfield matrix.
//!   The outer index is the hand (North … West) and the inner index is the
//!   suit (Spades … Clubs). Each `u32` is a bitmask: bit *n* is set when
//!   the card of rank *n* is present (bit 2 = 2, bit 14 = Ace).
//! * **PBN deals** are encoded as an 80-byte C string following the standard
//!   PBN `Deal:` tag format.
//!
//! ## Usage sketch
//!
//! ```ignore
//! use dds::deal::{DdsDealBuilder, DdsSuit, DdsHandEncoding};
//!
//! let deal = DdsDealBuilder::new()
//!     .trump(DdsSuit::Spades)
//!     .first(DdsHandEncoding::North)
//!     .remain_cards(my_deal_repr)
//!     .build()?;
//! ```

// Copyright (C) 2024 Alvaro Gaiotti
// See end of file for license information

use squeezer_macros::RawDDSRef;

use crate::{
    traits::{AsDDSCard, AsDDSContract, RawDDSRef},
    utils::BuildSequenceError,
};
use core::{
    convert::TryFrom,
    ffi::{c_char, c_int},
};

// ─────────────────────────────────────────────────────────────────────────────
// Individual deal representations
// ─────────────────────────────────────────────────────────────────────────────

/// A single bridge deal in the binary (bitfield) format that DDS expects.
///
/// This is a direct `#[repr(C)]` mirror of DDS's `deal` struct.  **Fields
/// are private.**  The only way to construct a value of this type outside the
/// crate is through [`DdsDealBuilder`].  Use the accessor methods to read
/// fields back out.
///
/// ### Field layout (accessed via methods)
///
/// | Accessor | Meaning |
/// |---|---|
/// [`trump()`][`DdsDeal::trump`] | Trump suit as a [`DdsSuit`] discriminant (0 ♠ … 4 NT) |
/// [`first()`][`DdsDeal::first`] | Hand on lead as a [`DdsHandEncoding`] discriminant (0 N … 3 W) |
/// [`current_trick_suit()`][`DdsDeal::current_trick_suit`] | Suits of up to 3 cards already played in the current trick |
/// [`current_trick_rank()`][`DdsDeal::current_trick_rank`] | Ranks (bit-flags) of those same cards |
/// [`remain_cards()`][`DdsDeal::remain_cards`] | 4×4 bitfield matrix of cards still in hands (see module docs) |
#[repr(C)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Copy, Clone, Hash)]
pub struct DdsDeal {
    /// Trump suit.  Must be a valid [`DdsSuit`] discriminant (0–4).
    trump: ::std::os::raw::c_int,
    /// Hand on lead.  Must be a valid [`DdsHandEncoding`] discriminant (0–3).
    first: ::std::os::raw::c_int,
    /// Suits of the cards already played in the current trick, encoded as
    /// [`DdsSuit`] discriminants.  Unused slots should be `0`.
    current_trick_suit: [::std::os::raw::c_int; 3usize],
    /// Ranks of the cards already played in the current trick, encoded as
    /// **bit-flags** (bit 2 = rank 2, …, bit 14 = Ace).  Unused slots
    /// should be `0`.
    current_trick_rank: [::std::os::raw::c_int; 3usize],
    /// Remaining cards.  `remain_cards[hand][suit]` is a bitmask of the
    /// ranks held by `hand` in `suit`.
    remain_cards: [[::std::os::raw::c_uint; 4usize]; 4usize],
}

// ─────────────────────────────────────────────────────────────────────────────
// Current-trick helpers
// ─────────────────────────────────────────────────────────────────────────────

/// The suits of the (up to three) cards already played in the current trick.
///
/// Each element is a [`DdsSuit`] discriminant.  Slots that have not yet been
/// played should be `0`.  This type is only meaningful when you are
/// analysing a mid-trick position; for a position at the start of a trick
/// the default (all zeros) is correct.
///
/// Use [`DdsDealBuilder::current_trick`] to set this together with
/// [`DDSCurrTrickRank`] in one ergonomic call.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, RawDDSRef, Default, Copy, Hash, Clone)]
pub struct DDSCurrTrickSuit(#[raw] [i32; 3]);

/// The ranks of the (up to three) cards already played in the current trick.
///
/// Each element is a rank **bit-flag** (bit 2 = rank 2, …, bit 14 = Ace),
/// matching the encoding used in [`DdsDeal::current_trick_rank`].  Unused
/// slots should be `0`.
///
/// Use [`DdsDealBuilder::current_trick`] to set this together with
/// [`DDSCurrTrickSuit`] in one ergonomic call.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, RawDDSRef, Default, Copy, Hash, Clone)]
pub struct DDSCurrTrickRank(#[raw] [i32; 3]);

// ─────────────────────────────────────────────────────────────────────────────
// Suit and rank enums / newtypes
// ─────────────────────────────────────────────────────────────────────────────

/// The suit (or no-trump) encoding used by DDS.
///
/// The discriminant values are the integers DDS expects:
///
/// | Variant | Value | Symbol |
/// |---|---|---|
/// [`Spades`][`DdsSuit::Spades`] | 0 | ♠ |
/// [`Hearts`][`DdsSuit::Hearts`] | 1 | ♥ |
/// [`Diamonds`][`DdsSuit::Diamonds`] | 2 | ♦ |
/// [`Clubs`][`DdsSuit::Clubs`] | 3 | ♣ |
/// [`NoTrump`][`DdsSuit::NoTrump`] | 4 | NT |
///
/// ### Conversion
///
/// `TryFrom<integer>` is implemented for all common integer types.  An
/// out-of-range value produces a
/// [`ConstructDdsDealError::TrumpUnconvertable`] error.
#[allow(clippy::exhaustive_enums)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
#[repr(i32)]
pub enum DdsSuit {
    /// Spades (DDS value `0`).
    Spades = 0,
    /// Hearts (DDS value `1`).
    Hearts = 1,
    /// Diamonds (DDS value `2`).
    Diamonds = 2,
    /// Clubs (DDS value `3`).
    Clubs = 3,
    /// No Trump (DDS value `4`).  Used as the trump field when the contract
    /// is in no-trump.
    NoTrump = 4,
}

impl std::fmt::Display for DdsSuit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match *self {
                DdsSuit::Spades => "♠",
                DdsSuit::Hearts => "♥",
                DdsSuit::Diamonds => "♦",
                DdsSuit::Clubs => "♣",
                DdsSuit::NoTrump => "NT",
            }
        )
    }
}

/// A card rank in the sequential encoding used by this crate (2 … 14).
///
/// ### Dual-encoding note
///
/// DDS uses *bit-flag* ranks internally: bit *n* set means rank *n* is
/// present.  This type always holds the **sequential** rank (2 … 14).
/// Use [`as_bitflag`][`DdsRank::as_bitflag`] to convert to the DDS bit-flag
/// form, and [`from_bitflag`][`DdsRank::from_bitflag`] to convert back.
///
/// | Sequential | Bit-flag     | Card  |
/// |---|---|---|
/// | 2  | `1 << 2`  = 4     | 2     |
/// | …  | …                 | …     |
/// | 10 | `1 << 10` = 1024  | 10    |
/// | 11 | `1 << 11` = 2048  | Jack  |
/// | 12 | `1 << 12` = 4096  | Queen |
/// | 13 | `1 << 13` = 8192  | King  |
/// | 14 | `1 << 14` = 16384 | Ace   |
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct DdsRank(i32);

impl DdsRank {
    #[inline]
    #[must_use]
    /// Creates a new [`DdsRank`] from a sequential rank value.
    ///
    /// Returns `Some` only for values in `2..=14` (2 through Ace).
    /// Any other value returns `None`.
    ///
    /// # Examples
    ///
    /// ```
    /// use dds::deal::DdsRank;
    ///
    /// assert!(DdsRank::new(14).is_some()); // Ace
    /// assert!(DdsRank::new(1).is_none());  // invalid
    /// assert!(DdsRank::new(15).is_none()); // invalid
    /// ```
    pub fn new(rank: i32) -> Option<Self> {
        if (2..=14).contains(&rank) {
            Some(Self(rank))
        } else {
            None
        }
    }

    #[inline]
    #[must_use]
    /// Creates a [`DdsRank`] from a DDS bit-flag value.
    ///
    /// A valid bit-flag is exactly one bit set in the range bit 2 … bit 14
    /// (i.e. the value is a power of two between 4 and 16384 inclusive).
    /// Any other value returns `None`.
    ///
    /// # Examples
    ///
    /// ```
    /// use dds::deal::DdsRank;
    ///
    /// assert_eq!(DdsRank::from_bitflag(1 << 14), DdsRank::new(14)); // Ace
    /// assert_eq!(DdsRank::from_bitflag(1 << 2),  DdsRank::new(2));  // 2
    /// assert!(DdsRank::from_bitflag(0).is_none());                  // zero
    /// assert!(DdsRank::from_bitflag(3).is_none());                  // not a single bit
    /// ```
    pub fn from_bitflag(flag: i32) -> Option<Self> {
        // Must be exactly one bit set, and that bit must be in 2..=14.
        if flag <= 0 || (flag & (flag - 1)) != 0 {
            return None; // zero, negative, or more than one bit set
        }
        // A positive `i32` has its highest bit at index 30, so the signed
        // reinterpretation of the trailing-zero count is lossless.
        let bit_pos = flag.trailing_zeros().cast_signed();
        if (2..=14).contains(&bit_pos) {
            Some(Self(bit_pos))
        } else {
            None
        }
    }

    #[inline]
    #[must_use]
    /// Returns the DDS bit-flag representation of this rank.
    ///
    /// The bit-flag is `1 << sequential_rank`.  For example, the Ace (14)
    /// becomes `1 << 14 = 16384`.
    ///
    /// # Examples
    ///
    /// ```
    /// use dds::deal::DdsRank;
    ///
    /// assert_eq!(DdsRank::new(14).unwrap().as_bitflag(), 1 << 14);
    /// assert_eq!(DdsRank::new(2).unwrap().as_bitflag(),  1 << 2);
    /// ```
    pub fn as_bitflag(self) -> i32 {
        1 << self.0
    }
}

impl std::fmt::Display for DdsRank {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.0 {
            11 => write!(f, "J"),
            12 => write!(f, "Q"),
            13 => write!(f, "K"),
            14 => write!(f, "A"),
            n => write!(f, "{n}"),
        }
    }
}

// ─── TryFrom impls for DdsSuit ────────────────────────────────────────────────

// See https://stackoverflow.com/questions/28028854/how-do-i-match-enum-values-with-an-integer
/// Blanket `TryFrom<$from>` for [`DdsSuit`] for unsigned and smaller signed
/// integer types.  The error path converts the value to `i32` for the
/// diagnostic; if that conversion itself fails the sentinel `-1` is used.
macro_rules! impl_tryfrom_dds_suit {
    ($($from:ty),*) => {
        $(impl core::convert::TryFrom<$from> for DdsSuit {
            type Error = ConstructDdsDealError;
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

impl TryFrom<i32> for DdsSuit {
    type Error = ConstructDdsDealError;

    /// Converts an `i32` to a [`DdsSuit`].
    ///
    /// # Errors
    ///
    /// Returns [`ConstructDdsDealError::TrumpUnconvertable`] if `value` is
    /// not in `0..=4`.
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

// `i32` is handled by the manual impl directly above.  Do NOT add it to
// either invocation below — you will get a duplicate-impl compile error.
impl_tryfrom_dds_suit!(u8, u16, u32, usize);
impl_tryfrom_dds_suit!(i8, i16, isize);

// ─────────────────────────────────────────────────────────────────────────────
// Hand / seat encoding
// ─────────────────────────────────────────────────────────────────────────────

/// The seat (hand) encoding used by DDS.
///
/// | Variant | Value |
/// |---|---|
/// [`North`][`DdsHandEncoding::North`] | 0 |
/// [`East`][`DdsHandEncoding::East`] | 1 |
/// [`South`][`DdsHandEncoding::South`] | 2 |
/// [`West`][`DdsHandEncoding::West`] | 3 |
///
/// ### Conversion
///
/// `TryFrom<integer>` is implemented for all common integer types.  An
/// out-of-range value produces a
/// [`ConstructDdsDealError::SeatUnconvertable`] error.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[allow(clippy::exhaustive_enums)]
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum DdsHandEncoding {
    /// North (DDS value `0`).
    North = 0,
    /// East (DDS value `1`).
    East = 1,
    /// South (DDS value `2`).
    South = 2,
    /// West (DDS value `3`).
    West = 3,
}

/// Blanket `TryFrom<$from>` for [`DdsHandEncoding`].  Same sentinel strategy
/// as [`impl_tryfrom_dds_suit`].
macro_rules! impl_tryfrom_dds_hand_encoding {
    ($($from:ty),*) => {
        $(impl core::convert::TryFrom<$from> for DdsHandEncoding {
            type Error = ConstructDdsDealError;

            #[inline]
            fn try_from(value: $from) -> Result<Self, Self::Error> {
                match value {
                    0 => Ok(Self::North),
                    1 => Ok(Self::East),
                    2 => Ok(Self::South),
                    3 => Ok(Self::West),
                    _ => Err(Self::Error::SeatUnconvertable(value.try_into().unwrap_or(-1i32))),
                }
            }
        })*
    };
}

impl TryFrom<i32> for DdsHandEncoding {
    type Error = ConstructDdsDealError;

    /// Converts an `i32` to a [`DdsHandEncoding`].
    ///
    /// # Errors
    ///
    /// Returns [`ConstructDdsDealError::SeatUnconvertable`] if `value` is
    /// not in `0..=3`.
    #[inline]
    fn try_from(value: i32) -> Result<Self, Self::Error> {
        match value {
            0i32 => Ok(Self::North),
            1i32 => Ok(Self::East),
            2i32 => Ok(Self::South),
            3i32 => Ok(Self::West),
            _ => Err(Self::Error::SeatUnconvertable(value)),
        }
    }
}

// `i32` is handled by the manual impl directly above.  Do NOT add it here.
impl_tryfrom_dds_hand_encoding!(u8, u16, u32, usize, i8, i16, isize);

// ─────────────────────────────────────────────────────────────────────────────
// Deal representation newtypes
// ─────────────────────────────────────────────────────────────────────────────

/// The 4 × 4 bitfield matrix that DDS uses to represent which cards remain
/// in each hand.
///
/// ### Layout
///
/// ```text
/// remain_cards[hand][suit]
///   hand: 0 = North, 1 = East, 2 = South, 3 = West  (see DdsHandEncoding)
///   suit: 0 = Spades, 1 = Hearts, 2 = Diamonds, 3 = Clubs  (see DdsSuit)
/// ```
///
/// Each `u32` is a bitmask: bit *n* is set when the card of rank *n* is
/// present in that hand/suit combination.  Ranks run from 2 (bit 2) to
/// Ace (bit 14); bits 0, 1, and 15–31 are unused and must be `0`.
///
/// ### Example
///
/// To indicate that North holds the Ace of Spades:
///
/// ```
/// let mut arr = [[0u32; 4]; 4];
/// arr[0][0] |= 1 << 14;   // North, Spades, Ace
/// let repr = dds::deal::DDSDealRepr::from(arr);
/// ```
#[derive(Debug, RawDDSRef, Default, Copy, Hash, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
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
    /// Creates a new, all-zero [`DDSDealRepr`] (no cards in any hand).
    pub fn new() -> Self {
        Self([[0; 4]; 4])
    }

    #[inline]
    #[must_use]
    /// Returns the underlying 4 × 4 array by value.
    ///
    /// If you need a reference to the array without copying, use
    /// [`RawDDSRef::get_raw`] instead.
    pub fn as_array(self) -> [[u32; 4]; 4] {
        self.0
    }
}

/// The 80-byte PBN string representation of a deal's remaining cards.
///
/// DDS expects a null-terminated C string in the standard PBN `Deal:` format.
/// The string is always exactly 80 bytes long (including padding / null
/// terminator).
#[derive(Debug, RawDDSRef)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct DDSDealPBNRepr(
    #[cfg_attr(feature = "serde", serde(with = "serde_big_array::BigArray"))]
    #[raw]
    [c_char; 80],
);

// ─────────────────────────────────────────────────────────────────────────────
// Trait: AsDDSDeal
// ─────────────────────────────────────────────────────────────────────────────

/// Trait for types that can produce a [`DDSDealRepr`] (the binary card
/// matrix).
///
/// Implement this for your own deal representation so that it can be converted
/// into a [`DdsDeal`] through [`DdsDealBuilder::remain_cards`].
///
/// ### Encoding contract
///
/// The returned [`DDSDealRepr`] must use the DDS conventions:
/// * Outer index = hand (0 N, 1 E, 2 S, 3 W).
/// * Inner index = suit (0 ♠, 1 ♥, 2 ♦, 3 ♣).
/// * Each `u32` is a rank bitmask (bit *n* = rank *n*).
pub trait AsDDSDeal {
    /// Converts `self` into the DDS binary deal representation.
    fn as_dds_deal(&self) -> DDSDealRepr;
}

// ─────────────────────────────────────────────────────────────────────────────
// DdsDealBuilder
// ─────────────────────────────────────────────────────────────────────────────

/// A step-by-step, validating builder for [`DdsDeal`].
///
/// All fields start as `None` and must be set before [`build`][`DdsDealBuilder::build`]
/// is called.  The only optional group is the current trick: if *neither*
/// suit nor rank is set, the trick defaults to empty (all zeros).  If only
/// one of the two is set, [`build`][`DdsDealBuilder::build`] returns an error.
///
/// ### Typical usage
///
/// ```ignore
/// use dds::deal::*;
///
/// let deal = DdsDealBuilder::new()
///     .trump(DdsSuit::Hearts)
///     .first(DdsHandEncoding::South)
///     .remain_cards(my_repr)
///     .build()?;
/// ```
///
/// If you already have a contract object that implements [`AsDDSContract`],
/// you can set trump and leader in one call:
///
/// ```ignore
/// let deal = DdsDealBuilder::new()
///     .contract(&my_contract)   // sets trump + first
///     .remain_cards(my_repr)
///     .build()?;
/// ```
pub struct DdsDealBuilder {
    /// Trump for the deal, `None` when not yet set.
    trump: Option<DdsSuit>,
    /// Hand on lead, `None` when not yet set.
    first: Option<DdsHandEncoding>,
    /// Suits of cards already played in the current trick, `None` when not set.
    current_trick_suit: Option<DDSCurrTrickSuit>,
    /// Ranks of cards already played in the current trick, `None` when not set.
    current_trick_rank: Option<DDSCurrTrickRank>,
    /// Remaining cards (the bitfield matrix), `None` when not yet set.
    remain_cards: Option<DDSDealRepr>,
}

// ─────────────────────────────────────────────────────────────────────────────
// Errors
// ─────────────────────────────────────────────────────────────────────────────

/// Errors that can occur while constructing a [`DdsDeal`] via
/// [`DdsDealBuilder`] or while converting integer values into DDS enums.
///
/// | Variant | When it happens |
/// |---|---|
/// [`DuplicatedCard`][`ConstructDdsDealError::DuplicatedCard`] | The same card appears more than once in a deal |
/// [`CurrentTrickRankNotSet`][`ConstructDdsDealError::CurrentTrickRankNotSet`] | Trick suits were set but ranks were not |
/// [`CurrentTrickSuitNotSet`][`ConstructDdsDealError::CurrentTrickSuitNotSet`] | Trick ranks were set but suits were not |
/// [`CardsNotProvided`][`ConstructDdsDealError::CardsNotProvided`] | `remain_cards` was never set on the builder |
/// [`TrumpNotDeclared`][`ConstructDdsDealError::TrumpNotDeclared`] | `trump` was never set |
/// [`FirstNotDeclared`][`ConstructDdsDealError::FirstNotDeclared`] | `first` (leader) was never set |
/// [`SeatUnconvertable`][`ConstructDdsDealError::SeatUnconvertable`] | An integer could not be converted to a [`DdsHandEncoding`] |
/// [`TrumpUnconvertable`][`ConstructDdsDealError::TrumpUnconvertable`] | An integer could not be converted to a [`DdsSuit`] |
/// [`IncorrectSequence`][`ConstructDdsDealError::IncorrectSequence`] | A card sequence failed validation (wraps [`BuildSequenceError`]) |
#[non_exhaustive]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Copy, Clone, Hash, thiserror::Error)]
pub enum ConstructDdsDealError {
    /// A card identified by `(suit, rank)` bit-flags appears more than once
    /// in the deal.  The two `c_int` values are the suit index and the rank
    /// bit-flag respectively.
    ///
    /// `format_bitflag_card` never panics; it returns a diagnostic fallback
    /// string for values it cannot decode.
    #[error("duplicated card: {}", format_bitflag_card(*.0, *.1))]
    DuplicatedCard(c_int, c_int),
    /// The builder has `current_trick_suit` set but `current_trick_rank` is
    /// missing (or vice-versa — see [`CurrentTrickSuitNotSet`][`ConstructDdsDealError::CurrentTrickSuitNotSet`]).
    #[error("current trick rank is not set while current trick suit is")]
    CurrentTrickRankNotSet,
    /// The builder has `current_trick_rank` set but `current_trick_suit` is
    /// missing.
    #[error("current trick suit is not set while current trick rank is")]
    CurrentTrickSuitNotSet,
    /// [`DdsDealBuilder::remain_cards`] was never called before
    /// [`DdsDealBuilder::build`].
    #[error("deal not loaded")]
    CardsNotProvided,
    /// [`DdsDealBuilder::trump`] (or a contract that sets trump) was never
    /// called before [`DdsDealBuilder::build`].
    #[error("trump not declared")]
    TrumpNotDeclared,
    /// [`DdsDealBuilder::first`] (or a contract that sets the leader) was
    /// never called before [`DdsDealBuilder::build`].
    #[error("leader not declared")]
    FirstNotDeclared,
    /// An integer value could not be mapped to any [`DdsHandEncoding`]
    /// variant.  The inner `i32` is the offending value.
    #[error("seat cannot be converted from the value you provided: {0}")]
    SeatUnconvertable(i32),
    /// An integer value could not be mapped to any [`DdsSuit`] variant.
    /// The inner `i32` is the offending value.
    #[error("trump cannot be converted from the value you provided: {0}")]
    TrumpUnconvertable(i32),
    /// A card sequence (e.g. a trick) failed structural validation.
    /// Wraps the underlying [`BuildSequenceError`].
    #[error("sequence has incorrect encoding:\n\t{0}")]
    IncorrectSequence(#[from] BuildSequenceError),
}

// ─────────────────────────────────────────────────────────────────────────────
// Internal helpers
// ─────────────────────────────────────────────────────────────────────────────

/// Formats a `(suit_index, rank_bitflag)` pair into a human-readable card
/// string (e.g. `"♠A"`).
///
/// This function **never panics**.  If either value cannot be decoded it
/// returns a diagnostic string like `"?suit=99,rank=7?"` so that error
/// messages remain useful without risking an unwind.
fn format_bitflag_card(suit: c_int, rank: c_int) -> String {
    let suit_str = match suit {
        0 => "♠",
        1 => "♥",
        2 => "♦",
        3 => "♣",
        _ => return format!("?suit={suit},rank={rank}?"),
    };

    // Decode rank bit-flag via DdsRank so the logic lives in one place.
    match DdsRank::from_bitflag(rank) {
        Some(r) => format!("{suit_str}{r}"),
        None => format!("?suit={suit},rank={rank}?"),
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Builder impl
// ─────────────────────────────────────────────────────────────────────────────

impl Default for DdsDealBuilder {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl DdsDealBuilder {
    #[inline]
    #[must_use]
    /// Creates a new, empty [`DdsDealBuilder`].  All fields default to
    /// `None` and must be populated before [`build`][`DdsDealBuilder::build`] is called.
    pub fn new() -> Self {
        DdsDealBuilder {
            trump: None,
            first: None,
            current_trick_suit: None,
            current_trick_rank: None,
            remain_cards: None,
        }
    }

    #[inline]
    #[must_use]
    /// Sets both the trump suit and the opening leader from a contract
    /// object.
    ///
    /// This is a convenience shortcut for calling [`trump`][`DdsDealBuilder::trump`] and
    /// [`first`][`DdsDealBuilder::first`] separately.  The contract's
    /// [`AsDDSContract::as_dds_contract`] method is called to extract both
    /// values.
    ///
    /// If you call this *and* also call [`trump`][`DdsDealBuilder::trump`] or [`first`][`DdsDealBuilder::first`]
    /// afterwards, the later call wins.
    pub fn contract<T: AsDDSContract>(mut self, contract: &T) -> Self {
        let (trump, leader) = contract.as_dds_contract();
        self.trump = Some(trump);
        self.first = Some(leader);
        self
    }

    #[inline]
    #[must_use]
    /// Sets the trump suit for the deal.
    ///
    /// Use [`DdsSuit::NoTrump`] for a no-trump contract.
    pub fn trump(mut self, trump: DdsSuit) -> Self {
        self.trump = Some(trump);
        self
    }

    #[inline]
    #[must_use]
    /// Sets the hand that leads the first trick (or the current trick, if
    /// the position is mid-trick).
    pub fn first(mut self, first: DdsHandEncoding) -> Self {
        self.first = Some(first);
        self
    }

    #[inline]
    #[must_use]
    /// Sets the remaining cards for all four hands.
    ///
    /// This is the 4 × 4 bitfield matrix; see [`DDSDealRepr`] for the
    /// encoding details.
    pub fn remain_cards(mut self, remain_cards: DDSDealRepr) -> Self {
        self.remain_cards = Some(remain_cards);
        self
    }

    #[inline]
    #[must_use]
    /// Sets the cards already played in the current trick.
    ///
    /// Pass an array of three `Option<T>` values, where `T: AsDDSCard`.
    /// `None` entries represent empty trick slots (cards not yet played).
    /// For a position at the *start* of a trick, pass `[None, None, None]`
    /// or simply omit this call entirely (the default is an empty trick).
    pub fn current_trick<T: AsDDSCard>(mut self, current_trick: [Option<T>; 3]) -> Self {
        // Unroll the fixed-size array directly — no heap allocation, no
        // unwrap on a Vec→array conversion needed.
        let mut ranks = [0i32; 3];
        let mut suits = [0i32; 3];
        for (i, card) in current_trick.into_iter().enumerate() {
            if let Some(card) = card {
                let (rank, suit) = card.as_card();
                ranks[i] = rank.0;
                suits[i] = suit as i32;
            }
        }
        self.current_trick_rank = Some(DDSCurrTrickRank(ranks));
        self.current_trick_suit = Some(DDSCurrTrickSuit(suits));
        self
    }

    #[allow(clippy::question_mark_used, clippy::as_conversions)]
    #[inline]
    /// Consumes the builder and produces a [`DdsDeal`].
    ///
    /// ### Required fields
    ///
    /// * `trump` (via [`trump`][`DdsDealBuilder::trump`] or [`contract`][`DdsDealBuilder::contract`])
    /// * `first` (via [`first`][`DdsDealBuilder::first`] or [`contract`][`DdsDealBuilder::contract`])
    /// * `remain_cards` (via [`remain_cards`][`DdsDealBuilder::remain_cards`])
    ///
    /// ### Optional fields
    ///
    /// * Current trick — defaults to an empty trick (all zeros) if neither
    ///   [`current_trick`][`DdsDealBuilder::current_trick`] suits nor ranks were set.
    ///
    /// # Errors
    ///
    /// | Error | Cause |
    /// |---|---|
    /// [`ConstructDdsDealError::CardsNotProvided`] | `remain_cards` was not set |
    /// [`ConstructDdsDealError::TrumpNotDeclared`] | `trump` was not set |
    /// [`ConstructDdsDealError::FirstNotDeclared`] | `first` was not set |
    /// [`ConstructDdsDealError::CurrentTrickSuitNotSet`] | Ranks set but suits missing |
    /// [`ConstructDdsDealError::CurrentTrickRankNotSet`] | Suits set but ranks missing |
    pub fn build(self) -> Result<DdsDeal, ConstructDdsDealError> {
        let remain_cards = self
            .remain_cards
            .ok_or(ConstructDdsDealError::CardsNotProvided)?;
        let trump = self.trump.ok_or(ConstructDdsDealError::TrumpNotDeclared)?;
        let first = self.first.ok_or(ConstructDdsDealError::FirstNotDeclared)?;
        let (current_trick_suit, current_trick_rank) =
            match (self.current_trick_suit, self.current_trick_rank) {
                (Some(suits), Some(ranks)) => Ok((suits, ranks)),
                (None, None) => Ok(Default::default()),
                (None, _) => Err(ConstructDdsDealError::CurrentTrickSuitNotSet),
                (_, None) => Err(ConstructDdsDealError::CurrentTrickRankNotSet),
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

// ─────────────────────────────────────────────────────────────────────────────
// DdsDeal – inherent impls (accessors + crate-internal constructor)
// ─────────────────────────────────────────────────────────────────────────────

impl DdsDeal {
    #[must_use]
    /// Creates a "blank" [`DdsDeal`] filled with sentinel values.
    ///
    /// * `trump` and `first` are set to `-1` (invalid / unset).
    /// * Current trick arrays are filled with `-1`.
    /// * `remain_cards` is all zeros (no cards).
    ///
    /// This is intended as a placeholder for buffer slots that will be
    /// overwritten before being sent to DDS. External code should always use [`DdsDealBuilder`],
    /// unless you want to pre-allocate an empty deal array without using [`std::mem::MaybeUninit`].
    pub fn sentinel() -> Self {
        Self {
            trump: -1,
            first: -1,
            current_trick_suit: [-1i32; 3],
            current_trick_rank: [-1i32; 3],
            remain_cards: [[0u32; 4]; 4],
        }
    }

    #[inline]
    #[must_use]
    /// Returns the trump suit as a raw `c_int` discriminant.
    ///
    /// The value is a [`DdsSuit`] discriminant (0 ♠ … 4 NT).  Use
    /// `DdsSuit::try_from(deal.trump())` to convert to the typed enum.
    pub fn trump(&self) -> c_int {
        self.trump
    }

    #[inline]
    #[must_use]
    /// Returns the hand on lead as a raw `c_int` discriminant.
    ///
    /// The value is a [`DdsHandEncoding`] discriminant (0 N … 3 W).  Use
    /// `DdsHandEncoding::try_from(deal.first())` to convert to the typed enum.
    pub fn first(&self) -> c_int {
        self.first
    }

    #[inline]
    #[must_use]
    /// Returns the suits of the cards already played in the current trick.
    ///
    /// Each element is a [`DdsSuit`] discriminant; unused slots are `0`.
    pub fn current_trick_suit(&self) -> [c_int; 3] {
        self.current_trick_suit
    }

    #[inline]
    #[must_use]
    /// Returns the ranks of the cards already played in the current trick.
    ///
    /// Each element is a rank **bit-flag** (bit *n* = rank *n*); unused
    /// slots are `0`.  Use [`DdsRank::from_bitflag`] to decode individual
    /// values.
    pub fn current_trick_rank(&self) -> [c_int; 3] {
        self.current_trick_rank
    }

    #[inline]
    #[must_use]
    /// Returns the remaining-cards bitfield matrix.
    ///
    /// `remain_cards[hand][suit]` is a bitmask of ranks present.  See the
    /// module-level docs for the full encoding.
    pub fn remain_cards(&self) -> [[::std::os::raw::c_uint; 4]; 4] {
        self.remain_cards
    }
}

#[allow(clippy::pedantic, deref_nullptr)]
#[allow(dead_code)]
#[cfg(test)]
mod test {
    use super::*;
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
            unsafe {
                let uninit = ::std::mem::MaybeUninit::<DdsDeal>::uninit();
                let ptr = uninit.as_ptr();
                ::std::ptr::addr_of!((*ptr).trump) as usize - ptr as usize
            },
            0usize,
            concat!(
                "Offset of field: ",
                stringify!(deal),
                "::",
                stringify!(trump)
            )
        );
        assert_eq!(
            unsafe {
                let uninit = ::std::mem::MaybeUninit::<DdsDeal>::uninit();
                let ptr = uninit.as_ptr();
                ::std::ptr::addr_of!((*ptr).first) as usize - ptr as usize
            },
            4usize,
            concat!(
                "Offset of field: ",
                stringify!(deal),
                "::",
                stringify!(first)
            )
        );
        assert_eq!(
            unsafe {
                let uninit = ::std::mem::MaybeUninit::<DdsDeal>::uninit();
                let ptr = uninit.as_ptr();
                ::std::ptr::addr_of!((*ptr).current_trick_suit) as usize - ptr as usize
            },
            8usize,
            concat!(
                "Offset of field: ",
                stringify!(deal),
                "::",
                stringify!(currentTrickSuit)
            )
        );
        assert_eq!(
            unsafe {
                let uninit = ::std::mem::MaybeUninit::<DdsDeal>::uninit();
                let ptr = uninit.as_ptr();
                ::std::ptr::addr_of!((*ptr).current_trick_rank) as usize - ptr as usize
            },
            20usize,
            concat!(
                "Offset of field: ",
                stringify!(deal),
                "::",
                stringify!(currentTrickRank)
            )
        );
        assert_eq!(
            unsafe {
                let uninit = ::std::mem::MaybeUninit::<DdsDeal>::uninit();
                let ptr = uninit.as_ptr();
                ::std::ptr::addr_of!((*ptr).remain_cards) as usize - ptr as usize
            },
            32usize,
            concat!(
                "Offset of field: ",
                stringify!(deal),
                "::",
                stringify!(remainCards)
            )
        );
    }
}
