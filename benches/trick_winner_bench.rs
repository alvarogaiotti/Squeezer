// Benchmark for trick winner finding functions
// Add this to your Cargo.toml:
//
// [dev-dependencies]
// criterion = { version = "0.5", features = ["html_reports"] }
// rand = "0.8"
//
// [[bench]]
// name = "trick_winner_bench"
// harness = false

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use rand::Rng;
use rand::rng;
use squeezer::{
    performance_analysis::{PreprocessedDeal, find_trick_winner, find_trick_winner_naive},
    prelude::*,
};
use std::hint::black_box;

/// Generate a random trick (4 cards)
fn generate_random_trick() -> [Card; 48] {
    let mut rng = rng();

    std::array::from_fn(|_| {
        let suit = match rng.random_range(0..4) {
            0 => Suit::Spades,
            1 => Suit::Hearts,
            2 => Suit::Diamonds,
            3 => Suit::Clubs,
            _ => unreachable!(),
        };
        let rank = rng.random_range(2..=14);
        Card::new(suit, rank)
    })
}

fn benchmark_trick_winner_functions(c: &mut Criterion) {
    let mut group = c.benchmark_group("trick_winner_comparison");
    group.throughput(Throughput::Elements(4)); // 4 cards per trick

    // Generate test data
    let tricks: Vec<[Card; 48]> = (0..1000).map(|_| generate_random_trick()).collect();
    let trump = Suit::Spades;
    let mut current_leader = Seat::South;

    // Benchmark the optimized version (with preprocessing)
    group.bench_function("optimized_with_preprocessing", |b| {
        b.iter(|| {
            for tricks in tricks.iter() {
                let preprocessed = PreprocessedDeal::new(tricks, Some(trump));
                let mut trick_cards = [Card::NONCE; 4];

                for trick_idx in 0..12 {
                    let card_offset = trick_idx * 4;

                    trick_cards.copy_from_slice(&tricks[card_offset..card_offset + 4]);

                    let trick_preprocessed = preprocessed.get_trick(trick_idx);
                    let lead_suit = trick_cards[0].suit();
                    //dbg!(trick_cards);
                    //dbg!(trick_preprocessed);
                    // Get DD results for this trick

                    // Find winner using SIMD-friendly comparison
                    let winner_position_in_trick = find_trick_winner(
                        black_box(trick_preprocessed),
                        black_box(trick_cards),
                        black_box(lead_suit),
                    );
                    //println!("{winner_position_in_trick}");
                    current_leader = current_leader + winner_position_in_trick;
                }
            }
        })
    });
    let mut current_leader = Seat::South;

    // Benchmark the naive version
    group.bench_function("naive", |b| {
        b.iter(|| {
            {
                for tricks in tricks.iter() {
                    let mut trick_cards = [Card::NONCE; 4];

                    for trick_idx in 0..12 {
                        let card_offset = trick_idx * 4;

                        trick_cards.copy_from_slice(&tricks[card_offset..card_offset + 4]);

                        let lead_suit = trick_cards[0].suit();
                        //dbg!(trick_cards);
                        //dbg!(trick_preprocessed);
                        // Get DD results for this trick

                        // Find winner using SIMD-friendly comparison
                        let winner_position_in_trick = find_trick_winner_naive(
                            black_box(trick_cards),
                            black_box(lead_suit),
                            black_box(Some(trump)),
                        );
                        //println!("{winner_position_in_trick}");
                        current_leader = current_leader + winner_position_in_trick;
                    }
                }
            }
        })
    });

    group.finish();
}

// fn benchmark_single_trick(c: &mut Criterion) {
//     let mut group = c.benchmark_group("single_trick");
//     group.throughput(Throughput::Elements(1));

//     let trick = generate_random_trick();
//     let trump = Suit::Spades;
//     let lead_suit = Suit::Hearts;
//     let preprocessed = preprocess_trick(trick, trump);

//     group.bench_function("optimized", |b| {
//         b.iter(|| {
//             find_trick_winner(
//                 black_box(preprocessed),
//                 black_box(trick),
//                 black_box(lead_suit),
//             )
//         })
//     });

//     group.bench_function("naive", |b| {
//         b.iter(|| find_trick_winner_naive(black_box(trick), black_box(lead_suit), black_box(trump)))
//     });

//     group.finish();
// }

// fn benchmark_different_scenarios(c: &mut Criterion) {
//     let mut group = c.benchmark_group("scenarios");

//     // Scenario 1: All trump cards
//     let all_trump = [
//         Card::new(Suit::Spades, 14),
//         Card::new(Suit::Spades, 13),
//         Card::new(Suit::Spades, 12),
//         Card::new(Suit::Spades, 11),
//     ];
//     let preprocessed_all_trump = preprocess_trick(all_trump, Suit::Spades);

