use criterion::{black_box, criterion_group, criterion_main, Criterion};
use rand;
use squeezer::{Cards, FastSampler, UniformSampler};
use std::collections::HashMap;

fn bench_uniformity(c: &mut Criterion) {
    let mut rng = rand::thread_rng();
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
            let _top = counts.iter().max_by_key(|(_, &v)| v);
            println!("{:?}", counts);
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

            let top = counts.iter().max_by_key(|(_, &v)| v);
            println!("{:?}", counts);
        });
    });
}
criterion_group!(benches, bench_uniformity);
criterion_main!(benches);
