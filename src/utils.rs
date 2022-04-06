use crate::prelude::*;

pub fn polish_club(hands: &[Hand; 4], factory: &mut ShapeFactory) -> bool {
    factory.new_shape(Some("(4432)")).unwrap();
    factory.new_shape(Some("(4333)")).unwrap();
    factory.new_shape(Some("4414")).unwrap();
    factory.new_shape(Some("(5332)")).unwrap();
    let factory = factory - "(5xx)x";
    let hand = hands[0];
    factory.includes(&hand) && 10 < hand.hcp() && hand.hcp() < 15
        || hand.clubs().len() == hand.into_iter().map(|x| x.len()).max().unwrap()
            && !factory.is_not_in(&hand, "(5xx)5")
        || factory.is_not_in(&hand, "(144)4") && 14 < hand.hcp()
        || hand.hcp() > 17
}
