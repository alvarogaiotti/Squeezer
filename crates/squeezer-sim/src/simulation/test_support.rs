// Copyright (C) 2025 Alvaro Gaiotti
// See end of file for license information

//! Test-only solver stubs shared by the simulation module's unit tests.
//!
//! The card and trick count are arbitrary: the tests using this factory assert
//! that a configuration reaches the solver, not what the solver reports, so
//! one stub keeps the mock-solver tests without the native backend from
//! drifting apart.

use crate::prelude::*;

/// A factory whose contexts report one legal card at seven tricks.
pub(crate) struct ConstantFactory;

pub(crate) struct ConstantContext;

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
