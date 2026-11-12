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

//! The bridge model vocabulary: the whole domain model, from ready-made hand
//! predicates up through hand shapes, constraints and deals, to dealers,
//! recorded boards, auctions, plays and positions.
//!
//! This crate sits directly above [`squeezer_cards`]: it depends on the card
//! vocabulary, `rand`, `bitvec` and `itertools`, and deliberately names no
//! solver, import parser or network crate. A consumer that only builds,
//! generates and checks hands and deals therefore never compiles the native
//! double-dummy library. `serde` support is optional.
//!
//! Every fallible surface reports a typed error owned by the module that
//! produced it ([`CreationShapeError`], [`ConstraintError`], [`ContractError`],
//! [`SeatError`], [`DealerError`], [`DealError`], [`BoardError`],
//! [`AuctionError`], [`PositionError`]); the crate defines no crate-wide error
//! enum. The LIN and PBN writers for [`Deal`] and [`Board`] live here too; the
//! `LinFile` and `PbnFile` container writers need the parsers and live in
//! `squeezer-io`.

pub mod contract;
pub mod dealer;
pub mod hand_constraint;
mod range;
pub mod record;
pub mod shape;
pub mod shapeparser;
pub mod utils;

pub use contract::*;
pub use dealer::*;
pub use hand_constraint::*;
pub use record::*;
pub use shape::*;
pub use shapeparser::*;
pub use utils::*;

/// The items the moved record and dealer modules share, mirroring the
/// historical prelude so the code reads the same.
///
/// This is crate-internal: consumers import from [`squeezer_cards`] and this
/// crate's root, not through here.
pub(crate) mod prelude {
    pub(crate) use crate::*;
    pub(crate) use squeezer_cards::*;
    // The unit tests parse cards from strings; the library itself never needs
    // this trait, so the import is test-only to keep the lib warning-free.
    #[cfg(test)]
    pub(crate) use std::str::FromStr;
    pub(crate) use std::{array::IntoIter, error::Error, fmt, hash::Hash};
}

// ==============================================================================
// Table and hand limits
// ==============================================================================

/// Highest High Card Point total a single 13-card hand can hold (`AKQJ` in
/// three suits plus one ace).
pub const MAX_HCP_IN_HAND: u8 = 37;

/// High Card Point total of the whole deck: every ace, king, queen and jack.
pub const MAX_HCP_IN_DECK: u8 = 40;

/// Cards in a complete hand, and the longest a single suit can be.
pub const MAX_LENGTH: u8 = 13;

/// Number of suits in a deck.
pub const SUITS: usize = 4;

/// Number of distinct shape patterns: the suit lengths that sum to thirteen.
pub const SHAPE_COMBINATIONS: usize = 560;

/// Number of ranks in a suit.
pub const RANKS: u8 = 13;

/// Number of seats at the table, and therefore of hands in a deal.
pub const NUMBER_OF_HANDS: usize = 4;

/// Suit length of a void.
pub const ZERO_LENGTH: u8 = 0;
