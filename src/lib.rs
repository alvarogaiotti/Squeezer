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
    clippy::should_panic_without_expect,
    clippy::needless_doctest_main
)]
/*!This crate offers you every tool you might need for Bridge (the card game) related stuff.
 We support some variegated functionality: random single deal creation (via [`DealerBuilder`]),
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

 The double dummy results computed with [`dds`] feed the statistics utilities this crate provides
 (e.g. the [`performance_analysis::analyze_board()`] entry point or the lower-level
 [`performance_analysis::analyze_hand()`]), like accuracy or tricks lost, or tricks lost per deal,
 etc. Hand records come from wherever you get them: the `lin` and `pbn` features parse local files,
 while the separate `squeezer-bbo` crate handles downloading them from BBO's My hands, so this
 crate carries no HTTP stack.

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

  fn main() -> Result<(), Box<dyn std::error::Error>> {
      use crate::*;

      // Create a new random deal.
      // If you don't care about the characteristics of the deal
      // then this is the best method.
      let _deal = DealerBuilder::new().build()?.deal()?;

      // If you want a much finer approach, you should use a DealerBuilder:

      // Create a HandConstraint with the builder for more ergonomics
      let north_specs1 = HandConstraintBuilder::new()
          .with_longest(Suit::Diamonds)?
          .with_range(5, 15)
          .remove_shape("xx7+x")? // This removes any shape with 7 or more diamonds
          .build()?;

      // Or simply write it out yourself!
      let north_specs2 = HandConstraint::new(
          &["2623", "4432", "4522"],
          HcpRange::new(5, 15),
      )?;

      // Give everything to the DealerBuilder, which will craft a brand new impl Dealer for you!
      // The predeal grammar is positional: each space (or `.`) advances a suit,
      // starting from spades, and suit and rank letters are case-insensitive.
      let dealer_builder = DealerBuilder::new()
         .predeal(Seat::South, Cards::from_str("AKQ AJT9 T3 AK95")?)?
         .with_hand_constraint(
             Seat::North,
             HandConstraint::any(vec![north_specs1, north_specs2])?
         );
     let dealer = dealer_builder.build().unwrap();

      for _ in 0..200 {
          // Let's deal
          println!("{}", dealer.deal().unwrap());
      }
      Ok(())
  }
 ```

 DISCLOSURE: The [`crate::card`] module comes from David Roundy's crate [bridge-cards](https://github.com/droundy/bridge-cards).
 Only small modifications were made by me on that module.
*/

/// The card vocabulary (`Suit`, `Card`, `Cards` and the samplers) lives in the
/// `squeezer-cards` crate; this module keeps the historical `squeezer::card`
/// path and the prelude still re-exports its items flat.
pub mod card {
    pub use squeezer_cards::card::*;
}
mod contract;
#[cfg(feature = "dds")]
mod dds_bridge;
pub mod dealer;
mod error;
mod evaluator;
mod hand_constraint;
#[cfg(feature = "lin")]
pub mod linparser;
#[cfg(feature = "pbn")]
pub mod pbnparser;
/// Backend-agnostic performance analysis; the DDS-backed [`PlayAnalyser`]
/// lives in [`solver`], so `dds` alone is enough.
///
/// [`PlayAnalyser`]: solver::PlayAnalyser
#[cfg(feature = "dds")]
pub mod performance_analysis;
mod range;
pub mod record;
mod shape;
mod shapeparser;
#[cfg(feature = "dds")]
pub mod simulation;
/// Backend-agnostic double-dummy solver seam; feature-independent, so the core
/// builds without the native `dds` library.
pub mod solver;
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
    pub use crate::card::*;
    pub use crate::dealer::*;
    pub use crate::error::*;
    pub use crate::evaluator::*;
    pub use crate::hand_constraint::*;
    pub use crate::record::*;
    pub use crate::shape::*;
    pub use crate::shapeparser::*;
    #[cfg(feature = "dds")]
    pub use crate::simulation::*;
    pub use crate::solver::*;
    pub use crate::utils::*;
    pub(crate) use itertools::Itertools;
    pub use std::str::FromStr;

    pub(crate) use std::{array::IntoIter, error::Error, fmt, hash::Hash, ops::RangeInclusive};
}
pub use prelude::*;
