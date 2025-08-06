use criterion::{criterion_group, criterion_main, Criterion};
use squeezer::{polish_club_hand_descriptor, Dealer, DealerBuilder, Seat};

fn bench_descriptor_speed(c: &mut Criterion) {
    let mut group = c.benchmark_group("hand descriptor speed");
    let dealer = DealerBuilder::new()
        .with_hand_descriptor(Seat::South, polish_club_hand_descriptor())
        .build()
        .unwrap();

    group.bench_function("hand descriptor speed benchmark", |b| {
        b.iter(|| {
            for _ in 0..100_000 {
                dealer.deal().unwrap();
            }
        });
    });

    group.finish();
}

criterion_group!(benches, bench_descriptor_speed);
criterion_main!(benches);
