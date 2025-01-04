// Copyright (C) 2024 Alvaro Gaiotti
// See end of file for license information
#![warn(clippy::pedantic)]
#![allow(
    dead_code,
    clippy::module_name_repetitions,
    clippy::integer_division_remainder_used,
    clippy::missing_docs_in_private_items,
    clippy::pub_with_shorthand,
    clippy::question_mark_used,
    clippy::as_conversions,
    clippy::indexing_slicing,
    clippy::missing_trait_methods,
    clippy::single_call_fn,
    clippy::unseparated_literal_suffix,
    clippy::implicit_return,
    clippy::should_panic_without_expect
)]
/*!This crate offers you every tool you might need for Bridge (the card game) related stuff.
 We support some variegated functionality: random single deal creation (via [`Deal::new()`]),
 creation of specific deals (via a [`DealerBuilder`] and all its related method, giving you a
 [`Dealer`] with all the feature you asked for), predealing etc.
 We provide you, under a feature flag (`dds`) various tools for analyzing playes, calculating
 pars, double dummy analysis and more.
 The [`dds`] crate is simply a wrapper around [Bo Haglund's
 DDS](https://github.com/dds-bridge/dds), read more on the crate page.

 This crate is ideal for fast bridge simulation (the idea was born from the frustration of
 simulating innumerable hands with the majestic [Anthony Lee's
 Redeal](https://github.com/anntzer/redeal), which took a lot of time) and was build with the
 aim to help the bridge professional and _amateur_ alike improve theirs bidding system with an
 eye for frequencies and statistics (informed by partner probable hand, e.g. how often I'll have
 13 HCP in front of a partner showing a strong hand?: this can help in balancing some choices).

 Part of this library, under the features `bbo` and `bbo_async`, provides some features for
 downloading hands from BBO's My hands, so you can get the hands you played on the site, get double
 dummy results for every card you played with [`dds`] and then compute some statistics on the
 results using the utilities provided for you in this crate (e.g.
 [`performance_analysis::analyse_players_performance()`] function), like accuracy or triks
 lost, or trick lost per deal, etc.

 Another cool use that its implemented is the [`LeadSimulation`] utility, which mimics the
 functionality of [`LeadSolver` by Matthew J.
 Kidd](https://lajollabridge.com/Software/Lead-Solver/Lead-Solver-About.htm): giving you the answer you yearned for and
 telling you what your lead should have been given the information you obtained during the auction. We
 do this by simulating how many deals you want and double dummy solving them for every possible
 lead, collecting statistics in the meanwhile. And then making you feel sad for your wrong
 answer.

 The functionality of this crate and, particularly, its API is in extrimely early stages, since
 I used this project to learn Rust and I had no intentions to make it public.
 This crate is more of a place to experiment but I can imagine it growing and stabilizing a bit.

 My goal by releasing it is so that people other than me can experiment with its feature
 and help me with suggestion to get the crate farly usable.

 Enough talk, here some examples:
 ```
 # use squeezer::*;

 fn main() {
     use crate::*;

     // Create a new random deal, Board 1 etc.
     // If you don't care about the characteristics of the deal
     // then this is the best method.
     let _deal = Deal::new();

     // If you want a much finer approach, you should use a DealerBuilder:
     let mut dealer_builder = DealerBuilder::new();
     // Note that we are strict on the case of the string for the hand
     dealer_builder.predeal(Seat::South, Cards::from_str("AKQ AJT9 T3 AK95").unwrap());

     // Create a HandType with the builder for more ergonomics
     let north_specs1 = HandTypeBuilder::new()
         .with_longest(Suit::Diamonds)
         .with_range(5, 15)
         .remove_shape("xx7+x") // This removes any shape with 7 or more diamonds
         .unwrap()
         .build();

     // Or simply write it out yourself!
     let north_specs2 = HandType::new(
         Shape::new_from_patterns(&["2623", "4432", "4522"]).unwrap(),
         HcpRange::new(5, 15),
     );

     // Give everything to the DealerBuilder, which will craft a brand new impl Dealer for you!
     dealer_builder.with_hand_descriptor(
         Seat::North,
         HandDescriptor::new(vec![north_specs1, north_specs2]),
     );
     let dealer = dealer_builder.build().unwrap();

     for _ in 0..200 {
         // Let's deal
         println!("{}", dealer.deal().unwrap());
     }
 }
 ```
*/

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
#[cfg(feature = "dds")]
pub mod performance_analysis;
mod shape;
mod shapeparser;
#[cfg(feature = "dds")]
pub mod simulation;
mod utils;

pub mod prelude {
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
    pub(crate) use itertools::Itertools;
    pub use std::str::FromStr;

    pub(crate) use std::{
        array::IntoIter,
        error::Error,
        fmt,
        hash::{Hash, Hasher},
        ops::RangeInclusive,
    };
}
pub use prelude::*;
