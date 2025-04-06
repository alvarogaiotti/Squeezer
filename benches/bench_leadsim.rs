#![allow(clippy::all)]
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use squeezer::prelude::*;
use squeezer::Simulation;

#[inline]
fn lead_simulation_test(_: ()) {
    let hand = Cards::from_str("AQT KQ732 432 43").unwrap();
    let dealer = DealerBuilder::new()
        .predeal(Seat::South, hand)
        .unwrap()
        .with_function(|cards| {
            let east = cards.east();
            let west = cards.west();
            east.hlen() + west.hlen() == 8
                && east.hcp() + west.hcp() >= 20
                && east.hcp() + west.hcp() <= 24
        })
        .build()
        .unwrap();
    let contract = Contract::from_str("2HE", Vulnerable::No).unwrap();
    let simulation = LeadSimulation::new(1001, dealer, contract);
    let result = simulation.run().unwrap();
}

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("Payoff benchmark", |b| {
        b.iter(|| lead_simulation_test(black_box(())))
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
