use std::num::NonZeroI32;
pub enum ThreadIndex {
    Auto,
    NumThreads(NonZeroI32),
}

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
