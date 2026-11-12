// Copyright (C) 2024 Alvaro Gaiotti
// See end of file for license information

use squeezer_cards::{Cards, Suit};
use squeezer_model::*;
use std::str::FromStr;

/// Smoke test for a constrained dealer: a predeal, per-seat shape and HCP
/// constraints, and a user predicate must all agree on the produced deal.
#[test]
fn constrained_dealer_produces_a_matching_deal() {
    let south_specs = HandConstraint::new(&["3262", "2362", "2263", "2272"], HcpRange::new(18, 20))
        .expect("South shape and HCP pattern parses");
    let west_spec = HandConstraintBuilder::new()
        .with_longest(Suit::Spades)
        .expect("spades-longest patterns parse")
        .with_range(8, 15)
        .remove_shape("7xxx")
        .expect("7xxx parses")
        .remove_shape("8xxx")
        .expect("8xxx parses")
        .remove_shape("9xxx")
        .expect("9xxx parses")
        .build()
        .expect("West constraint is non-empty");
    let predeal = Cards::from_str("SQ63H542DK42").expect("predeal parses");
    let dealer = DealerBuilder::new()
        .predeal(Seat::North, predeal)
        .expect("predeal is disjoint")
        .with_hand_constraint(Seat::South, south_specs)
        .with_hand_constraint(Seat::West, west_spec)
        .with_function(|deal: &Deal| deal.south().spades().high_card_points() > 2)
        .build()
        .expect("dealer builds");
    let deal = dealer.deal().expect("constraint has a satisfying deal");
    println!("{deal}");
    assert!(deal.south().diamonds().len() >= 6);
    assert!((18..=20).contains(&deal.south().hcp()));
}
