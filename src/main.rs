mod deal;
mod hand;
mod shape;
mod smartstack;

mod prelude {
    pub const SUITS: usize = 4;
    pub const SHAPE_COMBINATIONS: usize = 14usize.pow(4);
    pub const RANKS: u8 = 13;
    pub use crate::deal::*;
    pub use crate::hand::*;
    pub use crate::shape::*;
    pub use crate::smartstack::*;
    pub use bridge_deck::{Card, Cards, Suit};
    pub use itertools::{any, Itertools};
    pub use std::{
        collections::HashMap,
        error::Error,
        fmt,
        hash::{Hash, Hasher},
        str::FromStr,
    };
}
use prelude::*;

fn main() {
    let mut found = 0;
    let goal = 1000;
    let mut fiori_nat = 0;
    let suitquality = Evaluator::new(&[2, 2, 1, 1, 1]);
    let mut factory = ShapeFactory::new();
    for x in 0..10usize.pow(10) {
        if found > goal {
            break;
        }
        let deal = Deal::new(Constraints::Bounds(&polish_club), &mut factory);
        found += 1;
        if deal.west().hcp() > 8
            && suitquality(&deal.west().clubs()) > 3
            && deal.west().clubs().len() > 5
        {
            println!("Fiori naturale");
            println!("{}", deal);
            fiori_nat += 1
        }
    }
    //println!("Found:{}", found);
    println!("Fiori naturale:{}", fiori_nat);
}

fn print_type_of<T>(_: &T) {
    println!("{}", std::any::type_name::<T>())
}

fn polish_club(hands: &[Hand; 4], factory: &mut ShapeFactory) -> bool {
    let mut factory = factory;
    let shape = factory.new_shape(Some("(4432)")).unwrap()
        + factory.new_shape(Some("(4333)")).unwrap()
        + factory.new_shape(Some("4414")).unwrap()
        + factory.new_shape(Some("(5332)")).unwrap()
        - factory.new_shape(Some("(5xx)x")).unwrap();
    let hand = hands[0];
    Shape::includes(&shape, &hand) && 10 < hand.hcp() && hand.hcp() < 15
        || (hand.clubs().len() == hand.into_iter().map(|x| x.len()).max().unwrap()
            && !Shape::includes(&factory.new_shape(Some("(5xx)5")).unwrap(), &hand)
            || Shape::includes(&factory.new_shape(Some("(144)4")).unwrap(), &hand))
            && 14 < hand.hcp()
        || hand.hcp() > 17
}
