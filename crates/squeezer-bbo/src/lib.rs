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

//! Downloading hands from Bridge Base Online.
//!
//! Two clients share the endpoint constants, the error type and the hand-link
//! parser; only the transport differs. `BlockingBBOClient` (feature `blocking`)
//! drives `reqwest::blocking`, while `AsyncBBOClient` (feature `async`, the
//! default) drives `reqwest`'s async client.
//!
//! The crate owns only the HTTP side of the download: it depends on nothing
//! from the card, deal, solver or parser crates, so a consumer that never
//! downloads hands never compiles an HTTP stack.

#[cfg(feature = "async")]
mod asynchronous;
#[cfg(feature = "blocking")]
mod blocking;
#[cfg(any(feature = "async", feature = "blocking"))]
mod helpers;

#[cfg(feature = "async")]
pub use asynchronous::AsyncBBOClient;
#[cfg(feature = "blocking")]
pub use blocking::BlockingBBOClient;
#[cfg(any(feature = "async", feature = "blocking"))]
pub use helpers::{BBOBASE, BBOClient, BBOHANDS, BBOLAND, BBOLOGIN, BboError};
