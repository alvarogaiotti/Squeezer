mod deal;
mod hand;
mod lib;
mod payoff;
mod shape;
mod smartstack;

mod prelude {
    pub const SUITS: usize = 4;
    pub const SHAPE_COMBINATIONS: usize = 14usize.pow(4);
    pub const MAX_N_OF_BOARDS: u8 = 32;
    pub const RANKS: u8 = 13;
    pub const NUMBER_OF_HANDS: usize = 4;
    pub use crate::deal::*;
    pub use crate::hand::*;
    pub use crate::lib::*;
    pub use crate::payoff::*;
    pub use crate::shape::*;
    pub use crate::smartstack::*;
    pub use bridge_deck::{Card, Cards, Suit};
    pub use colored::Colorize;
    pub use itertools::{any, Itertools};
    pub use std::{
        array::IntoIter,
        collections::{HashMap, HashSet},
        error::Error,
        fmt,
        hash::{Hash, Hasher},
        str::FromStr,
    };
}

use prelude::*;

fn main() {
    let mut found = 0;
    let goal = 100;
    let mut d33orks = 0;
    let mut squeeze = 0;
    let suitquality = Evaluator::new(&[2, 2, 1, 1, 1]);
    let mut factory = ShapeFactory::new();
    for _ in 0..10usize.pow(10) {
        if found > goal {
            break;
        }
        let mut deal = Deal::new(
            Constraints::Predeal([
                (
                    Seat::North,
                    Some(Hand::from_str("SASQS4HKH4DKD9D6D5CKCQCJC7").unwrap()),
                ),
                (
                    Seat::South,
                    Some(Hand::from_str("S7S5H2DAD7D2CACTC9C8C6C4C3").unwrap()),
                ),
                (Seat::East, None),
                (Seat::West, None),
            ]),
            &mut factory,
        );
        if deal.west().hearts().len() > 6 && suitquality.evaluate(&deal.west().hearts()) >= 3 {
            deal.long();
            found += 1;
            if deal.west().diamonds().len() == 3 || deal.west().cards.contains(Card::SK) {
                d33orks += 1;
                //deal.print()
            } else if deal.east().diamonds().len() > 3 && deal.east().cards.contains(Card::SK) {
                squeeze += 1;
                //deal.print()
            }
        }
    }
    println!("Found:{}", found);
    println!(
        "Squeeze:{}\nQuadri 3-3 o K♠ in impasse:{}",
        squeeze, d33orks
    );
}

fn polish_club(hands: &[Hand; 4], factory: &mut ShapeFactory) -> bool {
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
