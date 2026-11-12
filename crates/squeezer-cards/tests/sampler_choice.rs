//! Callers choose which sampler draws the cards.
//!
//! The samplers answer the same question — draw `num` cards out of a deck —
//! so every choice must honour the same contract: exactly `num` cards leave,
//! the deck shrinks by that many, and the two sets partition the deck.

use rand::SeedableRng;
use rand::rngs::StdRng;
use squeezer_cards::{Card, CardSampler, Cards, FastSampler, Sampler, UniformSampler};

/// Both samplers, so a test can state the contract once for each.
const SAMPLERS: [Sampler; 2] = [Sampler::Fast, Sampler::Uniform];

#[test]
fn the_default_sampler_is_the_fast_one() {
    // `Cards::pick` is the shorthand for the fast sampler, so the enum's
    // default must name the same choice or the two spellings would diverge.
    assert_eq!(Sampler::default(), Sampler::Fast);
}

#[test]
fn the_shorthand_pick_still_draws_a_hand() {
    // One-argument callers keep working; the loop above covers the full
    // partition contract for the same sampler.
    let mut deck = Cards::ALL;
    let hand = deck.pick(13).expect("a full deck holds 13 cards");
    assert_eq!(hand.len(), 13);
    assert_eq!(deck.len(), 39);
}

#[test]
#[should_panic(expected = "cannot draw 53 cards from a 52-card deck")]
fn a_draw_past_the_deck_size_panics() {
    // `num > 52` is a caller bug, not a deck that happens to be short, so the
    // documented panic carries a message that names the mistake.
    let mut deck = Cards::ALL;
    let _ = deck.pick_with(53, Sampler::Uniform);
}

#[test]
fn every_sampler_draws_the_requested_cards_and_leaves_the_rest() {
    for sampler in SAMPLERS {
        let mut deck = Cards::ALL;
        let hand = deck
            .pick_with(13, sampler)
            .unwrap_or_else(|| panic!("a full deck holds 13 cards for {sampler:?}"));

        assert_eq!(hand.len(), 13, "{sampler:?}");
        assert_eq!(deck.len(), 39, "{sampler:?}");
        // The hand and the deck must partition the original deck: no card is
        // duplicated and none is lost.
        assert_eq!(hand.difference(deck), hand, "{sampler:?}");
        assert_eq!(deck.difference(hand), deck, "{sampler:?}");
        assert_eq!(hand.union(deck), Cards::ALL, "{sampler:?}");
    }
}

#[test]
fn every_sampler_can_take_the_whole_deck() {
    for sampler in SAMPLERS {
        let mut deck = Cards::ALL;
        let hand = deck
            .pick_with(52, sampler)
            .unwrap_or_else(|| panic!("a full deck holds 52 cards for {sampler:?}"));

        assert_eq!(hand, Cards::ALL, "{sampler:?}");
        assert!(deck.is_empty(), "{sampler:?}");
    }
}

#[test]
fn every_sampler_refuses_a_draw_larger_than_the_deck() {
    // `pick` promises `None` (not a panic) whenever the deck is too small; the
    // panic is reserved for `num >= 53`, which no deck can satisfy.
    for sampler in SAMPLERS {
        let mut short = Cards::EMPTY
            .insert(Card::SA)
            .insert(Card::HA)
            .insert(Card::DA);
        assert!(short.pick_with(4, sampler).is_none(), "{sampler:?}");
        assert_eq!(short.len(), 3, "{sampler:?} must not consume cards");

        let mut empty = Cards::EMPTY;
        assert!(empty.pick_with(1, sampler).is_none(), "{sampler:?}");
    }
}

#[test]
fn the_enum_routes_to_the_same_sampler_as_the_type_parameter() {
    assert_routing_matches::<FastSampler>(Sampler::Fast);
    assert_routing_matches::<UniformSampler>(Sampler::Uniform);
}

/// The run-time variant and the compile-time sampler must do exactly the same
/// thing, so with the same seed they produce the same hand and the same
/// remaining deck.
fn assert_routing_matches<S: CardSampler>(sampler: Sampler) {
    let mut enum_rng = StdRng::seed_from_u64(0x5EED);
    let mut deck = Cards::ALL;
    let via_enum = deck.pick_rng_with(&mut enum_rng, 13, sampler);

    let mut type_rng = StdRng::seed_from_u64(0x5EED);
    let mut type_deck = Cards::ALL;
    let via_type = type_deck.pick_rng::<_, S>(&mut type_rng, 13);

    assert_eq!(via_enum, via_type, "{sampler:?}");
    assert_eq!(deck, type_deck, "{sampler:?}");
}
