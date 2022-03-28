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
    let mut deck = Cards::ALL;
}

fn print_type_of<T>(_: &T) {
    println!("{}", std::any::type_name::<T>())
}
