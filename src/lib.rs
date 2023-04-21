#[allow(dead_code)]
mod card;
mod dds;
mod ddsffi;
mod deal;
mod dealproduction;
mod evaluator;
mod hand;
mod linparser;
mod payoff;
mod shape;
mod utils;

mod prelude {
    pub const ZERO_LENGTH: u8 = 0;
    pub const ZERO_HCP: u8 = 0;
    pub const MAX_HCP_IN_HAND: u8 = 37;
    pub const MAX_HCP_IN_DECK: u8 = 40;
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
    pub use crate::parse::*;
    pub use crate::payoff::*;
    pub use crate::shape::*;
    pub use crate::utils::*;
    pub(crate) use colored::Colorize;
    pub(crate) use itertools::{any, Itertools};
    pub use std::str::FromStr;

    pub(crate) use std::{
        array::IntoIter,
        collections::{HashMap, HashSet},
        error::Error,
        fmt,
        hash::{Hash, Hasher},
        ops::RangeInclusive,
    };
}

pub use prelude::*;
#[cfg(feature = "bbo")]
pub mod bbo;
#[cfg(feature = "bbo_async")]
pub mod bbo_async;
