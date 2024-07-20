// Copyright (C) 2024 Alvaro Gaiotti
// See end of file for license information

#![allow(dead_code)]
#![warn(clippy::pedantic)]
#![allow(clippy::unseparated_literal_suffix, clippy::implicit_return)]
#![allow(clippy::single_call_fn)]
#![allow(clippy::missing_trait_methods)]
#![allow(clippy::indexing_slicing)]
#![allow(clippy::as_conversions)]
#![allow(clippy::question_mark_used)]
#![allow(clippy::std_instead_of_core)]
#![allow(clippy::std_instead_of_alloc)]
#![allow(clippy::absolute_paths)]
#![allow(clippy::expect_used)]
#![allow(clippy::arithmetic_side_effects)]
#![allow(clippy::pub_with_shorthand)]
#![allow(clippy::missing_docs_in_private_items)]
#![allow(clippy::missing_inline_in_public_items)]
#![allow(clippy::unreachable)]
#![allow(clippy::unwrap_used)]
#![allow(clippy::integer_division_remainder_used)]
#![allow(clippy::module_name_repetitions)]

#[cfg(feature = "dds")]
mod analyse_performance;
#[cfg(feature = "bbo")]
mod bbo;
#[cfg(feature = "bbo_async")]
mod bbo_async;
#[cfg(any(feature = "bbo", feature = "bbo_async"))]
mod bbohelpers;
mod card;
mod contract;
mod deal;
mod dealproduction;
mod error;
mod evaluator;
mod hand;
#[cfg(feature = "lin")]
mod linparser;
mod shape;
mod shapeparser;
#[cfg(feature = "dds")]
mod simulation;
mod utils;

mod prelude {
    #[cfg(feature = "dds")]
    pub extern crate dds;
    pub const ZERO_LENGTH: u8 = 0;
    pub const MAX_HCP_IN_HAND: u8 = 37;
    pub const MAX_HCP_IN_DECK: u8 = 40;
    pub const MAX_LENGTH: u8 = 13;
    pub const SUITS: usize = 4;
    pub const SHAPE_COMBINATIONS: usize = 560;
    pub const MAX_N_OF_BOARDS: u8 = 128;
    pub const RANKS: u8 = 13;
    pub use crate::contract::*;
    pub const NUMBER_OF_HANDS: usize = 4;
    #[cfg(feature = "dds")]
    pub use crate::analyse_performance::*;
    #[cfg(feature = "bbo")]
    pub use crate::bbo::*;
    #[cfg(feature = "bbo_async")]
    pub use crate::bbo_async::*;
    #[cfg(any(feature = "bbo", feature = "bbo_async"))]
    pub use crate::bbohelpers::*;
    pub use crate::card::*;
    pub use crate::deal::*;
    pub use crate::dealproduction::*;
    pub use crate::error::*;
    pub use crate::evaluator::*;
    pub use crate::hand::*;
    #[cfg(feature = "lin")]
    pub use crate::linparser::*;
    pub use crate::shape::*;
    pub use crate::shapeparser::*;
    #[cfg(feature = "dds")]
    pub use crate::simulation::*;
    pub use crate::utils::*;
    pub(crate) use colored::Colorize;
    pub(crate) use itertools::Itertools;
    pub use std::str::FromStr;

    pub(crate) use std::{
        array::IntoIter,
        collections::HashMap,
        error::Error,
        fmt,
        hash::{Hash, Hasher},
        ops::RangeInclusive,
    };
}

pub use prelude::*;
