use criterion::{Criterion, Throughput, criterion_group, criterion_main};
use dds::deal::DdsDeal;
use squeezer::prelude::*;
use std::hint::black_box;

fn example_position() -> Position {
    let deal = Deal::new([
        Hand::from_str("SQJ6 HK652 DJ85 CT98").expect("north hand parses"),
        Hand::from_str("S873 HJ97 DAT764 CQ4").expect("east hand parses"),
        Hand::from_str("SK5 HT83 DKQ9 CA7652").expect("south hand parses"),
        Hand::from_str("SAT942 HAQ4 D32 CKJ3").expect("west hand parses"),
    ])
    .expect("example hands form a deal");
    let contract = Contract::new(
        1,
        Strain::Spades,
        Seat::West,
        Vulnerable::No,
        Doubled::NotDoubled,
    );

    Position::from_deal_prefix_with_contract(&deal, &[], contract)
        .expect("opening position is valid")
}

fn benchmark_dds_conversion(c: &mut Criterion) {
    let position = example_position();
    let mut group = c.benchmark_group("dds_conversion");
    group.throughput(Throughput::Elements(1));

    group.bench_function("position_to_dds_deal", |b| {
        b.iter(|| {
            black_box(DdsDeal::try_from(black_box(&position)).expect("position converts to DDS"));
        });
    });

    group.finish();
}

criterion_group!(benches, benchmark_dds_conversion);
criterion_main!(benches);
