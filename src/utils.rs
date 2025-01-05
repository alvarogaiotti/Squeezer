// Copyright (C) 2024 Alvaro Gaiotti
// See end of file for license information

use crate::prelude::*;

#[must_use]
#[allow(clippy::missing_panics_doc)]
pub fn polish_club_hand_descriptor() -> HandDescriptor {
    let weak1n = HandTypeBuilder::new()
        .add_shape("(4432)")
        .unwrap()
        .add_shape("(4333)")
        .unwrap()
        .add_shape("(4414)")
        .unwrap()
        .add_shape("(5332)")
        .unwrap()
        .remove_shape("(5x)(2+2+)")
        .unwrap()
        .with_range(11, 14)
        .build();
    let unbal_with_clubs = HandTypeBuilder::new()
        .with_longest(Suit::Clubs)
        .remove_shape("(4x)x5")
        .unwrap()
        .remove_shape("(5-5-5-)6+")
        .unwrap()
        .with_range(11, 14)
        .build();
    let strong_any = HandTypeBuilder::new()
        .add_shape("xxxx")
        .unwrap()
        .with_range(18, 37)
        .build();
    HandDescriptor::new(vec![weak1n, strong_any, unbal_with_clubs])
    /* let hand = hands[seat as usize];
    hand_type.check(&hand) && 10 < hand.hcp() && hand.hcp() < 15
        || hand.clubs().len() == hand.into_iter().map(|x| x.len()).max().unwrap()
            && !Shapes::new().add_shape("(5xx)5").(&hand, "(5xx)5")
        || factory.is_not_in(&hand, "(144)4") && 14 < hand.hcp()
        || hand.hcp() > 17 */
}

#[must_use]
#[allow(clippy::missing_panics_doc)]
pub fn weak2_hand_descriptor(hand: Hand) -> bool {
    let w2 = Evaluator::new(&[2, 2, 1, 1, 1]);
    let controls = Evaluator::new(&[2, 1]);

    let wk2 = HandTypeBuilder::new()
        .add_shape("(63-)4-4-")
        .unwrap()
        .with_range(5, 10)
        .build();
    wk2.check(hand)
        && ((hand.slen() == 6
            && w2.evaluate(hand.spades()) > 3
            && controls.evaluate(hand.as_cards()) < 4)
            || (hand.hlen() == 6
                && w2.evaluate(hand.hearts()) > 3
                && controls.evaluate(hand.as_cards()) < 4))
}

#[must_use]
fn devaluate_short_honors(hand: Hand) -> u8 {
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
    let mut sorted_suits = hand.into_iter().sorted_by_key(Cards::len);
    let shortest = sorted_suits.next().unwrap();
    let third_longest = sorted_suits.next().unwrap();
    let second_longest = sorted_suits.next().unwrap();
    let longest = sorted_suits.next().unwrap();
    let sum_long = longest.len() + second_longest.len();
    let diff_long_short = longest.len() - shortest.len();
    let mut points = sum_long + diff_long_short;
    let weak_concentration = 10 < hand.hcp()
        && hand.hcp() < 15
        && longest.high_card_points() + second_longest.high_card_points() >= hand.hcp() - 1;
    let strong_concentration = 14 < hand.hcp()
        && longest.high_card_points()
            + second_longest.high_card_points()
            + third_longest.high_card_points()
            >= hand.hcp() - 1;
    if weak_concentration || strong_concentration {
        points += 1;
    }

    if points == 25 && hand.spades().len() > 3 {
        points += 1;
    }
    points
}
#[must_use]
pub fn zar_points(hand: Hand) -> u8 {
    let zar_evaluator = Evaluator::new(&[6, 4, 2, 1]);
    zar_evaluator.evaluate(hand.as_cards()) + evaluate_lenght_and_concentration(hand)
        - devaluate_short_honors(hand)
}

#[must_use]
#[allow(clippy::missing_panics_doc)]
pub fn dealer_of_lavazza_3nt_opening(seat: Option<Seat>) -> impl Dealer {
    let mut builder = DealerBuilder::new();
    builder.with_function(Box::new(move |hands: &Hands| {
        let hand_type = HandTypeBuilder::new()
            .add_shape("(8x)xx")
            .unwrap()
            .add_shape("(7x)xx")
            .unwrap()
            .add_shape("(9x)xx")
            .unwrap()
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
