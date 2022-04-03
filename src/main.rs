mod deal;
mod hand;
mod lib;
mod payoff;
mod shape;
mod smartstack;

mod prelude {
    pub const SUITS: usize = 4;
    pub const SHAPE_COMBINATIONS: usize = 14usize.pow(4);
    pub const RANKS: u8 = 13;
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
    let mut fiori_nat = 0;
    let suitquality = Evaluator::new(&[2, 2, 1, 1, 1]);
    let mut factory = ShapeFactory::new();
    for _ in 0..10usize.pow(10) {
        if found > goal {
            break;
        }
        let deal = Deal::new(Constraints::Bounds(&polish_club), &mut factory);
        if deal.west().hcp() > 8
            && suitquality.evaluate(&deal.west().clubs()) > 3
            && deal.west().clubs().len() > 5
        {
            found += 1;
            println!("Fiori naturale:");
            println!("{}", deal);
            fiori_nat += 1;
            let mut remain_cards = [[0u32; 4]; 4];
            for (i, hand) in deal.into_iter().enumerate() {
                for (j, suit) in hand.into_iter().enumerate() {
                    let sum = suit.map(|card| 1 << card.rank() as u32).sum();
                    remain_cards[i][j] = sum;
                }
            }
            let c_deal = crate::lib::deal {
                trump: 3,
                first: 0,
                currentTrickSuit: [0; 3],
                currentTrickRank: [0; 3],
                remainCards: remain_cards,
            };
            let mut futp: crate::lib::futureTricks = futureTricks {
                nodes: 0,
                cards: 0,
                suit: [0; 13],
                rank: [0; 13],
                equals: [0; 13],
                score: [0; 13],
            };
            let ptr: *mut futureTricks = &mut futp;
            unsafe { crate::lib::SolveBoard(c_deal, -1, 1, 1, ptr, 0) };
            println!("{}", 13 - futp.score[0]);
        }
    }
    //println!("Found:{}", found);
    println!("Fiori naturale:{}", fiori_nat);
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
