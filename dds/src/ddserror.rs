// Copyright (C) 2024 Alvaro Gaiotti
// See end of file for license information

/// Module that defines and provides all the error for this crate.
/// Re-exports some errors from the DDS library: see <https://github.com/dds-bridge/dds/blob/develop/doc/DLL-dds_x.pdf> for documentation.
use crate::bindings::ddsffi::{
    RETURN_CARD_COUNT, RETURN_CHUNK_SIZE, RETURN_DUPLICATE_CARDS, RETURN_FIRST_WRONG,
    RETURN_MODE_WRONG_HI, RETURN_MODE_WRONG_LO, RETURN_NO_SUIT, RETURN_PBN_FAULT,
    RETURN_PLAYED_CARD, RETURN_PLAY_FAULT, RETURN_SOLNS_WRONG_HI, RETURN_SOLNS_WRONG_LO,
    RETURN_SUIT_OR_RANK, RETURN_TARGET_TOO_HIGH, RETURN_TARGET_WRONG_HI, RETURN_TARGET_WRONG_LO,
    RETURN_THREAD_CREATE, RETURN_THREAD_INDEX, RETURN_THREAD_WAIT, RETURN_TOO_MANY_CARDS,
    RETURN_TOO_MANY_TABLES, RETURN_TRUMP_WRONG, RETURN_UNKNOWN_FAULT, RETURN_ZERO_CARDS,
};
use crate::deal::{ConstructDdsBoardError, ConstructDdsDealError};
use core::ffi::c_int;
use core::fmt;
use std::fmt::Debug;
/// Wrapper around the DDS errors.
/// See <https://github.com/dds-bridge/dds/blob/develop/doc/DLL-dds_x.pdf> for documentation.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Hash, Clone, Copy)]
pub struct DdsError {
    /// Represents what kind of error we got
    pub kind: DdsErrorKind,
}
impl Debug for DdsError {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        <Self as fmt::Display>::fmt(self, f)
    }
}

impl From<DdsErrorKind> for DdsError {
    #[inline]
    fn from(value: DdsErrorKind) -> Self {
        Self { kind: value }
    }
}
impl From<ConstructDdsBoardError> for DdsError {
    fn from(value: ConstructDdsBoardError) -> Self {
        Self::from(DdsErrorKind::from(value))
    }
}

impl From<c_int> for DdsError {
    #[inline]
    fn from(value: c_int) -> Self {
        assert_ne!(1i32, value, "if we fail the assertion we didn't check for the return result from dds, since a return result of 1 means success." );
        Self { kind: value.into() }
    }
}

impl From<ConstructDdsDealError> for DdsError {
    #[inline]
    fn from(value: ConstructDdsDealError) -> Self {
        Self::from(DdsErrorKind::from(value))
    }
}

impl fmt::Display for DdsError {
    #[inline]
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "something went wrong while solving boards:\n\t{}",
            self.kind
        )
    }
}

#[allow(clippy::missing_trait_methods, clippy::absolute_paths)]
impl std::error::Error for DdsError {}

/// Enum modelling all the way in which we can fail in the DDS related code.
/// The vast majority of the variants are provided by DDS directly and handled in Rust for a nicer interface.
/// Some are custom error regarding transformation from Rust types to the types that DDS uses.
/// See <https://github.com/dds-bridge/dds/blob/develop/doc/DLL-dds_x.pdf> for documentation.
#[allow(clippy::exhaustive_enums)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Copy, Clone, Hash)]
pub enum DdsErrorKind {
    UnknownFault,
    ZeroCards,
    TargetTooHigh,
    DuplicateCards,
    TargetWrongLo,
    TargetWrongHi,
    SolnsWrongLo,
    SolnsWrongHi,
    TooManyCards,
    SuitOrRank,
    PlayedCard,
    CardCount,
    ThreadIndex,
    ModeWrongLo,
    ModeWrongHi,
    TrumpWrong,
    FirstWrong,
    PlayFault,
    PbnFault,
    ThreadCreate,
    ThreadWait,
    NoSuit,
    TooManyTables,
    ChunkSize,
    // FIXME: This should be removed and the build_c_deal function should be made public, so people
    // can first try to build the deal and then pass the deal to DDS.
    // This will allow us to make errors more trasnparent to the user, providing them with the
    // ability to take corrective action in a more natural way, since the error are decoupled.
    UnbuildableDeal(ConstructDdsDealError),
    UnbuildableBoards(ConstructDdsBoardError),
}

impl Debug for DdsErrorKind {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        <Self as fmt::Display>::fmt(self, f)
    }
}