//     group.bench_function("all_trump/optimized", |b| {
//         b.iter(|| {
//             find_trick_winner(
//                 black_box(preprocessed_all_trump),
//                 black_box(all_trump),
//                 black_box(Suit::Spades),
//             )
//         })
//     });

//     group.bench_function("all_trump/naive", |b| {
//         b.iter(|| {
//             find_trick_winner_naive(
//                 black_box(all_trump),
//                 black_box(Suit::Spades),
//                 black_box(Suit::Spades),
//             )
//         })
//     });

//     // Scenario 2: No trump cards
//     let no_trump = [
//         Card::new(Suit::Hearts, 14),
//         Card::new(Suit::Hearts, 13),
//         Card::new(Suit::Hearts, 12),
//         Card::new(Suit::Hearts, 11),
//     ];
//     let preprocessed_no_trump = preprocess_trick(no_trump, Suit::Spades);

//     group.bench_function("no_trump/optimized", |b| {
//         b.iter(|| {
//             find_trick_winner(
//                 black_box(preprocessed_no_trump),
//                 black_box(no_trump),
//                 black_box(Suit::Hearts),
//             )
//         })
//     });

//     group.bench_function("no_trump/naive", |b| {
//         b.iter(|| {
//             find_trick_winner_naive(
//                 black_box(no_trump),
//                 black_box(Suit::Hearts),
//                 black_box(Suit::Spades),
//             )
//         })
//     });

//     // Scenario 3: Mixed suits
//     let mixed = [
//         Card::new(Suit::Spades, 14),   // Trump ace
//         Card::new(Suit::Hearts, 14),   // Lead ace
//         Card::new(Suit::Diamonds, 14), // Off-suit ace
//         Card::new(Suit::Clubs, 2),     // Off-suit deuce
//     ];
//     let preprocessed_mixed = preprocess_trick(mixed, Suit::Spades);

//     group.bench_function("mixed/optimized", |b| {
//         b.iter(|| {
//             find_trick_winner(
//                 black_box(preprocessed_mixed),
//                 black_box(mixed),
//                 black_box(Suit::Hearts),
//             )
//         })
//     });

//     group.bench_function("mixed/naive", |b| {
//         b.iter(|| {
//             find_trick_winner_naive(
//                 black_box(mixed),
//                 black_box(Suit::Hearts),
//                 black_box(Suit::Spades),
//             )
//         })
//     });

//     group.finish();
// }

// fn benchmark_full_deal(c: &mut Criterion) {
//     let mut group = c.benchmark_group("full_deal");
//     group.throughput(Throughput::Elements(12)); // 12 tricks per deal

//     // Generate 12 tricks (a full deal)
//     let tricks: Vec<[Card; 4]> = (0..12).map(|_| generate_random_trick()).collect();
//     let trump = Suit::Spades;
//     let lead_suits: Vec<Suit> = tricks.iter().map(|t| t[0].suit()).collect();

//     // Preprocess all tricks
//     let preprocessed: Vec<[u8; 4]> = tricks.iter().map(|&t| preprocess_trick(t, trump)).collect();

//     group.bench_function("optimized_12_tricks", |b| {
//         b.iter(|| {
//             for i in 0..12 {
//                 let _ = find_trick_winner(
//                     black_box(preprocessed[i]),
//                     black_box(tricks[i]),
//                     black_box(lead_suits[i]),
//                 );
//             }
//         })
//     });

//     group.bench_function("naive_12_tricks", |b| {
//         b.iter(|| {
//             for i in 0..12 {
//                 let _ = find_trick_winner_naive(
//                     black_box(tricks[i]),
//                     black_box(lead_suits[i]),
//                     black_box(trump),
//                 );
//             }
//         })
//     });

//     group.finish();
// }

// fn benchmark_correctness_check(c: &mut Criterion) {
//     // This is not a benchmark, but a correctness check
//     // Run this first to ensure both implementations give the same results

//     println!("\n=== Correctness Check ===");
//     let mut rng = rand::thread_rng();
//     let trump = Suit::Spades;

//     for _ in 0..1000 {
//         let trick = generate_random_trick();
//         let lead_suit = trick[0].suit();
//         let preprocessed = preprocess_trick(trick, trump);

//         let result_optimized = find_trick_winner(preprocessed, trick, lead_suit);
//         let result_naive = find_trick_winner_naive(trick, lead_suit, trump);

//         if result_optimized != result_naive {
//             panic!(
//                 "Mismatch! Optimized: {}, Naive: {}\nTrick: {:?}\nLead: {:?}, Trump: {:?}",
//                 result_optimized, result_naive, trick, lead_suit, trump
//             );
//         }
//     }

//     println!("✓ Both implementations agree on 1000 random tricks");
//     println!("========================\n");
// }

criterion_group!(
    benches,
    // benchmark_correctness_check,
    // benchmark_single_trick,
    benchmark_trick_winner_functions,
    // benchmark_different_scenarios,
    // benchmark_full_deal,
);
criterion_main!(benches);
