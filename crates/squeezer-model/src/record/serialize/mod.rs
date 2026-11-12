// Copyright (C) 2024 Alvaro Gaiotti
// See end of file for license information

//! Outbound serialization for the record types.
//!
//! Writing needs no parser and no native solver, so this module is
//! unconditional: a consumer can export a [`crate::record::Board`] or
//! [`crate::record::Deal`] without an import parser (and therefore without
//! `nom`). The LIN importer lives in `squeezer-io` behind `lin`, and the
//! `LinFile`/`PbnFile` container writers stay with those parser-side types.
//!
//! # Formats
//!
//! A board has two interchange formats, so there is deliberately no
//! `impl Display for Board`: an implicit format choice is a footgun. Call
//! [`crate::record::Board::as_lin`] explicitly.
//!
//! The writers are **lossy** for anything the model does not represent. A LIN
//! board can carry arbitrary tags, comments and formatting that
//! [`crate::record::Board`] does not keep; the output re-emits only the
//! modeled tags. LIN's file metadata (`vg|`/`rs|`/`pn|`) is comma-separated
//! with no escaping, so a name, team or result containing a comma does not
//! round-trip.

use crate::prelude::*;

pub mod lin;
pub mod pbn;

/// Renders a writer into a fresh `String`.
///
/// The `as_*` methods all share this shape; the `expect` is sound because
/// [`fmt::Write`] for [`String`] never fails.
fn written_string(capacity: usize, write: impl FnOnce(&mut String) -> fmt::Result) -> String {
    let mut out = String::with_capacity(capacity);
    write(&mut out).expect("writing to a String is infallible");
    out
}

#[cfg(test)]
pub(crate) mod fixtures {
    use crate::prelude::*;

    /// A deal with one whole suit per seat, so a golden hand listing is easy to
    /// read and each hand exercises three void suits.
    pub(crate) fn one_suit_per_seat() -> Deal {
        Deal::new([
            Cards::from_str("SAKQJT98765432").expect("spades hand parses"),
            Cards::from_str("HAKQJT98765432").expect("hearts hand parses"),
            Cards::from_str("DAKQJT98765432").expect("diamonds hand parses"),
            Cards::from_str("CAKQJT98765432").expect("clubs hand parses"),
        ])
        .expect("one suit per seat is a valid deal")
    }
}
