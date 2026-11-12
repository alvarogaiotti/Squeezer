// Copyright (C) 2024 Alvaro Gaiotti
// See end of file for license information

use crate::{
    ddserror::DdsError,
    deal::DdsDeal,
    future_tricks::FutureTricks,
    utils::{Mode, Solutions, Target},
};

#[allow(clippy::module_name_repetitions)]
/// Trait representing the ability to solve a deal with a determined contract, returning the
/// number of tricks makable by a declarer in a contract.
/// See <https://github.com/dds-bridge/dds/blob/develop/doc/DLL-dds_x.pdf> for documentation.
///
/// This is the single-deal surface only. Concurrency is owned by the caller's
/// thread pool (a bounded set of contexts, at most one per worker at a time),
/// not by DDS's batch entry points, so the parallel variants have been removed.
pub trait BridgeSolver {
    /// Returns the number of tricks makable in one contract by one player.
    /// # Errors
    /// Returns errors if the deal is impossible to be constructed or if the
    /// solver errors out
    fn dd_tricks(&self, deal: &DdsDeal) -> Result<u8, DdsError>;

    /// Returns the number of tricks makable in one contract by one player for
    /// every lead.
    /// # Errors
    /// Returns an error if the deal is impossible to be constructed or it the solver gives out an
    /// error
    fn dd_tricks_all_cards(&self, deal: &DdsDeal) -> Result<FutureTricks, DdsError>;

    /// With this function you gain much more flexibility regarding
    /// the solving strategy of DDS for a single deal.
    /// You can set the strategy of the searching alorithm ([`Mode`], now seems to be deprecated),
    /// the solution cards returned ([`Solutions`]) and the target of the search ([`Target`]).
    /// See <https://github.com/dds-bridge/dds/blob/develop/doc/DLL-dds_x.pdf> for documentation.
    ///
    /// # Errors
    /// Errors when DDS errors. See DDS docs for errors.
    /// Or check the inner workings of [`DdsError`].
    fn solve_with_params(
        &self,
        deal: &DdsDeal,
        mode: Mode,
        solutions: Solutions,
        target: Target,
    ) -> Result<FutureTricks, DdsError>;
}
