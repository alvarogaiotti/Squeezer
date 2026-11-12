#![allow(clippy::all)]
use criterion::{Criterion, criterion_group, criterion_main};
use squeezer_cards::*;
use squeezer_model::*;
use squeezer_sim::simulation::*;
use std::hint::black_box;
use std::str::FromStr;

#[inline]
fn lead_analysis_test(_: ()) {
    let hand = Cards::from_str("AQT KQ732 432 43").expect("fixed hand parses");
    let dealer = DealerBuilder::new()
        .predeal(Seat::South, hand)
        .expect("fixed hand is disjoint")
        .with_function(|cards| {
            let east = cards.east();
            let west = cards.west();
            east.len_of_suit(Suit::Hearts) + west.len_of_suit(Suit::Hearts) == 8
                && east.hcp() + west.hcp() >= 20
                && east.hcp() + west.hcp() <= 24
        })
        .build()
        .expect("fixed dealer builds");
    let contract =
        Contract::from_str_and_vuln("2HE", Vulnerable::No).expect("fixed contract parses");
    let analysis = LeadSimulation::new(1000, dealer, contract)
        .run()
        .expect("lead analysis runs");
    // Read the view the way the terminal formatter does, so the benchmark
    // covers the ranking and per-suit derivations, not just the solve.
    black_box(analysis.rankings());
    black_box(analysis.analyze_all_suits().len());
    black_box(analysis.best_by_set_percentage().map(|value| value.card));
}

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("Lead simulation benchmark", |b| {
        b.iter(|| lead_analysis_test(black_box(())))
    });
}

criterion_group!(
    name = benches;
    config = Criterion::default().sample_size(10);
    targets = criterion_benchmark
);
criterion_main!(benches);
