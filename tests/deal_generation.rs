// Copyright (C) 2024 Alvaro Gaiotti
// See end of file for license information

use squeezer::*;

#[test]
fn main_to_be() {
    let south_shapes = Shape::new_from_patterns(&["3262", "2362", "2263", "2272"]).unwrap();

    let south_specs = HandDescriptor::new(vec![HandType::new(south_shapes, HcpRange::new(18, 20))]);
    let mut hand_builder = HandTypeBuilder::new();
    hand_builder
        .with_longest(Suit::Spades)
        .with_range(8, 15)
        .remove_shape("7xxx")
        .and_then(|buildr| {
            buildr
                .remove_shape("8xxx")
                .and_then(|buildr| buildr.remove_shape("9xxx"))
        })
        .unwrap();
    let west_spec = hand_builder.build();
    let predeal = Cards::from_str("SQ63H542DK42").unwrap();
    let dealer = DealerBuilder::new()
        .predeal(Seat::North, predeal)
        .unwrap()
        .with_hand_descriptor(Seat::South, south_specs)
        .with_hand_descriptor(Seat::West, HandDescriptor::new(vec![west_spec]))
        .with_function(|hands: &Hands| hands.south().spades().high_card_points() > 2)
        .build()
        .unwrap();
    let deal = dealer.deal().unwrap();
    println!("{deal}");
    assert!(deal.south().diamonds().len() >= 6);
}
