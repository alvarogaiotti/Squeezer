use criterion::{Criterion, criterion_group, criterion_main};
use squeezer::{
    Cards, Dealer, DealerBuilder, HandConstraint, Seat, Shape, polish_club_hand_constraint,
};
use std::str::FromStr;

fn bench_descriptor_speed(c: &mut Criterion) {
    let mut group = c.benchmark_group("hand descriptor speed");
    let dealer = DealerBuilder::new()
        .with_hand_constraint(Seat::South, polish_club_hand_constraint().unwrap())
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

/// Profiles the Q3 deepening decision: the legacy `Shape::All` enum dispatch
/// (branch-and-return-true) against the deepened full-table lookup used by
/// `HandConstraint::all`. Guards against regressions from folding `All` away.
fn bench_shape_all(c: &mut Criterion) {
    let mut group = c.benchmark_group("shape all vs full table");
    let hands: Vec<Cards> = (0..1024)
        .map(|_| {
            let mut deck = Cards::new_deck();
            deck.pick(13).expect("a full deck holds thirteen cards")
        })
        .collect();
    let full_table = HandConstraint::all();
    let enum_dispatch = Shape::All;

    group.bench_function("Shape::All enum dispatch", |b| {
        b.iter(|| {
            for hand in &hands {
                std::hint::black_box(enum_dispatch.is_member(*hand));
            }
        });
    });

    group.bench_function("Shapes full table", |b| {
        b.iter(|| {
            for hand in &hands {
                std::hint::black_box(full_table.check(*hand));
            }
        });
    });

    group.finish();
}

fn bench_constraint_from_str(c: &mut Criterion) {
    use squeezer::HandConstraintBuilder;
    let hand = Cards::from_str("AKQJ432.32.32.32").unwrap();
    let constraint = HandConstraintBuilder::new()
        .with_longest(squeezer::Suit::Spades)
        .unwrap()
        .with_range(5, 10)
        .build()
        .unwrap();
    c.bench_function("hand constraint check", |b| {
        b.iter(|| std::hint::black_box(constraint.check(hand)));
    });
}

criterion_group!(
    benches,
    bench_descriptor_speed,
    bench_shape_all,
    bench_constraint_from_str
);
criterion_main!(benches);
