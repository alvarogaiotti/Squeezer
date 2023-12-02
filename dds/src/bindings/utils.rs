use crate::RawDDS;
use squeezer_macros::RawDDS;
use std::ffi::c_int;
use std::fmt::write;
use std::num::NonZeroI32;
pub enum ThreadIndex {
    Auto,
    NumThreads(NonZeroI32),
}

const SEQUENCE_LENGTH: usize = 52;

impl From<ThreadIndex> for std::ffi::c_int {
    fn from(value: ThreadIndex) -> Self {
        match value {
            ThreadIndex::Auto => 0,
            ThreadIndex::NumThreads(value) => value.into(),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Target {
    MaxTricks,
    LegalNoScore,
    Goal(NonZeroI32),
}

impl From<Target> for std::ffi::c_int {
    fn from(value: Target) -> Self {
        match value {
            Target::MaxTricks => -1,
            Target::LegalNoScore => 0,
            Target::Goal(goal) => std::ffi::c_int::max(13, goal.into()),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Solutions {
    Best,
    AllOptimal,
    AllLegal,
}

impl From<Solutions> for std::ffi::c_int {
    fn from(value: Solutions) -> Self {
        match value {
            Solutions::Best => 1,
            Solutions::AllOptimal => 2,
            Solutions::AllLegal => 3,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Mode {
    Auto,
    AutoSearchAlways,
    Always,
}

impl From<Mode> for std::ffi::c_int {
    fn from(value: Mode) -> Self {
        match value {
            Mode::Auto => 0,
            Mode::AutoSearchAlways => 1,
            Mode::Always => 2,
        }
    }
}

pub enum Side {
    NS = 0,
    EW = 1,
}

#[derive(Debug)]
pub enum SeqError {
    SequenceTooLong,
    SequenceTooShort,
}

impl std::error::Error for SeqError {}

impl std::fmt::Display for SeqError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let string = match self {
            Self::SequenceTooLong => format!("Sequence length is longer than {}", SEQUENCE_LENGTH),
            Self::SequenceTooShort => format!("Sequence length is shorter than {}", 1),
        };
        write!(f, "{}", string)
    }
}

#[derive(RawDDS)]
pub struct SuitSeq {
    #[raw]
    sequence: [c_int; SEQUENCE_LENGTH],
    pub length: c_int,
}
impl TryFrom<&[c_int]> for SuitSeq {
    type Error = SeqError;
    fn try_from(value: &[c_int]) -> Result<Self, Self::Error> {
        let length = value.len();
        if length == 0 {
            Err(SeqError::SequenceTooShort)
        } else if length > SEQUENCE_LENGTH {
            Err(SeqError::SequenceTooLong)
        } else {
            let mut array = Vec::with_capacity(SEQUENCE_LENGTH);
            array.extend_from_slice(value);
            array.resize(SEQUENCE_LENGTH, -1);
            Ok(Self {
                // SAFETY: checks already performed above
                sequence: array.try_into().unwrap(),
                length: length as i32,
            })
        }
    }
}
impl SuitSeq {
    /// Create a new `SuitSeq`, validating input.
    /// Slice gets truncated if too long
    pub fn new(mut sequence: &[c_int]) -> Self {
        let mut length = sequence.len();
        if length > SEQUENCE_LENGTH {
            (sequence, _) = sequence.split_at(SEQUENCE_LENGTH);
            length = SEQUENCE_LENGTH;
            Self {
                // SAFETY: checks already performed above
                sequence: sequence.try_into().unwrap(),
                length: length as c_int,
            }
        } else {
            let mut array = Vec::with_capacity(SEQUENCE_LENGTH);
            array.extend_from_slice(sequence);
            array.resize(SEQUENCE_LENGTH, -1);
            Self {
                // SAFETY: checks already performed above
                sequence: array.try_into().unwrap(),
                length: length as i32,
            }
        }
    }
}

#[derive(RawDDS)]
pub struct RankSeq {
    #[raw]
    sequence: [c_int; SEQUENCE_LENGTH],
    pub length: c_int,
}

impl RankSeq {
    /// Create a new `RankSeq`, validating input.
    /// Slice gets truncated if too long
    pub fn new(mut sequence: &[c_int]) -> Self {
        let mut length = sequence.len();
        if length > SEQUENCE_LENGTH {
            (sequence, _) = sequence.split_at(SEQUENCE_LENGTH);
            length = SEQUENCE_LENGTH;
            Self {
                // SAFETY: checks already performed above
                sequence: sequence.try_into().unwrap(),
                length: length as c_int,
            }
        } else {
            let mut array = Vec::with_capacity(SEQUENCE_LENGTH);
            array.extend_from_slice(sequence);
            array.resize(SEQUENCE_LENGTH, -1);
            Self {
                // SAFETY: checks already performed above
                sequence: array.try_into().unwrap(),
                length: length as i32,
            }
        }
    }
}
