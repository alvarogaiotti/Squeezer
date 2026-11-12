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

//! Reading and writing bridge hand records: the LIN and PBN importers and their
//! file-container writers.
//!
//! The parsers build [`squeezer_model::record::Board`]s and read cards through
//! [`squeezer_cards`], so those two lower-level crates plus `nom` are the whole
//! dependency set (`nom` and `serde` are optional). A consumer that reads local
//! records therefore never compiles the solver, the C++ toolchain or an HTTP
//! stack, and a consumer that only generates deals never compiles `nom`.
//!
//! # Features
//!
//! - `lin` enables [`linparser`] and the [`LinFile`](linparser::LinFile)
//!   writer. LIN is the format BBO exports; `parse_lin_documents` imports a
//!   batch of complete LIN documents, as a BBO download returns them.
//! - `pbn` enables [`pbnparser`] and the [`PbnFile`](pbnparser::PbnFile)
//!   writer.
//! - `serde` derives `Serialize`/`Deserialize` on the error categories.
//!
//! Both formats are on by default. `nom` is compiled only when one of them is.
//! The `Deal` and `Board` writers themselves are unconditional and live in
//! `squeezer-model`; only the file containers need a parser.

/// The model vocabulary the importers build, plus the card types they read.
///
/// This is crate-internal: consumers import from [`squeezer_model`] and this
/// crate's roots, not through here. It is gated with the importers so a
/// feature-less build compiles without unused imports.
#[cfg(any(feature = "lin", feature = "pbn"))]
pub(crate) mod prelude {
    pub(crate) use squeezer_cards::*;
    pub(crate) use squeezer_model::*;
}

/// The container writers can only live where the container types do, so they
/// sit with the importers rather than in the model crate.
#[cfg(any(feature = "lin", feature = "pbn"))]
mod containers;

/// The LIN importer, behind the `lin` feature.
#[cfg(feature = "lin")]
pub mod linparser;

/// The PBN importer, behind the `pbn` feature.
#[cfg(feature = "pbn")]
pub mod pbnparser;
