use core::ffi::c_int;

use crate::bindings::{
    deal::AsDDSDeal,
    ffi::{SolveAllChunksBin, SolveBoard},
    future_tricks::FutureTricks,
    utils::build_c_deal,
    AsDDSContract, Boards, DDSError, Mode, Solutions, Target, ThreadIndex, MAXNOOFBOARDS,
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
        let c_deal = build_c_deal((contract, deal))?;
        let mut future_tricks = FutureTricks::new();
        let futp: *mut FutureTricks = &mut future_tricks;
        let result;
        unsafe {
            result = SolveBoard(
                c_deal,
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
        deals: &[D; MAXNOOFBOARDS],
        contract: &[C; MAXNOOFBOARDS],
    ) -> Result<Vec<u8>, DDSError> {
        assert!(number_of_deals <= MAXNOOFBOARDS as i32);
        let mut boards = Boards::new(
            number_of_deals,
            deals,
            contract,
            &[Target::MaxTricks; MAXNOOFBOARDS],
            &[Solutions::Best; MAXNOOFBOARDS],
            &[Mode::Auto; MAXNOOFBOARDS],
        );
        let mut solved_boards = SolvedBoards::new(number_of_deals);
        let result;
        {
            let bop: *mut Boards = &mut boards;
            let solved_boards_ptr: *mut SolvedBoards = &mut solved_boards;
            unsafe {
                result = SolveAllChunksBin(bop, solved_boards_ptr, 1);
            }
        };
        if result != 1 {
            return Err(result.into());
        }
        Ok(solved_boards
            .into_iter()
            .map(|ft| ft.score[0] as u8)
            .take(number_of_deals as usize)
            .collect())
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct SolvedBoards {
    pub no_of_boards: ::std::os::raw::c_int,
    pub solved_board: [FutureTricks; 200usize],
}

impl SolvedBoards {
    fn new(no_of_boards: c_int) -> Self {
        Self {
            no_of_boards,
            solved_board: [FutureTricks::default(); MAXNOOFBOARDS],
        }
    }
}

impl IntoIterator for SolvedBoards {
    type Item = FutureTricks;
    type IntoIter = std::array::IntoIter<Self::Item, MAXNOOFBOARDS>;

    fn into_iter(self) -> Self::IntoIter {
        self.solved_board.into_iter()
    }
}
#[test]
fn bindgen_test_layout_solved_boards() {
    assert_eq!(
        ::std::mem::size_of::<SolvedBoards>(),
        43204usize,
        concat!("Size of: ", stringify!(solvedBoards))
    );
    assert_eq!(
        ::std::mem::align_of::<SolvedBoards>(),
        4usize,
        concat!("Alignment of ", stringify!(solvedBoards))
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<SolvedBoards>())).no_of_boards as *const _ as usize },
        0usize,
        concat!(
            "Offset of field: ",
            stringify!(solvedBoards),
            "::",
            stringify!(noOfBoards)
        )
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<SolvedBoards>())).solved_board as *const _ as usize },
        4usize,
        concat!(
            "Offset of field: ",
            stringify!(solvedBoards),
            "::",
            stringify!(solvedBoard)
        )
    );
}
