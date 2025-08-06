#![allow(clippy::all)]
use criterion::{criterion_group, criterion_main, Criterion};
use squeezer::prelude::*;
use squeezer::Simulation;
use std::hint::black_box;

#[inline]
fn payoff_report_test(_: ()) {
    let contracts = vec![
        Contract::from_str("3NN", Vulnerable::No).unwrap(),
        Contract::from_str("3HN", Vulnerable::No).unwrap(),
        Contract::from_str("3DN", Vulnerable::No).unwrap(),
        Contract::from_str("4HN", Vulnerable::No).unwrap(),
    ];
    let dealer = DealerBuilder::new()
        .predeal(Seat::North, Cards::from_str("A AKjt Kqjt KQT9").unwrap())
        .unwrap()
        .build()
        .unwrap();
    let sim = PayoffSimulation::new(100, dealer, contracts, imps);
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
