// Copyright (C) 2024 Alvaro Gaiotti
// See end of file for license information

use core::ffi::c_int;

use crate::{
    bindings::MAXNOOFBOARDS, ddserror::DDSError, deal::AsDDSDeal, future_tricks::FutureTricks,
    traits::AsDDSContract,
};

#[allow(clippy::module_name_repetitions)]
/// Trait representing the ability to solve a deal with a determined contract, returning the
/// number of tricks makable by a declarer in a contract.
/// Provides parallelized and unparallelized versions of the solving function.
/// If you have to solve more than a couple dozen of deals, you are better off
/// using the parallelized versions.
pub trait BridgeSolver {
    /// Returns the number of tricks makable in one contract by one player
    /// If you have more than a dozen deals to analyse use [`BridgeSolver::dd_tricks_parallel`]
    /// instead.
    /// # Errors
    /// Returns errors if the deal is impossible to be constructed or if the
    /// solver errors out
    fn dd_tricks<D: AsDDSDeal, C: AsDDSContract>(
        &self,
        deal: &D,
        contract: &C,
    ) -> Result<u8, DDSError>;
    /// Returns the number of tricks makable in one contract by one player for
    /// every lead.
    /// If you have more than a dozen deals to analyse use [`BridgeSolver::dd_tricks_parallel`]
    /// instead.
    /// # Errors
    /// Returns an error if the deal is impossible to be constructed or it the solver gives out an
    /// error
    fn dd_tricks_all_cards<D: AsDDSDeal, C: AsDDSContract>(
        &self,
        deal: &D,
        contract: &C,
    ) -> Result<FutureTricks, DDSError>;

    /// Same as [`BridgeSolver::dd_tricks`] but computes multiple deals in paralles.
    /// If you have more than a dozen deals to analyse use this function instead.
    /// # Errors
    /// Returns an error if the deal is impossible to be constructed or it the solver gives out an
    /// error
    fn dd_tricks_parallel<D: AsDDSDeal, C: AsDDSContract>(
        &self,
        number_of_deals: i32,
        deals: &[D],
        contract: &[C],
    ) -> Result<Vec<u8>, DDSError>;

    /// Same as [`BridgeSolver::dd_tricks_all_cards`] but computes multiple deals in paralles.
    /// If you have more than a dozen deals to analyse use this function instead.
    /// # Errors
    /// Returns an error if the deal is impossible to be constructed or it the solver gives out an
    /// error
    fn dd_tricks_all_cards_parallel<D: AsDDSDeal, C: AsDDSContract>(
        &self,
        number_of_deals: i32,
        deals: &[D],
        contract: &[C],
    ) -> Result<SolvedBoards, DDSError>;
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
/// DDS struct that represents a number of solved boards with its [`FutureTricks`] content.
/// Can hold up to 200 [`FutureTricks`].
pub struct SolvedBoards {
    pub no_of_boards: ::std::os::raw::c_int,
    pub solved_board: [FutureTricks; 200usize],
}

impl SolvedBoards {
    #[inline]
    #[must_use]
    /// Create a new [`SolvedBoards`] for `no_of_boards` boards.
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
#[cfg(test)]
#[allow(deref_nullptr, clippy::ref_as_ptr)]
mod test {
    use super::SolvedBoards;
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
}
