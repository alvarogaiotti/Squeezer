mod deal;
mod hand;
mod shape;

mod prelude {
    pub const SUITS: usize = 4;
    pub const SHAPE_COMBINATIONS: usize = 14usize.pow(4);
    pub const RANKS: u8 = 13;
    pub use crate::deal::*;
    pub use crate::hand::*;
    pub use crate::shape::*;
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
    let constraint = Constraints::None;
    let _placeholder = Deal::new(constraint);
}
