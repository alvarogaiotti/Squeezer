use super::{
    ddsffi::SolveBoard,
    deal::{AsDDSDeal, DDSDealBuilder, DDSDealConstructionError},
    future_tricks::FutureTricks,
    AsDDSContract,
};
use crate::RawDDS;
use std::num::NonZeroI32;

enum ThreadIndex {
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

enum Target {
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

enum Solutions {
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

enum Mode {
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

pub trait BridgeSolver {
    fn dd_tricks<D: AsDDSDeal, C: AsDDSContract>(
        &self,
        deal: &D,
        contract: &C,
    ) -> Result<u8, Box<dyn std::error::Error>>;
}
pub struct DDSSolver {}

impl BridgeSolver for DDSSolver {
    fn dd_tricks<D: AsDDSDeal, C: AsDDSContract>(
        &self,
        deal: &D,
        contract: &C,
    ) -> Result<u8, Box<dyn std::error::Error>> {
        let (trump, first) = contract.as_dds_contract();
        let c_deal = DDSDealBuilder::new()
            .trump(trump.try_into()?)
            .first(first.try_into()?)
            .remain_cards(deal.as_dds_deal())
            .build()?;
        let future_tricks = FutureTricks::new();
        let futp = &mut future_tricks.get_raw();
        let result;
        unsafe {
            result = SolveBoard(
                c_deal.get_raw(),
                Target::MaxTricks.into(),
                Solutions::Best.into(),
                Mode::AutoSearchAlways.into(),
                futp,
                ThreadIndex::Auto.into(),
            )
        };
        Ok(13 - future_tricks.score()[0] as u8)
    }
}
