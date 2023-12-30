use core::ffi::c_int;

use crate::bindings::{
    ddsffi::{boards, futureTricks, solvedBoards, SolveAllChunksBin, SolveBoard},
    deal::{AsDDSDeal, DDSDealBuilder},
    future_tricks::FutureTricks,
    utils::build_c_deal,
    AsDDSContract, Boards, DDSDeal, DDSError, Mode, RawDDSRef, RawDDSRefMut, Solutions, Target,
    ThreadIndex, MAXNOOFBOARDSEXPORT,
};

#[allow(clippy::module_name_repetitions)]
pub trait BridgeSolver {
    /// Returns the number of tricks makable in one contract by one player
    /// # Errors
    /// Returns errors if the deal is impossible to be constructed or if the
    /// solver errors out
    fn dd_tricks<D: AsDDSDeal, C: AsDDSContract>(
        &self,
        deal: &D,
        contract: &C,
    ) -> Result<u8, Box<dyn std::error::Error>>;
}

#[non_exhaustive]
#[allow(clippy::module_name_repetitions)]
pub struct DDSSolver;

impl BridgeSolver for DDSSolver {
    #[inline]
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
                *c_deal.get_raw(),
                Target::MaxTricks.into(),
                Solutions::Best.into(),
                Mode::AutoSearchAlways.into(),
                futp,
                ThreadIndex::Auto.into(),
            );
        };
        if result != 1 {
            return Err(Box::new(DDSError::from(result)));
        }
        return Ok(13 - future_tricks.score()[0] as u8);
    }
}

impl DDSSolver {
    fn dd_tricks_parallel<D: AsDDSDeal, C: AsDDSContract>(
        &self,
        number_of_deals: i32,
        deals: &[D; MAXNOOFBOARDSEXPORT],
        contract: &[C; MAXNOOFBOARDSEXPORT],
    ) -> Result<Vec<u8>, DDSError> {
        let mut boards = Boards::new(
            number_of_deals,
            deals,
            contract,
            [Target::MaxTricks; MAXNOOFBOARDSEXPORT],
            [Solutions::Best; MAXNOOFBOARDSEXPORT],
            [Mode::Auto; MAXNOOFBOARDSEXPORT],
        )?;
        let mut solved_boards = SolvedBoards {
            solved_boards: solvedBoards::new(number_of_deals),
        };
        let result;
        {
            let bop: *mut boards = boards.get_raw_mut();
            let solved_boards_ptr: *mut solvedBoards = solved_boards.get_raw_mut();
            unsafe {
                result = SolveAllChunksBin(bop, solved_boards_ptr, 1);
            }
        };
        if result != 1 {
            return Err(result.into());
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

#[derive(Debug, RawDDSRef, RawDDSRefMut)]
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
