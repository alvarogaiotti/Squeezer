// Benchmark for SIMD vs scalar preprocessing.
use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use dds::analyseplay::SolvedPlay;
use rand::{Rng, rng};
use squeezer::performance_analysis::AnalysisInput;
use squeezer::performance_analysis::PreprocessedDeal;
use squeezer::performance_analysis::analyze_hand;
use squeezer::performance_analysis::find_trick_winner;
use squeezer::prelude::*;
use std::hint::black_box;

fn generate_random_cards() -> [Card; 48] {
    let mut rng = rng();
    let mut cards = [Card::new(Suit::Spades, 2); 48];

    for card in &mut cards {
        let suit = match rng.random_range(0..4) {
            0 => Suit::Spades,
            1 => Suit::Hearts,
            2 => Suit::Diamonds,
            3 => Suit::Clubs,
            _ => unreachable!(),
        };
        let rank = rng.random_range(2..=14);
        *card = Card::new(suit, rank);
    }

    cards
}

fn benchmark_preprocessing(c: &mut Criterion) {
    let mut group = c.benchmark_group("preprocessing");
    group.throughput(Throughput::Elements(48)); // 48 cards per operation

    let cards = generate_random_cards();
    let trump = Some(Suit::Spades);

    // Benchmark scalar version
    group.bench_function("scalar", |b| {
        b.iter(|| PreprocessedDeal::new_scalar(black_box(&cards), black_box(trump)))
    });

    // Benchmark SSE2 version (x86_64 only)
    #[cfg(target_arch = "x86_64")]
    {
        if is_x86_feature_detected!("sse2") {
            group.bench_function("sse2", |b| {
                b.iter(|| unsafe {
                    PreprocessedDeal::new_sse2(black_box(&cards), black_box(trump))
                })
            });
        }

        if is_x86_feature_detected!("avx2") {
            group.bench_function("avx2", |b| {
                b.iter(|| unsafe {
                    PreprocessedDeal::new_avx2(black_box(&cards), black_box(trump))
                })
            });
        }
    }

    // No working NEON preprocessing exists (the constructor did not
    // compile); benchmark the scalar path on ARM so the suite measures.
    #[cfg(target_arch = "aarch64")]
    {
        group.bench_function("neon (scalar fallback)", |b| {
            b.iter(|| PreprocessedDeal::new_scalar(black_box(&cards), black_box(trump)))
        });
    }

    // Benchmark the auto-selecting version
    group.bench_function("auto", |b| {
        b.iter(|| PreprocessedDeal::new(black_box(&cards), black_box(trump)))
    });

    group.finish();
}

fn benchmark_trick_winner(c: &mut Criterion) {
    let mut group = c.benchmark_group("trick_winner");
    group.throughput(Throughput::Elements(4)); // 4 cards per trick

    let cards = generate_random_cards();
    let trump = Some(Suit::Spades);

    let preprocessed = PreprocessedDeal::new(&cards, trump);
    let trick_cards = [cards[0], cards[1], cards[2], cards[3]];
    let trick_preprocessed = preprocessed.get_trick(0);
    let lead_suit = trick_cards[0].suit();

    group.bench_function("find_winner", |b| {
        b.iter(|| {
            find_trick_winner(
                black_box(trick_preprocessed),
                black_box(trick_cards),
                black_box(lead_suit),
            )
        })
    });

    group.finish();
}

fn benchmark_full_hand_analysis(c: &mut Criterion) {
    let mut group = c.benchmark_group("full_analysis");
    group.throughput(Throughput::Elements(12)); // 12 tricks per hand

    // Generate a complete hand with mock data
    let cards = generate_random_cards();

    // `analyze_hand` only reads the cards, so the seat attribution is arbitrary.
    let play_sequence = PlayRecord::new(
        cards
            .iter()
            .enumerate()
            .map(|(index, &card)| Play::new(Seat::from(index), card))
            .collect(),
    );

    // Create mock DD results (initial state + 4 results per trick = 49 values)
    let mut dd_results = Vec::with_capacity(49);
    dd_results.push(10); // Initial state
    for _ in 0..12 {
        // 4 results per trick, gradually decreasing
        dd_results.extend_from_slice(&[10, 10, 9, 9]);
    }
    let dd_solved = SolvedPlay::from_seq(dd_results);

    let contract = Contract::new(
        1,
        Strain::Spades,
        Seat::South,
        Vulnerable::No,
        Doubled::NotDoubled,
    );

    let input = AnalysisInput {
        contract,
        play_sequence,
        dd_results: dd_solved,
    };

    group.bench_function("complete_hand", |b| {
        b.iter(|| analyze_hand(black_box(input.clone())))
    });

    group.finish();
}

fn benchmark_batch_processing(c: &mut Criterion) {
    let mut group = c.benchmark_group("batch_processing");

    // Benchmark preprocessing multiple deals
    for num_deals in [1, 10, 100, 1000].iter() {
        group.throughput(Throughput::Elements(*num_deals * 48));

        let deals: Vec<[Card; 48]> = (0..*num_deals).map(|_| generate_random_cards()).collect();

        group.bench_with_input(
            BenchmarkId::from_parameter(format!("{}_deals", num_deals)),
            num_deals,
            |b, &_| {
                b.iter(|| {
                    for deal in &deals {
                        let _ =
                            PreprocessedDeal::new(black_box(deal), black_box(Some(Suit::Spades)));
                    }
                })
            },
        );
    }

    group.finish();
}

fn benchmark_comparison_overhead(c: &mut Criterion) {
    let mut group = c.benchmark_group("comparison_methods");
    group.throughput(Throughput::Elements(4));

    let cards = generate_random_cards();
    let trump = Some(Suit::Spades);
    let preprocessed = PreprocessedDeal::new(&cards, trump);
    let trick_preprocessed = preprocessed.get_trick(0);
    let trick_cards = [cards[0], cards[1], cards[2], cards[3]];
    let lead_suit = trick_cards[0].suit();

    // Test current branchless implementation
    group.bench_function("branchless", |b| {
        b.iter(|| {
            find_trick_winner(
                black_box(trick_preprocessed),
                black_box(trick_cards),
                black_box(lead_suit),
            )
        })
    });

    // Test naive branching version for comparison
    group.bench_function("branching", |b| {
        b.iter(|| {
            find_trick_winner_branching(
                black_box(&trick_preprocessed),
                black_box(&trick_cards),
                black_box(lead_suit),
            )
        })
    });

    group.finish();
}

// Naive branching version for comparison
fn find_trick_winner_branching(
    preprocessed: &[u8; 4],
    original_cards: &[Card; 4],
    lead_suit: Suit,
) -> usize {
    let mut strengths = [0u8; 4];

    for i in 0..4 {
        let base = preprocessed[i];
        let suit = original_cards[i].suit();

        // Branching version
        strengths[i] = if base >= 128 {
            base // Already trump
        } else if suit == lead_suit {
            base + 64 // Lead suit
        } else {
            0 // Off-suit
        };
    }

    strengths
        .iter()
        .enumerate()
        .max_by_key(|(_, s)| *s)
        .map(|(idx, _)| idx)
        .unwrap()
}

criterion_group!(
    benches,
    benchmark_preprocessing,
    benchmark_trick_winner,
    benchmark_full_hand_analysis,
    benchmark_batch_processing,
    benchmark_comparison_overhead,
);
criterion_main!(benches);
