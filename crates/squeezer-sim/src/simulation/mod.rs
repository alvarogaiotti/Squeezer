//! Monte Carlo simulations over the solver seam.
//!
//! Every simulation is a builder: configure the dealer, contract(s), deal
//! count and valuation, then inject a
//! [`ContextFactory`](squeezer_solver::ContextFactory). With the `dds` feature
//! the factory may be omitted and the native backend is used; the example
//! below injects a tiny mock instead, so it runs without DDS.
//!
//! # Example: one card evaluation against a mock backend
//!
//! ```rust
//! use squeezer_cards::Cards;
//! use squeezer_model::*;
//! use squeezer_sim::simulation::*;
//! use squeezer_solver::*;
//! use std::str::FromStr;
//!
//! // A backend that reports one legal card at seven tricks. That is enough
//! // to exercise the simulation's derivation and reporting paths without the
//! // native solver.
//! struct Mock;
//! struct MockContext;
//!
//! impl ContextFactory for Mock {
//!     fn new_context(&self) -> Box<dyn SolverContext> {
//!         Box::new(MockContext)
//!     }
//! }
//!
//! impl DdTableCalculator for Mock {}
//!
//! impl SolverContext for MockContext {
//!     fn solve(
//!         &mut self,
//!         position: &Position,
//!         _mode: SolveMode,
//!         _solutions: Solutions,
//!     ) -> Result<SolveResults, SolverError> {
//!         let cards = position
//!             .legal_plays()
//!             .into_iter()
//!             .next()
//!             .map(|card| CardSolve {
//!                 card,
//!                 tricks: 7,
//!                 equivalents: Cards::EMPTY.insert(card),
//!             })
//!             .into_iter()
//!             .collect();
//!         Ok(SolveResults { cards })
//!     }
//!
//!     fn clear_tt(&mut self) {}
//!
//!     fn reset(&mut self) {}
//! }
//!
//! let evaluation = CardEvaluation::builder()
//!     .num_deals(8)
//!     .dealer(StandardDealer::new())
//!     .contract(Contract::from_str("3NN")?)
//!     .solver_factory(Box::new(Mock))
//!     .build()?
//!     .run()?;
//!
//! // The mock returns one card per deal, so the eight outcomes are spread
//! // across the cards it saw; their total must still be eight.
//! assert_eq!(
//!     evaluation.values.iter().map(|value| value.statistics.deals).sum::<usize>(),
//!     8
//! );
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
pub mod card_ev;
pub mod core;
pub mod denomination;
/// Terminal renderers for simulation results, behind the `formatters` feature.
#[cfg(feature = "formatters")]
pub mod formatters;
pub mod lead;
pub mod payoff;
mod solver;
#[cfg(test)]
mod test_support;

pub use card_ev::*;
pub use core::*;
pub use denomination::*;
#[cfg(feature = "formatters")]
pub use formatters::*;
pub use lead::*;
pub use payoff::*;
pub(crate) use solver::{
    missing_solver_factory, opening_position, resolve_solver_factory, solve_all_cards_pool,
    solve_tricks_pool,
};
