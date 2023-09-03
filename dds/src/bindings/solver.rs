use squeezer_macros::RawDDS;
use std::ffi::c_int;

use super::{
    ddsffi::{deal, futureTricks, solvedPlay, solvedPlays, SolveBoard},
    deal::{AsDDSDeal, DDSDealBuilder},
    future_tricks::FutureTricks,
    AsDDSContract, DDSDeal, DDSError, Mode, Solutions, Target, ThreadIndex, MAXNOOFBOARDSEXPORT,
};
use crate::{RawDDS, MAXNOOFBOARDS};

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
        let c_deal = build_c_deal(contract, deal)?;
        let future_tricks = FutureTricks::new();
        let futp: *mut futureTricks = &mut future_tricks.get_raw();
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

impl DDSSolver {
    fn dd_tricks_parallel<D: AsDDSDeal, C: AsDDSContract>(
        &self,
        deals: Vec<&D>,
        contract: Vec<&C>,
    ) -> Result<u8, Box<dyn std::error::Error>> {
        todo!()
    }
}

fn build_c_deal<C: AsDDSContract, D: AsDDSDeal>(
    contract: &C,
    deal: &D,
) -> Result<DDSDeal, Box<dyn std::error::Error>> {
    let (trump, first) = contract.as_dds_contract();
    Ok(DDSDealBuilder::new()
        .trump(trump.try_into()?)
        .first(first.try_into()?)
        .remain_cards(deal.as_dds_deal())
        .build()?)
}

pub struct SolvedBoards {
    no_of_boards: c_int,
    solved_boards: [futureTricks; MAXNOOFBOARDSEXPORT],
}

impl SolvedBoards {
    pub fn new(no_of_boards: c_int) -> Self {
        Self {
            no_of_boards,
            solved_boards: [futureTricks::default(); MAXNOOFBOARDSEXPORT],
        }
    }
}
