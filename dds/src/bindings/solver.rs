use rusty_dealer_macros::RawDDS;

use super::{
    ddsffi::{solvedPlay, solvedPlays, SolveBoard},
    deal::{AsDDSDeal, DDSDealBuilder},
    future_tricks::FutureTricks,
    AsDDSContract, DDSError, Mode, Solutions, Target, ThreadIndex,
};
use crate::RawDDS;

#[derive(RawDDS)]
pub struct SolvedPlays {
    #[raw]
    solved_play: solvedPlays,
}

#[derive(RawDDS)]
pub struct SolvedPlay {
    #[raw]
    solved_play: solvedPlay,
}

impl SolvedPlay {
    pub fn new() -> Self {
        Self {
            solved_play: solvedPlay {
                number: 0,
                tricks: [0; 53],
            },
        }
    }
    pub fn tricks(&self) -> &[i32; 53usize] {
        self.get_tricks()
    }

    fn get_tricks(&self) -> &[i32; 53usize] {
        &self.solved_play.tricks
    }

    pub fn number(&self) -> i32 {
        self.get_number()
    }
    fn get_number(&self) -> i32 {
        self.get_raw().number
    }
}

impl Default for SolvedPlay {
    fn default() -> Self {
        Self::new()
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
        if result != 1 {
            return Err(Box::new(DDSError::new(result)));
        }
        Ok(13 - future_tricks.score()[0] as u8)
    }
}