#[allow(clippy::unreachable)]
impl From<c_int> for DdsErrorKind {
    #[inline]
    fn from(value: c_int) -> Self {
        match value {
            RETURN_UNKNOWN_FAULT => Self::UnknownFault,
            RETURN_ZERO_CARDS => Self::ZeroCards,
            RETURN_TARGET_TOO_HIGH => Self::TargetTooHigh,
            RETURN_DUPLICATE_CARDS => Self::DuplicateCards,
            RETURN_TARGET_WRONG_LO => Self::TargetWrongLo,
            RETURN_TARGET_WRONG_HI => Self::TargetWrongHi,
            RETURN_SOLNS_WRONG_LO => Self::SolnsWrongLo,
            RETURN_SOLNS_WRONG_HI => Self::SolnsWrongHi,
            RETURN_TOO_MANY_CARDS => Self::TooManyCards,
            RETURN_SUIT_OR_RANK => Self::SuitOrRank,
            RETURN_PLAYED_CARD => Self::PlayedCard,
            RETURN_CARD_COUNT => Self::CardCount,
            RETURN_THREAD_INDEX => Self::ThreadIndex,
            RETURN_MODE_WRONG_LO => Self::ModeWrongLo,
            RETURN_MODE_WRONG_HI => Self::ModeWrongHi,
            RETURN_TRUMP_WRONG => Self::TrumpWrong,
            RETURN_FIRST_WRONG => Self::FirstWrong,
            RETURN_PLAY_FAULT => Self::PlayFault,
            RETURN_PBN_FAULT => Self::PbnFault,
            RETURN_THREAD_CREATE => Self::ThreadCreate,
            RETURN_THREAD_WAIT => Self::ThreadWait,
            RETURN_NO_SUIT => Self::NoSuit,
            RETURN_TOO_MANY_TABLES => Self::TooManyTables,
            RETURN_CHUNK_SIZE => Self::ChunkSize,
            // SAFETY:  return value from DDS cannot be different from its defined constants.
            _ => unreachable!(),
        }
    }
}

impl From<ConstructDdsDealError> for DdsErrorKind {
    #[inline]
    fn from(value: ConstructDdsDealError) -> Self {
        Self::UnbuildableDeal(value)
    }
}

impl From<ConstructDdsBoardError> for DdsErrorKind {
    #[inline]
    fn from(value: ConstructDdsBoardError) -> Self {
        Self::UnbuildableBoards(value)
    }
}
impl fmt::Display for DdsErrorKind {
    #[inline]
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Self::UnknownFault => write!(formatter, "fopen() failed or wrong number of boards"),
            Self::ZeroCards => write!(formatter, "zero cards"),
            Self::TargetTooHigh => write!(formatter, "target higher than number of tricks remaining"),
            Self::DuplicateCards => write!(formatter, "duplicated card"),
            Self::TargetWrongLo => write!(formatter, "target is less than -1"),
            Self::TargetWrongHi => write!(formatter, "target is higher than 13"),
            Self::SolnsWrongLo => write!(formatter, "solutions is less than 1"),
            Self::SolnsWrongHi => write!(formatter, "solutions is more than 3"),
            Self::TooManyCards => write!(formatter, "too many cards"),
            Self::SuitOrRank => write!(formatter, "currentTrickSuit or currentTrickRank have wrong data"),
            Self::PlayedCard => write!(formatter, "card already played"),
            Self::CardCount => write!(formatter, "wrong number of remaining cards for a hand"),
            Self::ThreadIndex => write!(formatter, "thread number is less than 0 or higher than the maximum permitted"),
            Self::ModeWrongLo => write!(formatter, "mode is less than 0"),
            Self::ModeWrongHi => write!(formatter, "mode is greater than 2"),
            Self::TrumpWrong => write!(formatter, "trump is not one of 0, 1, 2, 3 or 4"),
            Self::FirstWrong => write!(formatter, "first is not one of 0, 1, 2 or 3"),
            Self::PlayFault => write!(formatter, "less than 0 or more than 52 cards supplied, invalid suit or rank supplied or played card is not held by the right player"),
            Self::PbnFault => write!(formatter, "PBN string is malformed"),
            Self::ThreadCreate => write!(formatter, "thread created"),
            Self::ThreadWait => write!(formatter, "something went wrong while waiting for threads to complete"),
            Self::NoSuit => write!(formatter, "denomination filter vector has no entries"),
            Self::TooManyTables => write!(formatter, "too many tables requested"),
            Self::ChunkSize => write!(formatter, "chunk size is less than 1"),
            Self::UnbuildableDeal(ref inner) => write!(formatter, "unable to build DdsDeal: \n{inner}"),
            Self::UnbuildableBoards(ref inner) => write!(formatter, "unable to build Boards: \n{inner}"),
        }
    }
}
