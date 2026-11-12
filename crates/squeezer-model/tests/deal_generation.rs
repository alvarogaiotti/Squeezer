// Copyright (C) 2024 Alvaro Gaiotti
// See end of file for license information

use squeezer_cards::{Cards, Suit};
use squeezer_model::*;
use std::str::FromStr;
use std::sync::{Arc, Mutex};

#[test]
fn deal_generation() {
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
    for _ in 0..10 {
        let deal = dealer.deal().expect("constraint has a satisfying deal");
        // The constraints actually filter: south holds 18-20 HCP with 6+
        // diamonds, west holds 8-15 HCP with spades longest but no 7+ spades.
        assert!((18..=20).contains(&deal.south().hcp()));
        assert!(deal.south().diamonds().len() >= 6);
        assert!((8..=15).contains(&deal.west().hcp()));
        let west = deal.west();
        assert!(west.spades().len() >= west.hearts().len());
        assert!(west.spades().len() >= west.diamonds().len());
        assert!(west.spades().len() >= west.clubs().len());
        assert!(west.spades().len() < 7);
    }
}

/// The accept closure must only ever see candidates that passed every per-seat
/// constraint: a candidate whose seat failed is discarded before the closure
/// runs, so it cannot be recorded (or accepted).
#[test]
fn accept_function_only_sees_candidates_that_passed_the_seat_constraints() {
    let recorded = Arc::new(Mutex::new(Vec::new()));
    let sink = Arc::clone(&recorded);
    let dealer = DealerBuilder::new()
        .with_hand_constraint(Seat::North, HandConstraint::hcp(15..=17))
        .with_function(move |deal: &Deal| {
            let mut recorded = sink.lock().expect("recording lock is not poisoned");
            recorded.push(*deal);
            true
        })
        .build()
        .expect("dealer builds");

    // 15-17 HCP is the minority of hands, so the constraint rejects most
    // candidates and the closure would see plenty of failures if it ran first.
    for _ in 0..50 {
        dealer
            .deal()
            .expect("15-17 HCP North has a satisfying deal");
    }

    let seen = recorded.lock().expect("recording lock is not poisoned");
    assert_eq!(seen.len(), 50, "the closure runs once per produced deal");
    for deal in seen.iter() {
        assert!(
            (15..=17).contains(&deal.north().hcp()),
            "accept closure saw a candidate whose North constraint failed: {} HCP",
            deal.north().hcp()
        );
    }
}

/// An early (North) and a late (West) seat constraint are both enforced, and a
/// predealt seat keeps its cards and its constraint. Ten thousand deals is
/// enough that a skipped late seat would surface on the first few.
#[test]
fn early_late_and_predealt_seat_constraints_are_enforced() {
    let predeal = Cards::from_str("SAKQ").expect("predeal parses");
    let dealer = DealerBuilder::new()
        .predeal(Seat::North, predeal)
        .expect("predeal is disjoint")
        .with_hand_constraint(Seat::North, HandConstraint::hcp(10..=16))
        .with_hand_constraint(Seat::West, HandConstraint::hcp(12..=17))
        .build()
        .expect("dealer builds");

    for _ in 0..10_000 {
        let deal = dealer
            .deal()
            .expect("North and West constraints have a satisfying deal");
        assert!(
            deal.north().contains(predeal),
            "predealt North cards were not kept: {}",
            deal.north()
        );
        assert!(
            (10..=16).contains(&deal.north().hcp()),
            "North constraint was not enforced: {} HCP",
            deal.north().hcp()
        );
        assert!(
            (12..=17).contains(&deal.west().hcp()),
            "West (late seat) constraint was not enforced: {} HCP",
            deal.west().hcp()
        );
    }
}
