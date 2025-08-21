use criterion::{Criterion, criterion_group, criterion_main};
use squeezer::{Cards, FastSampler, UniformSampler};

fn bench_speed(c: &mut Criterion) {
    let mut group = c.benchmark_group("sampler_speed");
    let mut rng = rand::rng();
    let num = 13;

    group.bench_function("fast_sampler_draw_13", |b| {
        b.iter(|| {
            let mut deck = Cards::ALL;
            deck.pick_rng::<_, FastSampler>(&mut rng, num)
        });
    });

    group.bench_function("uniform_sampler_draw_13", |b| {
        b.iter(|| {
            let mut deck = Cards::ALL;
            deck.pick_rng::<_, UniformSampler>(&mut rng, num)
        });
    });

    group.finish();
}

criterion_group!(benches, bench_speed);
criterion_main!(benches);
