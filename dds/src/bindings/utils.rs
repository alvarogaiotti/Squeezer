use crate::RawDDS;
use squeezer_macros::RawDDS;
use std::ffi::c_int;
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

#[derive(RawDDS)]
pub struct SuitSeq {
    #[raw]
    sequence: [c_int; SEQUENCE_LENGTH],
    pub length: c_int,
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
