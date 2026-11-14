// Copyright (C) 2025 Alvaro Gaiotti
// See end of file for license information

//! # Consuming a contract-payoff simulation
//!
//! Runs a small Monte Carlo simulation against a stub solver, then reads the
//! result the way a GUI or a report would: decompose
//! [`SimulationOutput`] with `into_parts()`, inspect the raw per-deal data,
//! the computed statistics and the run metadata, and finally render the payoff
//! matrix with the crate's terminal formatter.
//!
//! The simulation only asks for a [`ContextFactory`], and the example supplies
//! this stub, so with the default `dds` feature off nothing links the native
//! backend. Run it with the `formatters` feature enabled:
//!
//! ```sh
//! cargo run -p squeezer-sim --example contract_payoff_consumer \
//!     --no-default-features --features formatters
//! ```
//!
//! Dropping only `--no-default-features` also works — but then Cargo builds DDS
//! even though the simulated deals never reach it. The stub's trick count
//! depends on the opening leader's aces, so the scores spread across deals;
//! real double-dummy values come from injecting `DdsSolver::new()` instead.
//!
//! [`SimulationOutput`]: squeezer_sim::simulation::SimulationOutput

use squeezer_cards::Cards;
use squeezer_model::*;
use squeezer_sim::simulation::*;
use squeezer_solver::*;
use std::error::Error;
use std::str::FromStr;

/// A solver factory whose contexts answer every position with one card.
struct StubFactory;

struct StubContext;

impl ContextFactory for StubFactory {
    fn new_context(&self) -> Box<dyn SolverContext> {
        Box::new(StubContext)
    }
}

// The stub solves positions but cannot build a DD table, so the trait's
// default error is the right answer for the capability the simulation never
// asks for.
impl DdTableCalculator for StubFactory {}

impl SolverContext for StubContext {
    fn solve(
        &mut self,
        position: &Position,
        _mode: SolveMode,
        _solutions: Solutions,
    ) -> Result<SolveResults, SolverError> {
        // Pretend the defenders on lead take three tricks plus one per ace in
        // their hand. It is not double-dummy, but it varies with the deal, so
        // the statistics below are not a flat tie.
        let legal_plays = position.legal_plays();
        let aces = legal_plays.aces().len();
        let tricks = 3 + aces;

        let cards = legal_plays
            .into_iter()
            .next()
            .map(|card| CardSolve {
                card,
                tricks,
                equivalents: Cards::EMPTY.insert(card),
            })
            .into_iter()
            .collect();

        Ok(SolveResults { cards })
    }

    fn clear_tt(&mut self) {}

    fn reset(&mut self) {}
}

fn main() -> Result<(), Box<dyn Error>> {
    // Two contracts by different declarers, so the stub sees two different
    // opening leaders on every deal.
    let contracts = vec![Contract::from_str("3NN")?, Contract::from_str("4SS")?];
    let (first, second) = (contracts[0], contracts[1]);

    let output = ContractPayoffBuilder::new()
        .num_deals(48)
        .dealer(StandardDealer::new())
        .contracts(contracts)
        .solver_factory(Box::new(StubFactory))
        .build()?
        .run()?;

    // `into_parts` is the consumer seam: no further simulation work, just the
    // three values a caller embeds or stores.
    let (raw, statistics, metadata) = output.into_parts();

    println!(
        "Simulated {} deals for {} contracts.",
        metadata.num_deals, metadata.num_comparisons
    );

    println!("\nRaw per-deal data, first three deals of each contract:");
    for option in &raw.option_results {
        let sample: Vec<String> = option
            .deals
            .iter()
            .take(3)
            .map(|deal| {
                format!(
                    "deal {}: {} tricks, {:+} points",
                    deal.deal_index, deal.tricks, deal.score
                )
            })
            .collect();
        println!("  {} -> {}", option.option, sample.join("; "));
    }

    // The pairwise view is directed: one entry per `(A, B)` pair, each holding
    // the per-deal score difference a chart or a custom analysis would bucket.
    if let Some(pair) = raw.comparison_between(&first, &second) {
        let differences = pair.differences();
        println!(
            "  {first} vs {second}: {} per-deal differences (first three: {:?})",
            differences.len(),
            &differences[..differences.len().min(3)]
        );
    }

    println!("\nStatistics computed from that raw data:");
    for (index, ranking) in statistics.rankings.iter().enumerate() {
        println!(
            "  {}. {} (average {:+.2}, {} significant wins, {} losses)",
            index + 1,
            ranking.option,
            ranking.average_score,
            ranking.wins,
            ranking.losses
        );
    }

    // The pair here is directed: the first contract is the "A" side, so the
    // mean is positive when 3NT by North beats 4S by South.
    if let Some(stats) = statistics.get_stats(&first, &second) {
        println!(
            "  {first} vs {second}: mean {:+0.2}, median {:+0.2}, range [{:+}, {:+}]",
            stats.mean, stats.median, stats.min, stats.max
        );
    }

    // One rendering step: the formatter reads the same `PayoffMatrix`. The
    // plain configuration keeps the output free of ANSI colour codes, so it
    // works in a log file as well as in a terminal.
    println!("\nRendered payoff matrix:");
    print!(
        "{}",
        PayoffMatrixFormatter::with_config(&statistics, FormatterConfig::plain())
    );

    Ok(())
}
