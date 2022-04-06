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
    let mut balanced = ShapeFactory::balanced();
    let constraint = Constraints::Bounds(&|deal: &[Hand; 4], factory: &mut ShapeFactory| -> bool {
        deal[Seat::South as usize].hcp() > 20
            && deal[Seat::South as usize].spades().len() > 4
            && deal[Seat::North as usize].hearts().len() > 3
            && !factory.includes(&deal[Seat::North as usize])
            || deal[Seat::North as usize].hcp() > 20
                && deal[Seat::North as usize].spades().len() > 4
                && deal[Seat::South as usize].hearts().len() > 3
                && !factory.includes(&deal[Seat::South as usize])
    });
    let mut found = 0;
    let goal = 100;
    for i in 0..101u8 {
        if found > goal {
            break;
        }
        let deal = Deal::new_with_conditions(&constraint, &mut balanced);
        found += 1;
        println!("{}", deal.as_lin(i));
    }
}
