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

//! The card vocabulary of a bridge deck: [`Suit`], [`Card`], [`Cards`] and the
//! samplers that draw from a [`Cards`] deck.
//!
//! This crate is the bottom of the Squeezer workspace. It names no other
//! workspace crate, carries no native dependency, and keeps `rand` and `serde`
//! behind optional features, so a consumer that only needs to parse and compare
//! cards pays for neither.

pub mod card;
pub use card::*;
