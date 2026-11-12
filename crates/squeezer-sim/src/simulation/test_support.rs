// Copyright (C) 2025 Alvaro Gaiotti
// See end of file for license information

//! Test-only solver stubs and scripted result fixtures shared by the
//! simulation module's unit tests.
//!
//! The mock factory's card and trick count are arbitrary: the tests using it
//! assert that a configuration reaches the solver, not what the solver
//! reports, so one stub keeps the mock-solver tests without the native backend
//! from drifting apart. [`scripted_lead_analysis`] goes the other way: it
//! scripts the per-card outcomes so ranking, per-suit and formatter tests pin
//! exact values without depending on DDS.

use crate::prelude::*;

/// A lead view over a scripted evaluation, for tests that pin ranking,
/// per-suit and formatter behaviour without the native solver.
///
/// `dd_tricks` is the side-to-play count on each deal, so the declarer count is
/// its complement and a defender trick count of `8 - level` or more sets the
/// contract.
pub(crate) fn scripted_lead_analysis(contract: Contract, cards: &[(Card, &[u8])]) -> LeadAnalysis {
    let deal = squeezer_solver::test_fixtures::example_deal();
    let position = Position::from_deal_prefix_with_contract(&deal, &[], contract)
        .expect("the example deal has an opening position");

    let values = cards
        .iter()
        .map(|&(card, dd_tricks)| {
            let deals: Vec<CardDealOutcome> = dd_tricks
                .iter()
                .enumerate()
                .map(|(deal_index, &dd)| CardDealOutcome {
                    deal_index,
                    dd_tricks: dd,
                    declarer_tricks: 13 - dd,
                    score: 0,
                    value: 0,
                })
                .collect();
            CardValue {
                card,
                statistics: CardStatistics::from_outcomes(&deals, contract.level()),
                deals,
            }
        })
        .collect();

    LeadAnalysis::new(
        CardEvaluation {
            position,
            denomination: Denomination::RawPoints,
            values,
            dd_optimal: Vec::new(),
        },
        contract,
    )
}

/// A factory whose contexts report one legal card at seven tricks.
pub(crate) struct ConstantFactory;

pub(crate) struct ConstantContext;

impl ContextFactory for ConstantFactory {
    fn new_context(&self) -> Box<dyn SolverContext> {
        Box::new(ConstantContext)
    }
}

// The mock backend solves positions but has no DD-table capability, so the
// trait's default error is enough.
impl DdTableCalculator for ConstantFactory {}

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
