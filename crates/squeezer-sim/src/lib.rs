// Copyright (C) 2025 Alvaro Gaiotti
// See end of file for license information

#![warn(clippy::pedantic)]
#![allow(
    dead_code,
    clippy::module_name_repetitions,
    clippy::integer_division_remainder_used,
    clippy::missing_docs_in_private_items,
    clippy::pub_with_shorthand,
    clippy::question_mark_used,
    clippy::as_conversions,
    clippy::indexing_slicing,
    clippy::missing_trait_methods,
    clippy::single_call_fn,
    clippy::unseparated_literal_suffix,
    clippy::implicit_return,
    clippy::should_panic_without_expect,
    clippy::needless_doctest_main
)]

//! Monte Carlo simulation and performance analysis over the double-dummy
//! solver seam.
//!
//! This crate owns the contract-payoff, card-EV and opening-lead simulations,
//! the result and statistics types they produce, the performance analysis of a
//! recorded board, and the terminal renderers behind the `formatters` feature.
//!
//! # Solver backend
//!
//! Every simulation takes a [`ContextFactory`](squeezer_solver::ContextFactory)
//! through a builder. With the `dds` feature on (the default) omitting it falls
//! back to the native `DdsSolver`; with the feature off the caller must pass a
//! factory, which is how the mock-solver tests run without linking DDS.
//!
//! # Errors
//!
//! A simulation reports [`SimulationError`]: invalid builder input, a dealer
//! failure or a solver failure. Performance analysis reports
//! [`AnalysisError<E>`], parameterised by the analyser's own error, so a custom
//! backend keeps its error type instead of being flattened into a crate-wide
//! enum.

mod error;
pub mod performance_analysis;
pub mod simulation;

pub use error::{AnalysisError, SimulationError};

/// The card and model vocabulary the simulation modules share, mirroring the
/// root crate's historical prelude so the moved code reads the same.
///
/// This is crate-internal: consumers import from [`squeezer_cards`] and
/// [`squeezer_model`] directly.
mod prelude {
    pub(crate) use crate::error::{AnalysisError, SimulationError};
    pub(crate) use crate::simulation::*;
    pub(crate) use squeezer_cards::*;
    pub(crate) use squeezer_model::*;
    pub(crate) use squeezer_solver::*;
    #[cfg(test)]
    pub(crate) use std::str::FromStr;
}
