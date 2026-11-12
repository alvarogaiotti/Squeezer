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
//! Two clients share the endpoint constants, the error type, the login form,
//! the session and listing classifiers, and the hand-link parser; only the
//! transport differs. `BlockingBBOClient` (feature `blocking`) drives
//! `reqwest::blocking`, while `AsyncBBOClient` (feature `async`, the default)
//! drives `reqwest`'s async client.
//!
//! The crate owns only the HTTP side of the download: it depends on nothing
//! from the card, deal, solver or parser crates, so a consumer that never
//! downloads hands never compiles an HTTP stack. `download` returns each
//! fetched hand record as a complete LIN document (`Vec<String>`);
//! `download_report` keeps the successful documents and the per-hand failures,
//! and `download_to_dir` writes BBO's suggested `<id>.lin` filenames while
//! skipping files that already exist. Converting a batch into `Board`s is the
//! importer's job (`squeezer_io::linparser::parse_lin_documents`), which keeps
//! this crate's dependency edge at zero Squeezer crates.
//!
//! BBO returns at most 31 days in one listing page, rejects a longer window
//! with an HTTP 200 `Invalid input` page, and redirects an expired session to
//! the login page. [`Window`] splits a wider range into 31-day pages, and the
//! clients classify those three responses instead of treating the login page
//! as an empty listing.

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
pub use helpers::{
    BBOBASE, BBOClient, BBOHANDS, BBOLAND, BBOLOGIN, BboError, DownloadFailure, DownloadPolicy,
    DownloadReport, DownloadedHand, MAX_LISTING_DAYS, Window,
};
