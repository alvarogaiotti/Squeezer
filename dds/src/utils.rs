// Copyright (C) 2024 Alvaro Gaiotti
// See end of file for license information

use crate::{
    ddserror::DdsError,
    deal::{AsDDSDeal, ConstructDdsDealError, DDSDealBuilder, DdsDeal},
    doubledummy::MultiThreadDoubleDummySolver,
    solver::BridgeSolver,
    traits::{AsDDSContract, ContractScorer, IntoRawDDS},
};
use core::{ffi::c_int, fmt::Display, num::NonZeroI32};
use squeezer_macros::IntoRawDDS;

/// The length of a sequence of suits or ranks
pub const SEQUENCE_LENGTH: usize = 52;

macro_rules! if_no_fault_return {
    ($result:ident, $ok:expr) => {
        #[allow(clippy::redundant_else)]
        if $result == crate::bindings::ddsffi::RETURN_NO_FAULT {
            return Ok($ok);
        } else {
            return Err($result.into());
        }
    };
}

pub(crate) use if_no_fault_return;

#[allow(clippy::exhaustive_enums)]
#[derive(Debug, Copy, Clone, Hash)]
pub enum ThreadIndex {
    Auto,
    NumThreads(NonZeroI32),
}

impl From<ThreadIndex> for c_int {
    #[inline]
    fn from(value: ThreadIndex) -> Self {
        match value {
            ThreadIndex::Auto => 0i32,
            ThreadIndex::NumThreads(thread_num) => thread_num.into(),
        }
    }
}

#[allow(clippy::exhaustive_enums)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Copy, Default)]
/// Target of the analysis of DDS.
/// DDS works by repeatedly calling its solving function with multiple targets until it fails (see its docs for more details):
/// this struct tells the solver what solving strategy should it use and works toghether with [`Solutions`] (see DDS docs for a
/// detailed breakdown):
/// - [`Target::MaxTricks`]: DDS will run until it finds the maximum number of tricks makable
/// - [`Target::LegalNoScore`]: DDS will just tell you what cards are legal to play
/// - [`Target::Goal`]: DDS will run to check if the target is reachable
pub enum Target {
    #[default]
    MaxTricks,
    LegalNoScore,
    Goal(u8),
}

impl From<Target> for c_int {
    #[inline]
    fn from(value: Target) -> Self {
        match value {
            Target::MaxTricks => -1i32,
            Target::LegalNoScore => 0i32,
            Target::Goal(goal) => i32::from(goal).clamp(1, 13),
        }
    }
}

#[allow(clippy::exhaustive_enums)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Default, Clone, Copy)]
/// This struct serves the purpose of telling DDS what solve mode should it employ when solving a deal.
/// Works toghether with [`Target`]: check DDS docs for the specifics way those two structs interact.
/// A general overview is as follows:
/// - [`Solutions::Best`]: DDS will return just the (or one of) the best solution(s).
/// - [`Solutions::AllOptimal`]: DDS will return all the best solutions.
/// - [`Solutions::AllLegal`]: DDS will return results for all the legal cards to play.
pub enum Solutions {
    #[default]
    Best = 1,
    AllOptimal = 2,
    AllLegal = 3,
}

impl From<Solutions> for c_int {
    #[inline]
    fn from(value: Solutions) -> Self {
        match value {
            Solutions::Best => 1i32,
            Solutions::AllOptimal => 2i32,
            Solutions::AllLegal => 3i32,
        }
    }
}

#[allow(clippy::exhaustive_enums)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Default, Clone, Copy)]
/// This struct defines the strategy used by DDS when it comes to
/// searching and reusing the transposition table.
/// - `AutoReuseLazySearch`: Automatic reuse of the TT, returns -2 as a card score if it is the only choice, whithout searching its score
/// - `AutoReuseAlwaysSearch`: Automatic reuse of the TT, searches even if a card is the only choice, returning its score
/// - `ForceReuseAlwaysSearch`: Force reuse of the TT and always searches. It is the programmer's responsibility to ensure that the deals
///     are similar and the TT is correct for the deal.
///
/// From the DDS docs:
/// > **Note**: mode no longer always has this effect internally in DDS. We think mode is no longer
/// > useful, and we may use it for something else in the future. If you think you need it, let us know!
/// > “Reuse” means “reuse the transposition table from the previous run with the same thread
/// > number”. For mode = 2 it is the responsibility of the programmer using the DLL to ensure that
/// > reusing the table is safe in the actual situation. Example: Deal is the same, except for
/// > deal.first. The trump suit is the same.
pub enum Mode {
    #[default]
    AutoReuseLazySearch,
    AutoReuseAlwaysSearch,
    ForceReuseAlwaysSearch,
}

impl From<Mode> for c_int {
    #[inline]
    fn from(value: Mode) -> Self {
        match value {
            Mode::AutoReuseLazySearch => 0i32,
            Mode::AutoReuseAlwaysSearch => 1i32,
            Mode::ForceReuseAlwaysSearch => 2i32,
        }
    }
}

#[allow(clippy::exhaustive_enums)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Copy)]
pub enum Side {
    NS = 0,
    EW = 1,
}

