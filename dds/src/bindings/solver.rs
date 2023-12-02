use squeezer_macros::RawDDS;
use std::ffi::c_int;

use crate::bindings::{
    ddserror::DDSErrorKind,
    ddsffi::{
        boards, deal, futureTricks, solvedBoards, solvedPlay, solvedPlays, SolveAllBoards,
        SolveAllChunksBin, SolveBoard,
    },
    deal::{AsDDSDeal, DDSDealBuilder},
    future_tricks::FutureTricks,
    traits::RawDDS,
    AsDDSContract, Boards, DDSDeal, DDSError, Mode, Solutions, Target, ThreadIndex,
    MAXNOOFBOARDSEXPORT,
};

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
        let mut future_tricks = FutureTricks::new();
        let futp: *mut futureTricks = &mut future_tricks.0;
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
        number_of_deals: i32,
        deals: &[D; MAXNOOFBOARDSEXPORT],
        contract: &[C; MAXNOOFBOARDSEXPORT],
    ) -> Result<Vec<u8>, DDSError> {
        let boards = Boards::new(
            number_of_deals,
            deals,
            contract,
            [Target::MaxTricks; MAXNOOFBOARDSEXPORT],
            [Solutions::Best; MAXNOOFBOARDSEXPORT],
            [Mode::Auto; MAXNOOFBOARDSEXPORT],
        )?;
        let bop: *mut boards = &mut boards.get_raw();
        let solved_boards = SolvedBoards::new(number_of_deals);
        let result;
        {
            let solved_boards_ptr: *mut solvedBoards = &mut solved_boards.get_raw();
            unsafe {
                result = SolveAllChunksBin(bop, solved_boards_ptr, 1);
            }
        };
        if result != 1 {
            return Err(DDSError::new(result));
        }
        Ok(solved_boards
            .get_raw()
            .solvedBoard
            .into_iter()
            .map(|ft| ft.score[0] as u8)
            .take(number_of_deals as usize)
            .collect())
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

#[derive(Debug, RawDDS)]
pub struct SolvedBoards {
    #[raw]
    solved_boards: solvedBoards,
}

impl solvedBoards {
    fn new(no_of_boards: c_int) -> Self {
        Self {
            noOfBoards: no_of_boards,
            solvedBoard: [futureTricks::default(); MAXNOOFBOARDSEXPORT],
        }
    }
}

impl SolvedBoards {
    pub fn new(no_of_boards: c_int) -> Self {
        Self {
            solved_boards: solvedBoards::new(no_of_boards),
        }
    }
}
