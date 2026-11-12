// Copyright (C) 2025 Alvaro Gaiotti
// See end of file for license information

//! # Shared pool-solve engine
//!
//! The single home of solver knowledge for Monte Carlo simulations. Three
//! `run()` implementations (contract payoff, card payoff, lead) used to share a
//! batching core that drove DDS's `SolveAllBoardsBin` batch entry point behind a
//! process-global mutex. They now share one pool-based core instead:
//!
//! - [`solve_tricks_pool`]: one declarer-relative trick count per variant
//!   (`payoff.rs`).
//! - [`solve_all_cards_pool`]: every legal play with its DD tricks
//!   (`lead.rs`).
//!
//! # Why deals are the unit of parallel work
//!
//! A context's transposition table only survives while the *deal* is unchanged;
//! changing the trump resets it (`solver_if.cpp` `newTrump`). The engine
//! therefore issues one closure call per sampled deal — the caller's fan-out
//! solves every variant of that deal consecutively on the single context rayon
//! hands to the call — and never fans individual solves across workers. The
//! caller orders the variants so same-strain contracts are adjacent; see
//! [`crate::simulation::payoff`] and ADR-0005.
//!
//! # Error and threading discipline
//!
//! Deals are generated sequentially, before any parallel work, so the dealer's
//! RNG order does not depend on the worker count and results are reproducible.
//! The fan-out returns the [`Position`]s to solve, so all DDS FFI stays behind
//! the seam's [`SolverContext`](squeezer_solver::SolverContext). A fan-out that
//! produces the wrong number of variants is an error, not a scrambled buffer.

use crate::prelude::*;
use squeezer_solver::{ContextFactory, DynSolverPool};

/// Per-deal trick counts in variant order, with the global deal index.
#[derive(Debug, Clone)]
pub(crate) struct DealTricks {
    /// Index of the deal in `0..num_deals`.
    pub deal_index: usize,
    /// One declarer-relative trick count per variant, in fan-out order.
    pub tricks: Vec<u8>,
}

/// Per-deal all-cards result with the global deal index and the solved
/// position, so a caller can keep the representative position without
/// re-deriving it.
#[derive(Debug, Clone)]
pub(crate) struct DealCards {
    /// Index of the deal in `0..num_deals`.
    pub deal_index: usize,
    /// The position that was solved for this deal.
    pub position: Position,
    /// Every legal play of the solved position, with DD tricks and classes.
    pub results: SolveResults,
}

/// Resolves a builder's optional solver factory.
///
/// With `dds`, an omitted factory falls back to the native backend. Without it,
/// the caller must inject one, so mock-solver tests and custom backends never
/// link DDS.
///
/// The `Result` is only fallible without `dds`; keeping one signature for both
/// builds would otherwise force every call site to branch on the feature.
#[allow(clippy::unnecessary_wraps)]
pub(crate) fn resolve_solver_factory(
    factory: Option<Box<dyn ContextFactory>>,
    _context: &'static str,
) -> Result<Box<dyn ContextFactory>, SimulationError> {
    match factory {
        Some(factory) => Ok(factory),
        #[cfg(feature = "dds")]
        None => Ok(Box::new(squeezer_solver::DdsSolver::new())),
        #[cfg(not(feature = "dds"))]
        None => Err(SimulationError::invalid_input(
            _context,
            "a solver factory is required when the dds feature is off",
        )),
    }
}

/// The opening position for `deal` under `contract`.
///
/// The deal's hands are unchanged and the table is empty, so the only inputs
/// that matter are the strain and the opening leader;
/// [`Position::from_deal_prefix_with_contract`] derives it without assembling a
/// full [`Board`] for every contract of every sampled deal.
pub(crate) fn opening_position(deal: &Deal, contract: Contract) -> Position {
    Position::from_deal_prefix_with_contract(deal, &[], contract)
        .expect("an empty prefix over a deal is always a valid position")
}

