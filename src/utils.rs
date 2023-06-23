use crate::prelude::*;

#[must_use]
pub fn polish_club(hand: Hand) -> bool {
    let weak1n = HandTypeBuilder::new()
        .add_shape("(4432)")
        .add_shape("(4333)")
        .add_shape("4414")
        .add_shape("(5332)")
        .remove_shape("(5xx)x")
        .add_shape("(5xxx)")
        .with_range(11, 14)
        .build();
    let strong_any = HandTypeBuilder::new()
        .add_shape("xxxx")
        .with_range(18, 37)
        .build();
    let possible_hands = HandDescriptor::new(vec![weak1n, strong_any]);
    /* let hand = hands[seat as usize];
    hand_type.check(&hand) && 10 < hand.hcp() && hand.hcp() < 15
        || hand.clubs().len() == hand.into_iter().map(|x| x.len()).max().unwrap()
            && !Shapes::new().add_shape("(5xx)5").(&hand, "(5xx)5")
        || factory.is_not_in(&hand, "(144)4") && 14 < hand.hcp()
        || hand.hcp() > 17 */
    possible_hands.check(hand)
}

#[must_use]
pub fn weak2(hand: Hand) -> bool {
    let w2 = Evaluator::new(&[2, 2, 1, 1, 1]);
    let controls = Evaluator::new(&[2, 1]);
    let hcp = hand.hcp();
    (5..=10).contains(&hcp)
        && hand.clen() <= 4
        && hand.dlen() <= 4
        && (hand.slen() == 6
            && hand.hlen() <= 3
            && w2.evaluate(hand.as_cards()) > 3
            && controls.evaluate(hand.as_cards()) < 4
            || hand.hlen() == 6
                && hand.slen() <= 3
                && w2.evaluate(hand.hearts()) > 3
                && controls.evaluate(hand.as_cards()) < 4)
}

#[must_use]
fn evaluate_short_honors(hand: Hand) -> u8 {
    let mut malus: u8 = 0;
    malus += hand
        .into_iter()
        .map(|suit| match suit.len() {
            0..=1 => suit.kings().len() + suit.queens().len() + suit.jacks().len(),
            2 => suit.queens().len() + suit.jacks().len(),
            _ => 0u8,
        })
        .sum::<u8>();
    malus
}

#[must_use]
fn evaluate_lenght_and_concentration(hand: Hand) -> u8 {
    let mut sorted_suits = hand.into_iter().sorted_by_key(Cards::len).rev();
    let longest = sorted_suits.next().unwrap();
    let longest2nd = sorted_suits.next().unwrap();
    let longest3rd = sorted_suits.next().unwrap();
    let shortest = sorted_suits.next().unwrap();
    let sum_long = longest.len() + longest2nd.len();
    let diff_long_short = longest.len() - shortest.len();
    let mut points = sum_long + diff_long_short;
    if 10 < hand.hcp()
        && hand.hcp() < 15
        && longest.high_card_points() + longest2nd.high_card_points() >= hand.hcp() - 1
        || (14 < hand.hcp()
            && longest.high_card_points()
                + longest2nd.high_card_points()
                + longest3rd.high_card_points()
                >= hand.hcp() - 1)
    {
        points += 1;
    }

    if points == 25 && hand.spades().len() > 3 {
        points += 1
    }
    points
}
#[must_use]
pub fn zar_points(hand: Hand) -> u8 {
    let zar_evaluator = Evaluator::new(&[6, 4, 2, 1]);
    zar_evaluator.evaluate(hand.as_cards()) + evaluate_lenght_and_concentration(hand)
        - evaluate_short_honors(hand)
}

#[must_use]
pub fn dealer_of_3nt_opening(seat: Option<Seat>) -> impl Dealer {
    let mut builder = DealerBuilder::new();
    builder.with_function(Box::new(move |hands: &Hands| {
        let hand_type = HandTypeBuilder::new()
            .add_shape("(8x)xx")
            .add_shape("(7x)xx")
            .add_shape("(9x)xx")
            .build();
        if let Some(seat) = seat {
            hand_type.check(hands[seat as usize])
                && (26..33).contains(&zar_points(hands[seat as usize]))
        } else {
            hands
                .iter()
                .any(|hand| hand_type.check(*hand) && (26..33).contains(&zar_points(*hand)))
        }
    }));
    builder.build().unwrap()
}

#[must_use]
pub fn deal_1nt_3nt(hands: &[Hand; 4], _factory: &mut ShapeFactory) -> bool {
    for (_seat, hand) in hands.iter().enumerate() {
        if hand.hcp() < 18 && hand.hcp() > 14 {}
    }
    todo!()
}
