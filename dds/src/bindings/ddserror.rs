use super::ddsffi::{
    RETURN_CARD_COUNT, RETURN_CHUNK_SIZE, RETURN_DUPLICATE_CARDS, RETURN_FIRST_WRONG,
    RETURN_MODE_WRONG_HI, RETURN_MODE_WRONG_LO, RETURN_NO_SUIT, RETURN_PBN_FAULT,
    RETURN_PLAYED_CARD, RETURN_PLAY_FAULT, RETURN_SOLNS_WRONG_HI, RETURN_SOLNS_WRONG_LO,
    RETURN_SUIT_OR_RANK, RETURN_TARGET_TOO_HIGH, RETURN_TARGET_WRONG_HI, RETURN_TARGET_WRONG_LO,
    RETURN_THREAD_CREATE, RETURN_THREAD_INDEX, RETURN_THREAD_WAIT, RETURN_TOO_MANY_CARDS,
    RETURN_TOO_MANY_TABLES, RETURN_TRUMP_WRONG, RETURN_UNKNOWN_FAULT, RETURN_ZERO_CARDS,
};

/// Wrapper around the DDS errors
#[derive(Debug)]
pub struct DDSError {
    /// Represents what kind of error we got
    kind: DDSErrorKind,
}

impl From<DDSErrorKind> for DDSError {
    fn from(value: DDSErrorKind) -> Self {
        Self { kind: value }
    }
}

impl From<i32> for DDSError {
    fn from(value: i32) -> Self {
        assert_ne!(1, value);
        Self { kind: value.into() }
    }
}

impl core::fmt::Display for DDSError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "something went wrong while solving boards:\n\t{}",
            self.kind
        )
    }
}
impl std::error::Error for DDSError {}

#[derive(Debug)]
pub enum DDSErrorKind {
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
}

impl From<core::ffi::c_int> for DDSErrorKind {
    fn from(value: core::ffi::c_int) -> Self {
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
            // SAFETY:  return value from DDS cannot be different from its defined constants
            _ => unreachable!(),
        }
    }
}

impl core::fmt::Display for DDSErrorKind {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::UnknownFault => write!(f, "fopen() failed or wrong number of boards"),
            Self::ZeroCards => write!(f, "zero cards"),
            Self::TargetTooHigh => write!(f, "target higher than number of tricks remaining"),
            Self::DuplicateCards => write!(f, "duplicated card"),
            Self::TargetWrongLo => write!(f, "target is less than -1"),
            Self::TargetWrongHi => write!(f, "target is higher than 13"),
            Self::SolnsWrongLo => write!(f, "solutions is less than 1"),
            Self::SolnsWrongHi => write!(f, "solutions is more than 3"),
            Self::TooManyCards => write!(f, "too many cards"),
            Self::SuitOrRank => write!(f, "currentTrickSuit or currentTrickRank have wrong data"),
            Self::PlayedCard => write!(f, "card already played"),
            Self::CardCount => write!(f, "wrong number of remining cards for a hand"),
            Self::ThreadIndex => write!(f, "thread number is less than 0 or higher than the maximum permitted"),
            Self::ModeWrongLo => write!(f, "mode is less than 0"),
            Self::ModeWrongHi => write!(f, "mode is greater than 2"),
            Self::TrumpWrong => write!(f, "trump is not one of 0,1,2,3 or 4"),
            Self::FirstWrong => write!(f, "first is not one of 0,1,2 or 3"),
            Self::PlayFault => write!(f, "less than 0 or more than 52 cards supplied, invalid suit or rank supplied or played card is not held by the right player"),
            Self::PbnFault => write!(f, "PBN string is malformed"),
            Self::ThreadCreate => write!(f, "thread created"),
            Self::ThreadWait => write!(f, "something went wrong while waiting for threads to complete"),
            Self::NoSuit => write!(f, "denomination filter vector has no entries"),
            Self::TooManyTables => write!(f, "too many tables requested"),
            Self::ChunkSize => write!(f, "chunk size is less than 1"),
        }
    }
}
