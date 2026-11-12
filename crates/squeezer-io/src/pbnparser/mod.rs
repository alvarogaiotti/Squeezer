// Copyright (C) 2024 Alvaro Gaiotti
// See end of file for license information

//! The PBN importer, behind the `pbn` feature: [`parse_pbn_file`] builds one
//! [`squeezer_model::record::Board`] per PBN game and [`PbnFile`] keeps them in file
//! order.
//!
//! PBN carries much more than the record model: players, event and scoring
//! tags, result and annotations. The importer is therefore lossy in the same
//! way the LIN importer is, and only the boards survive. A [`PbnFile`] writes
//! back through [`PbnFile::as_pbn`], whose output this importer re-reads.
//!
//! # Documented losses
//!
//! - `[Result]` (tricks won) and a claimed trick count have no counterpart on
//!   [`squeezer_model::record::Board`].
//! - Call alerts and annotations (PBN suffixes, `=n=` note references,
//!   `[Note]` text and `$n` NAGs) are read and discarded; mapping them onto
//!   [`squeezer_model::record::RecordedCall::annotation`] is deferred. An annotation
//!   pasted onto its call (`1S!`) is one token and fails as a call error rather
//!   than being silently reinterpreted.
//! - Every other unmodeled mandatory and supplemental tag (`Event`, `Site`,
//!   `Date`, players, `Scoring`, `Room`, …) is dropped.
//! - An irregular declarer (`^X`) and an end-position contract (a denomination
//!   with no level) are rejected, not modeled.
//! - A `-` unknown card and an out-of-turn lead (PBN's `^L`) are rejected: the
//!   model stores every played card and attributes each play's seat by
//!   rotation from the contract's leader, so there is no representation for
//!   either.
//!
//! The section bodies also require annotations to be whitespace-separated, as
//! PBN's own examples write them; see the parser module for the full grammar
//! and for the tags that are optional on import.

mod parser;
mod types;
pub use parser::*;
pub use types::*;
