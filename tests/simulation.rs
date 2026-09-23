// Copyright (C) 2024 Alvaro Gaiotti
// See end of file for license information

use squeezer::*;

#[test]
fn simulation() {
    let south_specs =
        HandConstraint::new(&["3262", "2362", "2263", "2272"], HcpRange::new(18, 20)).unwrap();
    let west_spec = HandConstraintBuilder::new()
        .with_longest(Suit::Spades)
        .unwrap()
        .with_range(8, 15)
        .remove_shape("7xxx")
        .unwrap()
        .remove_shape("8xxx")
        .unwrap()
        .remove_shape("9xxx")
        .unwrap()
        .build()
        .unwrap();
    let predeal = Cards::from_str("SQ63H542DK42").unwrap();
    let dealer = DealerBuilder::new()
        .predeal(Seat::North, predeal)
        .unwrap()
        .with_hand_constraint(Seat::South, south_specs)
        .with_hand_constraint(Seat::West, west_spec)
        .with_function(|deal: &Deal| deal.south().spades().high_card_points() > 2)
        .build()
        .unwrap();
    let deal = dealer.deal().unwrap();
    println!("{deal}");
    assert!(deal.south().diamonds().len() >= 6);
    assert!((18..=20).contains(&deal.south().hcp()));
}
