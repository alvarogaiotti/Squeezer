fn main_to_be() -> String {
    let predeal = Hand::from_str("SQJ763HT2CJT9832").unwrap();
    let mut south_shapes = Shapes::new();
    south_shapes
        .add_shape(ShapeDescriptor::new("(4333)"))
        .unwrap();
    let south_specs = HandDescriptor::new(vec![HandArchetype::new(
        south_shapes,
        HcpRange::new(24, 37),
    )]);
    let mut builder = DealerBuilder::new();
    builder
        .predeal(Seat::North, predeal)
        .with_hand_specification(Seat::South, south_specs);
    let dealer = builder.build();
    let mut deal = dealer.deal().unwrap();
    deal.pbn();
    deal.as_string()
}
