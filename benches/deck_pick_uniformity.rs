use criterion::{Criterion, criterion_group, criterion_main};
use squeezer::{Cards, UniformSampler};
use std::collections::HashMap;

fn bench_uniformity(c: &mut Criterion) {
    let mut rng = rand::rng();
    c.bench_function("fast_sampler_uniformity_4bit", |b| {
        b.iter(|| {
            let mut counts = HashMap::new();
            for _ in 0..100_000 {
                let mut deck = Cards::ALL;
                if let Some(hand) = deck.pick_rng::<_, UniformSampler>(&mut rng, 13) {
                    let key = hand.as_bits(); // raw bit representation
                    *counts.entry(key).or_insert(0) += 1;
                }
            }

            // Simulate summary step to prevent optimizer skipping
            println!("{counts:?}");
        });
    });

    c.bench_function("uniform_sampler_uniformity_4bit", |b| {
        b.iter(|| {
            let mut counts = HashMap::new();
            for _ in 0..100_000 {
                let mut deck = Cards::ALL;
                if let Some(hand) = deck.pick_rng::<_, UniformSampler>(&mut rng, 13) {
                    let key = hand.as_bits();
                    *counts.entry(key).or_insert(0) += 1;
                }
            }

            println!("{counts:?}");
        });
    });
}
criterion_group!(benches, bench_uniformity);
criterion_main!(benches);