/// Generates `num_deals` deals sequentially, preserving the dealer's order.
fn generate_deals<D: Dealer + ?Sized>(
    dealer: &D,
    num_deals: usize,
) -> Result<Vec<Deal>, SimulationError> {
    let mut deals = Vec::with_capacity(num_deals);
    for _ in 0..num_deals {
        deals.push(dealer.deal()?);
    }
    Ok(deals)
}

/// Solves every deal for `variants_per_deal` variants, trick counts only.
///
/// The result is one [`DealTricks`] per deal, in deal order, with the variant
/// counts in the order the fan-out returned them. Trick counts are
/// declarer-relative: the seam reports the side to play, which for an opening
/// position is the defending side, so the engine complements it.
///
/// # Errors
///
/// Propagates dealer and solver errors, and rejects a fan-out that returns a
/// number of positions other than `variants_per_deal`.
pub(crate) fn solve_tricks_pool<D, F>(
    factory: Box<dyn ContextFactory>,
    dealer: &D,
    num_deals: usize,
    variants_per_deal: usize,
    sim: &'static str,
    fan_out: F,
) -> Result<Vec<DealTricks>, SimulationError>
where
    D: Dealer + ?Sized,
    F: Fn(&Deal, usize) -> Result<Vec<Position>, SimulationError> + Send + Sync,
{
    if variants_per_deal == 0 {
        return Err(SimulationError::invalid_input(
            sim,
            "variants per deal must be at least 1",
        ));
    }
    let deals = generate_deals(dealer, num_deals)?;
    let pool = DynSolverPool::with_available_workers(factory);
    pool.try_map(num_deals, |context, deal_index| {
        let positions = fan_out(&deals[deal_index], deal_index)?;
        if positions.len() != variants_per_deal {
            return Err(SimulationError::invalid_input(
                sim,
                format!(
                    "fan-out produced {} variants for deal {deal_index}, expected {variants_per_deal}",
                    positions.len()
                ),
            ));
        }
        let mut tricks = Vec::with_capacity(positions.len());
        for position in &positions {
            let side_to_play = context.solve_tricks(position)?;
            tricks.push(13u8.saturating_sub(side_to_play));
        }
        Ok(DealTricks { deal_index, tricks })
    })
}

/// Solves every deal for all of its legal plays.
///
/// # Errors
///
/// Propagates dealer and solver errors.
pub(crate) fn solve_all_cards_pool<D, F>(
    factory: Box<dyn ContextFactory>,
    dealer: &D,
    num_deals: usize,
    fan_out: F,
) -> Result<Vec<DealCards>, SimulationError>
where
    D: Dealer + ?Sized,
    F: Fn(&Deal, usize) -> Result<Position, SimulationError> + Send + Sync,
{
    let deals = generate_deals(dealer, num_deals)?;
    let pool = DynSolverPool::with_available_workers(factory);
    pool.try_map(num_deals, |context, deal_index| {
        let position = fan_out(&deals[deal_index], deal_index)?;
        let results = context.solve_all_cards(&position)?;
        Ok(DealCards {
            deal_index,
            position,
            results,
        })
    })
}

#[cfg(test)]
mod test {
    use super::*;
    use squeezer_solver::{CardSolve, SolveMode, SolverContext, SolverError};

    /// Fully predealt dealer: every `deal()` returns the same hands, so
    /// batching and indices are deterministic without seeding any RNG.
    fn fixed_dealer() -> impl Dealer {
        DealerBuilder::new()
            .predeal(
                Seat::North,
                Cards::from_str("SAKQJT98765432").expect("fixed cards parse"),
            )
            .expect("fixed cards do not overlap")
            .predeal(
                Seat::East,
                Cards::from_str("HAKQJT98765432").expect("fixed cards parse"),
            )
            .expect("fixed cards do not overlap")
            .predeal(
                Seat::South,
                Cards::from_str("DAKQJT98765432").expect("fixed cards parse"),
            )
            .expect("fixed cards do not overlap")
            .predeal(
                Seat::West,
                Cards::from_str("CAKQJT98765432").expect("fixed cards parse"),
            )
            .expect("fixed cards do not overlap")
            .build()
            .expect("fixed dealer builds")
    }

