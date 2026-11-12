use criterion::measurement::Measurement;
use criterion::{
    BatchSize, BenchmarkGroup, Criterion, Throughput, criterion_group, criterion_main,
};
use squeezer_cards::*;
use squeezer_model::*;
use squeezer_sim::simulation::*;
use squeezer_solver::*;
use std::hint::black_box;
use std::str::FromStr;

struct FixedFactory;

struct FixedContext;

impl ContextFactory for FixedFactory {
    fn new_context(&self) -> Box<dyn SolverContext> {
        Box::new(FixedContext)
    }
}

impl DdTableCalculator for FixedFactory {}

impl SolverContext for FixedContext {
    fn solve(
        &mut self,
        position: &Position,
        _mode: SolveMode,
        _solutions: Solutions,
    ) -> Result<SolveResults, SolverError> {
        let cards = position
            .legal_plays()
            .into_iter()
            .map(|card| CardSolve {
                card,
                tricks: 7,
                equivalents: Cards::EMPTY.insert(card),
            })
            .collect();
        Ok(SolveResults { cards })
    }

    fn clear_tt(&mut self) {}

    fn reset(&mut self) {}
}

fn example_deal() -> Deal {
    Deal::new([
        Cards::from_str("SQJ6 HK652 DJ85 CT98").expect("north hand parses"),
        Cards::from_str("S873 HJ97 DAT764 CQ4").expect("east hand parses"),
        Cards::from_str("SK5 HT83 DKQ9 CA7652").expect("south hand parses"),
        Cards::from_str("SAT942 HAQ4 D32 CKJ3").expect("west hand parses"),
    ])
    .expect("example hands form a deal")
}

fn fixed_dealer() -> impl Dealer {
    let deal = example_deal();
    DealerBuilder::new()
        .predeal(Seat::North, deal.north())
        .expect("north is disjoint")
        .predeal(Seat::East, deal.east())
        .expect("east is disjoint")
        .predeal(Seat::South, deal.south())
        .expect("south is disjoint")
        .predeal(Seat::West, deal.west())
        .expect("west is disjoint")
        .build()
        .expect("fixed dealer builds")
}

fn example_contract() -> Contract {
    Contract::new(
        1,
        Strain::Spades,
        Seat::West,
        Vulnerable::No,
        Doubled::NotDoubled,
    )
}

fn imps_against_the_opponent() -> Denomination {
    Denomination::Compared {
        baseline: Baseline::AverageOfOptions,
        scoring: Scoring::Imps,
    }
}

fn bench_mock_card_ev<M: Measurement>(
    group: &mut BenchmarkGroup<'_, M>,
    name: &str,
    denomination: fn() -> Denomination,
) {
    group.throughput(Throughput::Elements(1));
    group.bench_function(name, |b| {
        b.iter_batched(
            || {
                CardEvaluation::builder()
                    .num_deals(1_000)
                    .dealer(fixed_dealer())
                    .contract(example_contract())
                    .denomination(denomination())
                    .solver_factory(Box::new(FixedFactory))
                    .build()
                    .expect("card EV configuration is valid")
            },
            |simulation| black_box(simulation.run().expect("card EV runs")),
            BatchSize::SmallInput,
        );
    });
}

fn bench_mock_payoff<M: Measurement>(group: &mut BenchmarkGroup<'_, M>) {
    let contracts = vec![
        Contract::from_str("3NN").expect("contract parses"),
        Contract::from_str("3HN").expect("contract parses"),
        Contract::from_str("3DN").expect("contract parses"),
        Contract::from_str("4HN").expect("contract parses"),
    ];
    group.throughput(Throughput::Elements(1));
    group.bench_function("mock_payoff_postprocess", |b| {
        b.iter_batched(
            || {
                ContractPayoffSimulation::builder()
                    .num_deals(1_000)
                    .dealer(fixed_dealer())
                    .contracts(contracts.clone())
                    .denomination(imps_against_the_opponent())
                    .solver_factory(Box::new(FixedFactory))
                    .build()
                    .expect("payoff configuration is valid")
            },
            |simulation| black_box(simulation.run().expect("payoff runs")),
            BatchSize::SmallInput,
        );
    });
}

fn bench_mock_lead<M: Measurement>(group: &mut BenchmarkGroup<'_, M>) {
    group.throughput(Throughput::Elements(1));
    group.bench_function("mock_lead_postprocess", |b| {
        b.iter_batched(
            || {
                LeadSimulationBuilder::new()
                    .num_deals(1_000)
                    .dealer(fixed_dealer())
                    .contract(example_contract())
                    .solver_factory(Box::new(FixedFactory))
                    .build()
                    .expect("lead configuration is valid")
            },
            |simulation| black_box(simulation.run().expect("lead runs")),
            BatchSize::SmallInput,
        );
    });
}

fn bench_position_derivation<M: Measurement>(group: &mut BenchmarkGroup<'_, M>) {
    let deal = Deal::new([
        Cards::from_str("SAKQJT98765432").expect("north hand parses"),
        Cards::from_str("HAKQJT98765432").expect("east hand parses"),
        Cards::from_str("DAKQJT98765432").expect("south hand parses"),
        Cards::from_str("CAKQJT98765432").expect("west hand parses"),
    ])
    .expect("one-suit-per-seat deal is valid");
    let prefix = [
        Play::new(Seat::North, Card::S2),
        Play::new(Seat::East, Card::H2),
        Play::new(Seat::South, Card::D2),
        Play::new(Seat::West, Card::C2),
    ];
    let contract = Contract::new(
        1,
        Strain::Spades,
        Seat::West,
        Vulnerable::No,
        Doubled::NotDoubled,
    );

    group.throughput(Throughput::Elements(1));
    // A completed trick still needs its trick-winner scan and the full
    // snapshot validation, so this case guards the general path against
    // regressions as the opening path gets cheaper.
    group.bench_function("position_from_prefix", |b| {
        b.iter(|| {
            black_box(
                Position::from_deal_prefix_with_contract(&deal, &prefix, contract)
                    .expect("prefix is valid"),
            )
        });
    });

    // The opening position is the hot case in simulations: one derivation per
    // contract of every sampled deal, always with an empty prefix.
    group.bench_function("position_opening", |b| {
        b.iter(|| {
            black_box(
                Position::from_deal_prefix_with_contract(&deal, &[], contract)
                    .expect("an empty prefix is valid"),
            )
        });
    });
}

fn benchmark_simulation_paths(c: &mut Criterion) {
    let mut group = c.benchmark_group("simulation_paths_mock");
    group.sample_size(10);

    bench_position_derivation(&mut group);
    bench_mock_card_ev(&mut group, "mock_card_ev_raw", || Denomination::RawPoints);
    bench_mock_card_ev(&mut group, "mock_card_ev_imps", imps_against_the_opponent);
    bench_mock_lead(&mut group);
    bench_mock_payoff(&mut group);

    group.finish();
}

criterion_group!(benches, benchmark_simulation_paths);
criterion_main!(benches);
