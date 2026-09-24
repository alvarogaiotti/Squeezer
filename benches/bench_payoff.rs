#![allow(clippy::all)]
use criterion::{Criterion, criterion_group, criterion_main};
use squeezer::Simulation;
use squeezer::prelude::*;
use std::hint::black_box;

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
        Contract::from_str("3NN").unwrap(),
        Contract::from_str("3HN").unwrap(),
        Contract::from_str("3DN").unwrap(),
        Contract::from_str("4HN").unwrap(),
    ];
    let dealer = DealerBuilder::new()
        .predeal(Seat::North, Cards::from_str("A AKjt Kqjt KQT9").unwrap())
        .unwrap()
        .build()
        .unwrap();
    let sim = ContractPayoffSimulation::new(1000, dealer, contracts, imps_against_the_opponent());
    sim.run().unwrap();
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
