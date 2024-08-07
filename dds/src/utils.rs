// Copyright (C) 2024 Alvaro Gaiotti
// See end of file for license information

use crate::{
    ddserror::DDSError,
    deal::{AsDDSDeal, DDSDealBuilder, DDSDealConstructionError, DdsDeal},
    doubledummy::MultiThreadDoubleDummySolver,
    solver::BridgeSolver,
    traits::{AsDDSContract, ContractScorer, IntoRawDDS},
};
use core::{ffi::c_int, fmt::Display, num::NonZeroI32};
use squeezer_macros::IntoRawDDS;

/// The length of a sequence of suits or ranks
pub const SEQUENCE_LENGTH: usize = 52;

#[allow(clippy::exhaustive_enums)]
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
#[derive(Debug, Clone, Copy, Default)]
pub enum Target {
    #[default]
    MaxTricks,
    LegalNoScore,
    Goal(NonZeroI32),
}

impl From<Target> for c_int {
    #[inline]
    fn from(value: Target) -> Self {
        match value {
            Target::MaxTricks => -1i32,
            Target::LegalNoScore => 0i32,
            Target::Goal(goal) => c_int::max(13i32, goal.into()),
        }
    }
}

#[allow(clippy::exhaustive_enums)]
#[derive(Debug, Default, Clone, Copy)]
pub enum Solutions {
    #[default]
    Best,
    AllOptimal,
    AllLegal,
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
#[derive(Debug, Default, Clone, Copy)]
pub enum Mode {
    #[default]
    Auto,
    AutoSearchAlways,
    Always,
}

impl From<Mode> for c_int {
    #[inline]
    fn from(value: Mode) -> Self {
        match value {
            Mode::Auto => 0i32,
            Mode::AutoSearchAlways => 1i32,
            Mode::Always => 2i32,
        }
    }
}

#[allow(clippy::exhaustive_enums)]
pub enum Side {
    NS = 0,
    EW = 1,
}

#[allow(clippy::exhaustive_enums)]
#[derive(Debug, Clone, Copy)]
pub enum SeqError {
    SequenceTooLong,
    SequenceTooShort,
    SequenceNotValid,
}

impl std::error::Error for SeqError {}

impl Display for SeqError {
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
            type Error = SeqError;

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
                    return Err(SeqError::SequenceNotValid);
                }

                if length == 0 {
                    Err(SeqError::SequenceTooShort)
                } else if length > SEQUENCE_LENGTH {
                    return Err(SeqError::SequenceTooLong);
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

#[derive(IntoRawDDS, Debug, Clone)]
/// A `SuitSeq` is a sequence of cards' suit.
/// It's the sequence of suits used in [`PlayTraceBin`](crate::PlayTraceBin).
/// The suit is represented with the standard suit enconding used
/// throughout the codebase, which is [`DdsSuitEncoding`](super::DdsSuitEncoding).
/// - ♠️ => 0
/// - ♥️ => 1
/// - ♦️ => 2
/// - ♣️ => 3
/// - NT => 4
///
pub struct SuitSeq {
    #[raw]
    /// The sequence of the suit of the cards played
    sequence: [c_int; SEQUENCE_LENGTH],
    /// The real length of the suit sequence
    length: c_int,
}

impl<const N: usize> TryFrom<[c_int; N]> for SuitSeq {
    type Error = SeqError;

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
            return Err(SeqError::SequenceNotValid);
        }

        if length == 0 {
            Err(SeqError::SequenceTooShort)
        } else if length > SEQUENCE_LENGTH {
            return Err(SeqError::SequenceTooLong);
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
    type Error = SeqError;

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
            return Err(SeqError::SequenceNotValid);
        }

        if length == 0 {
            Err(SeqError::SequenceTooShort)
        } else if length > SEQUENCE_LENGTH {
            return Err(SeqError::SequenceTooLong);
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

#[derive(Debug, Clone, IntoRawDDS)]
/// A `RankSeq` is a sequence of cards' rank.
/// It's the sequence of ranks used in [`PlayTraceBin`](crate::PlayTraceBin).
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
    sequence: [c_int; SEQUENCE_LENGTH],
    /// The real length of the suit sequence
    length: c_int,
}

impl_tryfrom_array_for_sequence! {usize,u8,u16,u32,u64 ; RankSeq}
impl_tryfrom_array_for_sequence! {isize,i8,i16,i64 ; RankSeq}

impl TryFrom<&[c_int]> for RankSeq {
    type Error = SeqError;

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
            return Err(SeqError::SequenceNotValid);
        }

        if length == 0 {
            Err(SeqError::SequenceTooShort)
        } else if length > SEQUENCE_LENGTH {
            return Err(SeqError::SequenceTooLong);
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
    type Error = SeqError;

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
            return Err(SeqError::SequenceNotValid);
        }

        if length == 0 {
            Err(SeqError::SequenceTooShort)
        } else if length > SEQUENCE_LENGTH {
            return Err(SeqError::SequenceTooLong);
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
/// their encodings: [`DdsSuitEncoding`](super::DdsSuitEncoding) and
/// [`DdsHandEncoding`](super::DdsHandEncoding)
pub(crate) fn build_c_deal<C: AsDDSContract, D: AsDDSDeal>(
    contract_and_deal: (&C, &D),
) -> Result<DdsDeal, DDSDealConstructionError> {
    let (contract, deal) = contract_and_deal;
    let (trump, first) = contract.as_dds_contract();
    DDSDealBuilder::new()
        .trump(trump.try_into()?)
        .first(first.try_into()?)
        .remain_cards(deal.as_dds_deal())
        .build()
}

#[inline]
/// Some thing
///
/// # Errors
///
/// other
pub fn dd_score<D: AsDDSDeal, C: AsDDSContract + ContractScorer>(
    deal: &D,
    contract: &C,
) -> Result<i32, DDSError> {
    let solver = MultiThreadDoubleDummySolver::new();
    let tricks = solver.dd_tricks(deal, contract)?;
    Ok(contract.score(tricks))
}