#[allow(clippy::exhaustive_enums)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Copy, Hash)]
pub enum BuildSequenceError {
    SequenceTooLong,
    SequenceTooShort,
    SequenceNotValid,
}

impl std::error::Error for BuildSequenceError {}

impl Display for BuildSequenceError {
    #[inline]
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let string = match *self {
            Self::SequenceTooLong => format!("sequence length is longer than {SEQUENCE_LENGTH}"),
            Self::SequenceTooShort => format!("sequence length is shorter than {}", 1),
            Self::SequenceNotValid => "sequence contains invalid values".to_owned(),
        };
        write!(f, "{string}")
    }
}

/// Macro for implementing `TryFrom` from different integer types to a sequence
macro_rules! impl_tryfrom_array_for_sequence {
    ($($from:ty),*; $to:ty) => {
        $(impl<const N: usize> TryFrom<[$from; N]> for $to {
            type Error = BuildSequenceError;

            /// Create a new `SuitSeq`, validating input.
            ///
            /// # Errors
            ///
            /// Errors when the sequence is too long or too short
            #[allow(clippy::unwrap_in_result)]
            #[inline]
            fn try_from(value: [$from; N]) -> Result<Self, Self::Error> {
                let length = N;

                if value.iter().any(|x| !(2..=14).contains(x)) {
                    return Err(BuildSequenceError::SequenceNotValid);
                }

                if length == 0 {
                    Err(BuildSequenceError::SequenceTooShort)
                } else if length > SEQUENCE_LENGTH {
                    return Err(BuildSequenceError::SequenceTooLong);
                } else {
                    let mut array: Vec<i32> = value
                        .into_iter()
                        // SAFETY: checked values above
                        .map(|num| i32::try_from(num).unwrap())
                        .collect();
                    array.resize(SEQUENCE_LENGTH, -1i32);
                    return Ok(Self {
                        // SAFETY: checks already performed above
                        sequence: array.try_into().unwrap(),
                        // SAFETY: checks already performed above
                        length: i32::try_from(length).unwrap(),
                    });
                }
            }
        })*
    };
}

#[derive(IntoRawDDS, Debug, Clone, Copy)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
/// A `SuitSeq` is a sequence of cards' suit.
/// It's the sequence of suits used in [`PlayTraceBin`](crate::analyseplay::PlayTraceBin).
/// The suit is represented with the standard suit enconding used
/// throughout the codebase, which is [`DdsSuit`](crate::deal::DdsSuit).
/// - ♠️ => 0
/// - ♥️ => 1
/// - ♦️ => 2
/// - ♣️ => 3
/// - NT => 4
///
pub struct SuitSeq {
    #[raw]
    /// The sequence of the suit of the cards played
    #[cfg_attr(feature = "serde", serde(with = "serde_big_array::BigArray"))]
    sequence: [c_int; SEQUENCE_LENGTH],
    /// The real length of the suit sequence
    length: c_int,
}

impl<const N: usize> TryFrom<[c_int; N]> for SuitSeq {
    type Error = BuildSequenceError;

    /// Create a new [`SuitSeq`], validating input.
    ///
    /// # Errors
    ///
    /// Errors when the sequence is too long or too short
    #[allow(clippy::unwrap_in_result)]
    #[inline]
    fn try_from(value: [c_int; N]) -> Result<Self, Self::Error> {
        let length = N;

        if value.iter().any(|&x| !(0i32..=3i32).contains(&x)) {
            return Err(BuildSequenceError::SequenceNotValid);
        }

        if length == 0 {
            Err(BuildSequenceError::SequenceTooShort)
        } else if length > SEQUENCE_LENGTH {
            return Err(BuildSequenceError::SequenceTooLong);
        } else {
            let mut sequence = [-1i32; SEQUENCE_LENGTH];
            sequence[0..length].copy_from_slice(&value[0..length]);
            return Ok(Self {
                sequence,
                // SAFETY: checks already performed above
                length: i32::try_from(length).unwrap(),
            });
        }
    }
}
impl TryFrom<&[c_int]> for SuitSeq {
    type Error = BuildSequenceError;

    /// Create a new [`SuitSeq`], validating input.
    ///
    /// # Errors
    ///
    /// Errors when the sequence is too long or too short
    #[allow(clippy::unwrap_in_result)]
    #[inline]
    fn try_from(value: &[c_int]) -> Result<Self, Self::Error> {
        let length = value.len();

        if value.iter().any(|&x| !(0i32..=3i32).contains(&x)) {
            return Err(BuildSequenceError::SequenceNotValid);
        }

        if length == 0 {
            Err(BuildSequenceError::SequenceTooShort)
        } else if length > SEQUENCE_LENGTH {
            return Err(BuildSequenceError::SequenceTooLong);
        } else {
            // let mut array = Vec::with_capacity(SEQUENCE_LENGTH);
            // array.extend_from_slice(value);
            // array.resize(SEQUENCE_LENGTH, -1i32);
            let mut sequence = [-1i32; SEQUENCE_LENGTH];
            sequence[..length].copy_from_slice(value);
            return Ok(Self {
                sequence,
                // SAFETY: checks already performed above
                length: i32::try_from(length).unwrap(),
            });
        }
    }
}

