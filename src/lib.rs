#![allow(dead_code)]

#[cfg(feature = "bbo")]
mod bbo;
#[cfg(feature = "bbo_async")]
mod bbo_async;
#[cfg(any(feature = "bbo", feature = "bbo_async"))]
mod bbohelpers;
mod card;
mod deal;
mod dealproduction;
mod evaluator;
mod hand;
#[cfg(feature = "lin")]
mod linparser;
//mod parser;
mod payoff;
mod shape;
mod simulation;
mod utils;

mod prelude {
    pub extern crate dds;
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
    #[cfg(feature = "bbo")]
    pub use crate::bbo::*;
    #[cfg(feature = "bbo_async")]
    pub use crate::bbo_async::*;
    #[cfg(any(feature = "bbo", feature = "bbo_async"))]
    pub use crate::bbohelpers::*;
    pub use crate::card::*;
    pub use crate::deal::*;
    pub use crate::dealproduction::*;
    pub use crate::evaluator::*;
    pub use crate::hand::*;
    pub use crate::linparser::*;
    #[cfg(feature = "lin")]
    pub use crate::linparser::*;
    pub use crate::payoff::*;
    pub use crate::shape::*;
    pub use crate::simulation::*;
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
