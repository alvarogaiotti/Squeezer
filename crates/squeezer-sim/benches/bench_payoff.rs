#![allow(clippy::all)]
use criterion::{Criterion, criterion_group, criterion_main};
use squeezer_cards::*;
use squeezer_model::*;
use squeezer_sim::simulation::*;
use std::hint::black_box;
use std::str::FromStr;

/// IMPs against the pairwise opponent, matching the old bare `imps` scoring.
fn imps_against_the_opponent() -> Denomination {
    Denomination::Compared {
        baseline: Baseline::AverageOfOptions,
        scoring: Scoring::Imps,
    }
}

#[inline]
fn payoff_report_test(_: ()) {
    let contracts = vec![
        Contract::from_str("3NN").expect("fixed contract parses"),
        Contract::from_str("3HN").expect("fixed contract parses"),
        Contract::from_str("3DN").expect("fixed contract parses"),
        Contract::from_str("4HN").expect("fixed contract parses"),
    ];
    let dealer = DealerBuilder::new()
        .predeal(
            Seat::North,
            Cards::from_str("A AKjt Kqjt KQT9").expect("fixed hand parses"),
        )
        .expect("fixed hand is disjoint")
        .build()
        .expect("fixed dealer builds");
    let sim = ContractPayoffSimulation::new(1000, dealer, contracts, imps_against_the_opponent());
    sim.run().expect("payoff simulation runs");
}

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("Payoff benchmark", |b| {
        b.iter(|| payoff_report_test(black_box(())))
    });
}

criterion_group!(
    name = benches;
    config = Criterion::default().sample_size(10);
    targets = criterion_benchmark
);
criterion_main!(benches);