impl SuitSeq {
    #[inline]
    #[must_use]
    pub fn length(&self) -> i32 {
        self.length
    }
}

impl_tryfrom_array_for_sequence! {usize,u8,u16,u32,u64 ; SuitSeq}
impl_tryfrom_array_for_sequence! {isize,i8,i16,i64 ; SuitSeq}

#[derive(Debug, Copy, Clone, IntoRawDDS)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
/// A `RankSeq` is a sequence of cards' rank.
/// It's the sequence of ranks used in [`PlayTraceBin`](crate::analyseplay::PlayTraceBin).
/// Card are encoded with a incremental integer encoding, unlike in
/// other parts of the codebase:
/// - 2 => 2;
/// - 3 => 3;
/// - ...
/// - K => 13;
/// - A => 14;
pub struct RankSeq {
    #[raw]
    /// The sequence of the suit of the cards played
    #[cfg_attr(feature = "serde", serde(with = "serde_big_array::BigArray"))]
    sequence: [c_int; SEQUENCE_LENGTH],
    /// The real length of the suit sequence
    length: c_int,
}

impl_tryfrom_array_for_sequence! {usize,u8,u16,u32,u64 ; RankSeq}
impl_tryfrom_array_for_sequence! {isize,i8,i16,i64 ; RankSeq}

impl TryFrom<&[c_int]> for RankSeq {
    type Error = BuildSequenceError;

    /// Create a new [`RankSeq`], validating input.
    ///
    /// # Errors
    ///
    /// Errors when the sequence is too long or too short
    #[allow(clippy::unwrap_in_result)]
    #[inline]
    fn try_from(value: &[c_int]) -> Result<Self, Self::Error> {
        let length = value.len();

        if value.iter().any(|&x| !(2i32..=14i32).contains(&x)) {
            return Err(BuildSequenceError::SequenceNotValid);
        }

        if length == 0 {
            Err(BuildSequenceError::SequenceTooShort)
        } else if length > SEQUENCE_LENGTH {
            return Err(BuildSequenceError::SequenceTooLong);
        } else {
            let mut array = Vec::with_capacity(SEQUENCE_LENGTH);
            array.extend_from_slice(value);
            array.resize(SEQUENCE_LENGTH, -1i32);
            return Ok(Self {
                // SAFETY: checks already performed above
                sequence: array.try_into().unwrap(),
                // SAFETY: checks already performed above
                length: i32::try_from(length).unwrap(),
            });
        }
    }
}

impl<const N: usize> TryFrom<[c_int; N]> for RankSeq {
    type Error = BuildSequenceError;

    /// Create a new [`RankSeq`], validating input.
    ///
    /// # Errors
    ///
    /// Errors when the sequence is too long or too short
    #[allow(clippy::unwrap_in_result)]
    #[inline]
    fn try_from(value: [c_int; N]) -> Result<Self, Self::Error> {
        let length = N;

        if value.iter().any(|x| !(2i32..=14i32).contains(x)) {
            return Err(BuildSequenceError::SequenceNotValid);
        }

        if length == 0 {
            Err(BuildSequenceError::SequenceTooShort)
        } else if length > SEQUENCE_LENGTH {
            return Err(BuildSequenceError::SequenceTooLong);
        } else {
            let mut array = value.to_vec();
            array.resize(SEQUENCE_LENGTH, -1i32);
            return Ok(Self {
                // SAFETY: checks already performed above
                sequence: array.try_into().unwrap(),
                // SAFETY: checks already performed above
                length: i32::try_from(length).unwrap(),
            });
        }
    }
}

impl RankSeq {
    #[inline]
    #[must_use]
    pub fn length(&self) -> i32 {
        self.length
    }
}

/// Builds a [`DdsDeal`] from its components:
/// - Trump
/// - Leader
/// - Cards
///
/// # Errors
/// Will error if the trump or the player are not valid values following
/// their encodings: [`DdsSuitEncoding`](crate::deal::DdsSuitEncoding) and
/// [`super::utlils::DdsHandEncoding`](crate::deal::DdsHandEncoding)
#[inline]
pub(crate) fn build_c_deal<C: AsDDSContract, D: AsDDSDeal>(
    contract_and_deal: (&C, &D),
) -> Result<DdsDeal, ConstructDdsDealError> {
    let (contract, deal) = contract_and_deal;
    let (trump, first) = contract.as_dds_contract();
    DDSDealBuilder::new()
        .trump(trump)
        .first(first)
        .remain_cards(deal.to_dds_deal())
        .build()
}

/// Utility function for score of a contract based on double dummy solved hand.
///
/// # Errors
/// See [`crate::solver::BridgeSolver`] for errors
///
#[inline]
pub fn dd_score<D: AsDDSDeal, C: AsDDSContract + ContractScorer>(
    deal: &D,
    contract: &C,
) -> Result<i32, DdsError> {
    let solver = MultiThreadDoubleDummySolver::new();
    let tricks = solver.dd_tricks(deal, contract)?;
    Ok(contract.score(tricks))
}
