mod card;
mod dds;
mod ddsffi;
mod deal;
mod dealproduction;
mod evaluator;
mod hand;
mod payoff;
mod shape;
mod utils;

mod prelude {
    pub const ZERO_LENGTH: u8 = 0;
    pub const MAX_LENGTH: u8 = 13;
    pub const SUITS: usize = 4;
    pub const SHAPE_COMBINATIONS: usize = 14usize.pow(4);
    pub const MAX_N_OF_BOARDS: u8 = 128;
    pub const RANKS: u8 = 13;
    pub const NUMBER_OF_HANDS: usize = 4;
    pub use crate::card::*;
    pub use crate::dds::*;
    pub use crate::ddsffi::*;
    pub use crate::deal::*;
    pub use crate::dealproduction::*;
    pub use crate::evaluator::*;
    pub use crate::hand::*;
    pub use crate::payoff::*;
    pub use crate::shape::*;
    pub use crate::utils::*;
    pub(crate) use colored::Colorize;
    pub(crate) use itertools::{any, Itertools};
    pub(crate) use std::{
        array::IntoIter,
        collections::{HashMap, HashSet},
        error::Error,
        fmt,
        hash::{Hash, Hasher},
        ops::RangeInclusive,
        str::FromStr,
    };
}

pub use prelude::*;
