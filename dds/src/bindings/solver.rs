// Copyright (C) 2024 Alvaro Gaiotti
// See end of file for license information

use core::ffi::c_int;

use crate::bindings::{
    deal::AsDDSDeal, future_tricks::FutureTricks, AsDDSContract, DDSError, MAXNOOFBOARDS,
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
    ) -> Result<u8, DDSError>;

    fn dd_tricks_parallel<D: AsDDSDeal, C: AsDDSContract>(
        &self,
        number_of_deals: i32,
        deals: &[D; MAXNOOFBOARDS],
        contract: &[C; MAXNOOFBOARDS],
    ) -> Result<Vec<u8>, DDSError>;
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct SolvedBoards {
    pub no_of_boards: ::std::os::raw::c_int,
    pub solved_board: [FutureTricks; 200usize],
}

impl SolvedBoards {
    pub fn new(no_of_boards: c_int) -> Self {
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