    /// A factory whose contexts report a fixed DD trick count, so the engine
    /// can be exercised without the native solver.
    struct ConstantFactory;

    struct ConstantContext;

    impl ContextFactory for ConstantFactory {
        fn new_context(&self) -> Box<dyn SolverContext> {
            Box::new(ConstantContext)
        }
    }

    impl SolverContext for ConstantContext {
        fn solve(
            &mut self,
            position: &Position,
            _mode: SolveMode,
            _solutions: Solutions,
        ) -> Result<SolveResults, SolverError> {
            let cards = position
                .legal_plays()
                .into_iter()
                .next()
                .map(|card| {
                    vec![CardSolve {
                        card,
                        tricks: 7,
                        equivalents: Cards::EMPTY.insert(card),
                    }]
                })
                .unwrap_or_default();
            Ok(SolveResults { cards })
        }

        fn clear_tt(&mut self) {}

        fn reset(&mut self) {}
    }

    #[allow(clippy::unnecessary_wraps)]
    fn fan_out_two(deal: &Deal, _deal_index: usize) -> Result<Vec<Position>, SimulationError> {
        Ok(vec![
            opening_position(
                deal,
                Contract::from_str("1NN").expect("fixed contract parses"),
            ),
            opening_position(
                deal,
                Contract::from_str("2SN").expect("fixed contract parses"),
            ),
        ])
    }

    #[test]
    fn tricks_pool_covers_every_deal_with_contiguous_indices() {
        let solved = solve_tricks_pool(
            Box::new(ConstantFactory),
            &fixed_dealer(),
            250,
            2,
            "test",
            fan_out_two,
        )
        .expect("constant solves never fail");

        assert_eq!(solved.len(), 250);
        for (i, entry) in solved.iter().enumerate() {
            assert_eq!(entry.deal_index, i);
            assert_eq!(entry.tricks.len(), 2);
            // The mock reports 7 tricks for the side to play, so the
            // declarer-relative count is 13 - 7 = 6.
            assert_eq!(entry.tricks, vec![6, 6]);
        }
    }

    #[test]
    fn tricks_pool_rejects_zero_variants() {
        assert!(
            solve_tricks_pool(
                Box::new(ConstantFactory),
                &fixed_dealer(),
                10,
                0,
                "test",
                fan_out_two
            )
            .is_err()
        );
    }

    #[test]
    fn tricks_pool_detects_misaligned_fan_out() {
        let misaligned =
            |deal: &Deal, _deal_index: usize| -> Result<Vec<Position>, SimulationError> {
                // Declares 2 variants but returns 1: must error, not scramble.
                Ok(vec![opening_position(
                    deal,
                    Contract::from_str("1NN").expect("fixed contract parses"),
                )])
            };
        assert!(
            solve_tricks_pool(
                Box::new(ConstantFactory),
                &fixed_dealer(),
                3,
                2,
                "test",
                misaligned
            )
            .is_err()
        );
    }

    #[test]
    fn tricks_pool_propagates_fan_out_errors() {
        let failing = |_: &Deal, _: usize| -> Result<Vec<Position>, SimulationError> {
            Err(SimulationError::invalid_input("test", "boom"))
        };
        assert!(
            solve_tricks_pool(
                Box::new(ConstantFactory),
                &fixed_dealer(),
                3,
                2,
                "test",
                failing
            )
            .is_err()
        );
    }

    #[test]
    fn all_cards_pool_covers_every_deal() {
        let solved = solve_all_cards_pool(
            Box::new(ConstantFactory),
            &fixed_dealer(),
            205,
            |deal: &Deal, _deal_index: usize| {
                Ok(opening_position(
                    deal,
                    Contract::from_str("1NN").expect("fixed contract parses"),
                ))
            },
        )
        .expect("constant solves never fail");

        assert_eq!(solved.len(), 205);
        for (i, entry) in solved.iter().enumerate() {
            assert_eq!(entry.deal_index, i);
        }
        // Every deal is the same fixed deal, so identical analysis.
        assert_eq!(
            solved[0].results.cards.len(),
            solved[204].results.cards.len()
        );
    }
}
