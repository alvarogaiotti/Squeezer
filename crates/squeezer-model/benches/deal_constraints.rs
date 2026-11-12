use criterion::measurement::Measurement;
use criterion::{BenchmarkGroup, Criterion, Throughput, criterion_group, criterion_main};
use squeezer_cards::{Cards, Suit};
use squeezer_model::*;
use std::hint::black_box;
use std::str::FromStr;

fn balanced_shape() -> HandConstraint {
    HandConstraint::new(&["(4432)", "(4333)", "(5332)"], HcpRange::new(0, 37))
        .expect("balanced shape patterns parse")
}

fn opening_shape_and_hcp() -> HandConstraint {
    HandConstraint::new(&["(4432)", "(4333)", "(5332)"], HcpRange::new(12, 14))
        .expect("opening shape and HCP patterns parse")
}

fn nt_shape_and_hcp() -> HandConstraint {
    HandConstraint::new(&["(4432)", "(4333)", "(5332)"], HcpRange::new(15, 17))
        .expect("NT shape and HCP patterns parse")
}

fn bench_dealer<D: Dealer, M: Measurement>(
    group: &mut BenchmarkGroup<'_, M>,
    name: &str,
    dealer: D,
) {
    group.throughput(Throughput::Elements(1));
    group.bench_function(name, |b| {
        b.iter(|| black_box(dealer.deal().expect("constraint has a satisfying deal")));
    });
}

fn benchmark_constraint_matrix(c: &mut Criterion) {
    let mut group = c.benchmark_group("deal_creation_constraints");
    group.sample_size(10);

    let unconstrained = DealerBuilder::new().build().expect("dealer builds");
    bench_dealer(&mut group, "unconstrained", unconstrained);

    let shape = DealerBuilder::new()
        .with_hand_constraint(Seat::North, balanced_shape())
        .build()
        .expect("shape dealer builds");
    bench_dealer(&mut group, "shape_balanced", shape);

    let hcp = DealerBuilder::new()
        .with_hand_constraint(Seat::North, HandConstraint::hcp(12..=17))
        .build()
        .expect("HCP dealer builds");
    bench_dealer(&mut group, "hcp_12_17", hcp);

    let shape_hcp = DealerBuilder::new()
        .with_hand_constraint(Seat::North, opening_shape_and_hcp())
        .build()
        .expect("shape/HCP dealer builds");
    bench_dealer(&mut group, "shape_hcp_12_14", shape_hcp);

    let nt_north = DealerBuilder::new()
        .with_hand_constraint(Seat::North, nt_shape_and_hcp())
        .build()
        .expect("NT North dealer builds");
    bench_dealer(&mut group, "nt_15_17_north", nt_north);

    let nt_south = DealerBuilder::new()
        .with_hand_constraint(Seat::South, nt_shape_and_hcp())
        .build()
        .expect("NT South dealer builds");
    bench_dealer(&mut group, "nt_15_17_south", nt_south);

    let nt_west = DealerBuilder::new()
        .with_hand_constraint(Seat::West, nt_shape_and_hcp())
        .build()
        .expect("NT West dealer builds");
    bench_dealer(&mut group, "nt_15_17_west", nt_west);

    let function = DealerBuilder::new()
        .with_function(|deal: &Deal| {
            let ns_spades = deal.north().spades().len() + deal.south().spades().len();
            let ns_hearts = deal.north().hearts().len() + deal.south().hearts().len();
            ns_spades >= 8 || ns_hearts >= 8
        })
        .build()
        .expect("function dealer builds");
    bench_dealer(&mut group, "function_ns_fit", function);

    let shape_function = DealerBuilder::new()
        .with_hand_constraint(Seat::North, balanced_shape())
        .with_function(|deal: &Deal| deal.north().hcp() + deal.south().hcp() >= 25)
        .build()
        .expect("shape/function dealer builds");
    bench_dealer(&mut group, "shape_plus_function", shape_function);

    let multi_seat = DealerBuilder::new()
        .with_hand_constraints_all([
            Some(HandConstraint::hcp(10..=13)),
            None,
            Some(HandConstraint::hcp(10..=13)),
            None,
        ])
        .build()
        .expect("multi-seat dealer builds");
    bench_dealer(&mut group, "two_seat_hcp", multi_seat);

    let conditioned = DealerBuilder::new()
        .predeal(
            Seat::North,
            Cards::from_str("SAKQ").expect("predeal parses"),
        )
        .expect("predeal is disjoint")
        .with_hand_constraint(Seat::North, HandConstraint::hcp(10..=16))
        .build()
        .expect("conditioned dealer builds");
    bench_dealer(&mut group, "predeal_plus_hcp", conditioned);

    // Polish Club 1♣ opening, inlined so the benchmark stays self-contained:
    // weak balanced 11-14, unbalanced with clubs 11-17, or any strong 18+ hand.
    let weak1n = HandConstraintBuilder::new()
        .add_shape("(4432)")
        .expect("(4432) parses")
        .add_shape("(4333)")
        .expect("(4333) parses")
        .add_shape("(4414)")
        .expect("(4414) parses")
        .remove_shape("4441")
        .expect("4441 parses")
        .add_shape("(5332)")
        .expect("(5332) parses")
        .remove_shape("(5x)(2+2+)")
        .expect("(5x)(2+2+) parses")
        .with_range(11, 14)
        .build()
        .expect("weak 1NT constraint is non-empty");
    let unbal_with_clubs = HandConstraintBuilder::new()
        .with_longest(Suit::Clubs)
        .expect("clubs-longest patterns parse")
        .remove_shape("(4x)x5")
        .expect("(4x)x5 parses")
        .remove_shape("(5-5-5-)6+")
        .expect("(5-5-5-)6+ parses")
        .with_range(11, 17)
        .build()
        .expect("clubs-longest constraint is non-empty");
    let strong_any = HandConstraint::hcp(18..=37);
    let polish_club = DealerBuilder::new()
        .with_hand_constraint(
            Seat::South,
            HandConstraint::any(vec![weak1n, unbal_with_clubs, strong_any])
                .expect("Polish Club has branches"),
        )
        .build()
        .expect("Polish Club dealer builds");
    bench_dealer(&mut group, "polish_club", polish_club);

    group.finish();
}

criterion_group!(benches, benchmark_constraint_matrix);
criterion_main!(benches);
