//! Every sampler draws each fixed-size subset with equal probability.
//!
//! A full deck has C(52, 13) ≈ 6.35e11 thirteen-card hands, far too many to
//! bin: any test that tried would compare near-empty bins and detect nothing.
//! Two complementary checks therefore cover the samplers:
//!
//! - a census on a deck small enough that every subset owns one bin (four
//!   cards out of eight, 70 bins), which tests the whole distribution, and
//! - a full-deck marginal check: every card must be dealt equally often over
//!   many thirteen-card hands, which exercises the production draw size even
//!   though the hands themselves cannot be binned.
//!
//! Sampling is random, so the tests snapshot one seeded RNG stream rather than
//! resampling: with a fixed seed the chi-square statistic is reproducible, and
//! a fair sampler lands far below the critical value. Measured about 2.2 s
//! wall-clock for the whole file with the default parallel test harness in a
//! debug build; each test alone takes 2.1 s (subset census), 0.9 s (full-deck
//! marginals) and 0.2 s (biased-sampler power check).

use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use squeezer_cards::{Card, CardSampler, Cards, Sampler};

/// Eight distinct cards, so a four-card hand is one of C(8, 4) = 70 subsets.
const DECK: [Card; 8] = [
    Card::S2,
    Card::S3,
    Card::S4,
    Card::S5,
    Card::H2,
    Card::H3,
    Card::H4,
    Card::H5,
];
const DRAW: usize = 4;
const BINS: usize = 70;
const DRAWS: usize = 1_000_000;

/// Chi-square with 69 degrees of freedom has its 0.999 quantile at about 111;
/// 130 sits near the 1e-5 quantile. At one million draws each bin expects
/// 14,286 hands, so a single bin that is 10% over-represented contributes
/// about 143 — over the threshold — while a fair sampler stays near 69.
const SUBSET_THRESHOLD: f64 = 130.0;

/// Thirteen-card hands the full-deck marginal check draws.
const FULL_DECK_DRAWS: usize = 200_000;

/// Chi-square with 51 degrees of freedom (one per card) has its 0.999
/// quantile at about 88; a fair sampler over these draws stays near 51.
const FULL_DECK_THRESHOLD: f64 = 110.0;

fn deck() -> Cards {
    DECK.iter()
        .fold(Cards::EMPTY, |deck, card| deck.insert(*card))
}

/// The bin a hand belongs to: one bit per deck card, so a hand is a mask.
fn bin(hand: Cards) -> usize {
    DECK.iter().enumerate().fold(0, |mask, (index, card)| {
        if hand.has_card(*card) {
            mask | (1 << index)
        } else {
            mask
        }
    })
}

/// The sum over bins of `(observed - expected)^2 / expected`.
fn chi_square(counts: impl Iterator<Item = u64>, expected: f64) -> f64 {
    counts
        .map(|count| {
            let deviation = count as f64 - expected;
            deviation * deviation / expected
        })
        .sum()
}

/// Draws `DRAWS` hands with `draw` and returns their chi-square statistic
/// against the uniform distribution over all `DRAW`-card subsets.
fn subset_chi_square(mut draw: impl FnMut(&mut Cards, &mut StdRng) -> Option<Cards>) -> f64 {
    let mut rng = StdRng::seed_from_u64(0xC0FFEE);
    let mut counts = [0_u64; 1 << DECK.len()];

    for _ in 0..DRAWS {
        let mut deck = deck();
        let hand = draw(&mut deck, &mut rng).expect("a full deck holds the draw");
        assert_eq!(hand.len() as usize, DRAW, "sampler returned the wrong size");
        assert_eq!(deck.len() as usize, DECK.len() - DRAW);
        counts[bin(hand)] += 1;
    }

    // Only masks with exactly `DRAW` bits name a real subset; skipping the
    // rest keeps the expected count per bin uniform.
    chi_square(
        counts
            .iter()
            .enumerate()
            .filter(|(mask, _)| (*mask as u32).count_ones() as usize == DRAW)
            .map(|(_, &count)| count),
        DRAWS as f64 / BINS as f64,
    )
}

/// A sampler that always takes the lowest cards: the test must reject it.
struct LowestCards;

impl CardSampler for LowestCards {
    fn pick_n<R: Rng>(deck: &mut Cards, num: usize, _rng: &mut R) -> Option<Cards> {
        let hand = deck.into_iter().rev().take(num).collect::<Vec<_>>();
        let hand = Cards::from(&hand);
        *deck = deck.difference(hand);
        Some(hand)
    }
}

#[test]
fn both_samplers_draw_uniform_subsets() {
    for sampler in [Sampler::Fast, Sampler::Uniform] {
        let statistic = subset_chi_square(|deck, rng| deck.pick_rng_with(rng, DRAW, sampler));
        assert!(
            statistic < SUBSET_THRESHOLD,
            "{sampler:?} is not uniform: chi-square {statistic:.2} exceeds {SUBSET_THRESHOLD} over {BINS} bins"
        );
    }
}

#[test]
fn a_full_deck_deals_every_card_equally_often() {
    for sampler in [Sampler::Fast, Sampler::Uniform] {
        let mut rng = StdRng::seed_from_u64(0xDEAD_BEEF);
        // Card offsets are packed as `rank + 16 * suit`, so the highest card
        // (ace of clubs) sits at 62; 64 slots keep the lookup total.
        let mut counts = [0_u64; 64];

        for _ in 0..FULL_DECK_DRAWS {
            let mut deck = Cards::ALL;
            let hand = deck
                .pick_rng_with(&mut rng, 13, sampler)
                .expect("a full deck holds 13 cards");
            assert_eq!(hand.len(), 13, "{sampler:?}");
            for card in hand {
                counts[card.as_u8() as usize] += 1;
            }
        }

        // One bin per card, in the same offset order `Card::as_u8` uses.
        let statistic = chi_square(
            Cards::ALL
                .into_iter()
                .map(|card| counts[card.as_u8() as usize]),
            FULL_DECK_DRAWS as f64 * 13.0 / 52.0,
        );
        assert!(
            statistic < FULL_DECK_THRESHOLD,
            "{sampler:?} favours some cards: chi-square {statistic:.2} exceeds {FULL_DECK_THRESHOLD} over 52 cards"
        );
    }
}

#[test]
fn the_uniformity_check_rejects_a_biased_sampler() {
    // Without this, a test that never fails would look like a passing check.
    let statistic = subset_chi_square(|deck, rng| LowestCards::pick_n(deck, DRAW, rng));
    assert!(
        statistic > SUBSET_THRESHOLD,
        "the biased sampler slipped through: chi-square {statistic:.2}"
    );
}
