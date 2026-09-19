// Benchmark for trick winner finding functions.
use criterion::{Criterion, Throughput, criterion_group, criterion_main};
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
criterion_group!(benches, benchmark_trick_winner_functions);
criterion_main!(benches);
