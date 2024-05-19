use squeezer::*;

#[test]
fn main_to_be() {
    let predeal = Hand::from_str("SQ63H542DK42C8763").unwrap();
    let mut south_shapes = Shapes::new();
    south_shapes.add_shape("3262").unwrap();
    south_shapes.add_shape("2362").unwrap();
    south_shapes.add_shape("2263").unwrap();
    south_shapes.add_shape("2272").unwrap();

    let south_specs = HandDescriptor::new(vec![HandType::new(
        Shape::Custom(south_shapes),
        HcpRange::new(18, 20),
    )]);
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
    let mut builder = DealerBuilder::new();
    builder
        .predeal(Seat::North, predeal)
        .with_hand_specification(Seat::South, south_specs)
        .with_hand_specification(Seat::West, HandDescriptor::new(vec![west_spec]))
        .with_function(|hands: &Hands| hands.south().spades().high_card_points() > 2);
    let dealer = builder.build().unwrap();
    let deal = dealer.deal().unwrap();
    assert!(deal.south().diamonds().len() >= 6);
}
