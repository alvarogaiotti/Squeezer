use crate::prelude::*;

pub fn polish_club(hands: &[Hand; 4], factory: &mut ShapeFactory) -> bool {
    factory.new_shape("(4432)").unwrap();
    factory.new_shape("(4333)").unwrap();
    factory.new_shape("4414").unwrap();
    factory.new_shape("(5332)").unwrap();
    let factory = factory - "(5xx)x";
    let hand = hands[0];
    factory.includes(&hand) && 10 < hand.hcp() && hand.hcp() < 15
        || hand.clubs().len() == hand.into_iter().map(|x| x.len()).max().unwrap()
            && !factory.is_not_in(&hand, "(5xx)5")
        || factory.is_not_in(&hand, "(144)4") && 14 < hand.hcp()
        || hand.hcp() > 17
}

pub fn weak2(hand: &Hand) -> bool {
    let w2 = Evaluator::new(&[2, 2, 1, 1, 1]);
    let controls = Evaluator::new(&[2, 1]);
    5 <= hand.hcp()
        && hand.hcp() <= 10
        && hand.clubs().len() <= 4
        && hand.diamonds().len() <= 4
        && (hand.spades().len() == 6
            && hand.hearts().len() <= 3
            && w2.evaluate(&hand.spades()) > 3
            && controls.evaluate(&hand.cards) < 4
            || hand.hearts().len() == 6
                && hand.spades().len() <= 3
                && w2.evaluate(&hand.hearts()) > 3
                && controls.evaluate(&hand.cards) < 4)
}

fn evaluate_short_honors(hand: &Hand) -> u8 {
    let mut malus: u8 = 0;
    malus += hand
        .into_iter()
        .map(|suit| match suit.len() {
            0..=1 => (suit.kings().len() + suit.queens().len() + suit.jacks().len()) as u8,
            2 => (suit.queens().len() + suit.jacks().len()) as u8,
            _ => 0u8,
        })
        .sum::<u8>();
    malus
}

fn evaluate_lenght_and_concentration(hand: &Hand) -> u8 {
    let mut sorted_suits = hand.into_iter().sorted_by_key(|x| x.len()).rev();
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
    points as u8
}
pub fn zar_points(hand: &Hand) -> u8 {
    let zar_evaluator = Evaluator::new(&[6, 4, 2, 1]);
    zar_evaluator.evaluate(&hand.cards) + evaluate_lenght_and_concentration(hand)
        - evaluate_short_honors(hand)
}

pub fn deal_3nt_opening(hands: &[Hand; 4], factory: &mut ShapeFactory) -> bool {
    factory.new_shape("(8x)xx");
    factory.new_shape("(7x)xx");
    factory.new_shape("(9x)xx");
    for hand in hands {
        if factory.includes(hand) && 26 < zar_points(hand) && zar_points(hand) < 33 {
            return true;
        }
    }
    false
}

pub fn deal_1nt_3nt(hands: &[Hand; 4], factory: &mut ShapeFactory) -> bool {
    for (_seat, hand) in hands.iter().enumerate() {
        if hand.hcp() < 18 && hand.hcp() > 14 {}
    }
    todo!()
}
