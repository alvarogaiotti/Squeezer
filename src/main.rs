mod deal;
mod hand;
mod lib;
mod payoff;
mod shape;
mod smartstack;
mod utils;

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
    pub use crate::utils::*;
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
        let mut deal = Deal::new_with_conditions(Constraints::Predeal([
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
        ]));
        if deal.west().hearts().len() > 6 && suitquality.evaluate(&deal.west().hearts()) >= 3 {
            deal.long();
            found += 1;
            if deal.west().diamonds().len() == 3 || deal.west().contains(Card::SK) {
                d33orks += 1;
                //deal.print()
            } else if deal.east().diamonds().len() > 3 && deal.east().contains(Card::SK) {
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
