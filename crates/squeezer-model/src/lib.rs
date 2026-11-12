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

//! The bridge model vocabulary: hand shapes and their pattern DSL, per-seat
//! hand constraints, the contract vocabulary (seats, vulnerability, strains,
//! contracts) and the card evaluator.
//!
//! This crate sits directly above [`squeezer_cards`]: it depends on the card
//! vocabulary, `bitvec` and `itertools`, and deliberately names no solver,
//! import parser or network crate. A consumer that only builds and checks
//! hands therefore never compiles the native double-dummy library. `serde`
//! support is optional.
//!
//! Every fallible surface reports a typed error owned by the module that
//! produced it ([`CreationShapeError`], [`ConstraintError`], [`ContractError`],
//! [`SeatError`]); the crate defines no crate-wide error enum.

pub mod contract;
pub mod evaluator;
pub mod hand_constraint;
mod range;
pub mod shape;
pub mod shapeparser;

pub use contract::*;
pub use evaluator::*;
pub use hand_constraint::*;
pub use shape::*;
pub use shapeparser::*;

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
