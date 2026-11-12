// Copyright (C) 2024 Alvaro Gaiotti
// See end of file for license information

//! The LIN importer, behind the `lin` feature: [`parse_bridge_file`] builds one
//! [`squeezer_model::record::Board`] per board and [`LinFile`] keeps the file
//! metadata and the boards in file order. [`parse_lin_documents`] imports a
//! batch of complete LIN documents, the shape a BBO hand download returns, and
//! yields their boards in document order.
//!
//! LIN is a line grammar: file-level `vg|`/`rs|`/`pn|` lines, then per-board
//! `qx|`/`st||`/`md|`/`sv|` lines, then the auction's `mb|` calls and the
//! play's `pc|` cards. The importer is lossy for anything the record model does
//! not represent, and every malformed line fails at import with a
//! [`ParseLinError`] rather than yielding a corrupt board or a silent default.
//! `ParseLinErrorKind` is that error's public taxonomy, and
//! `ParseLinError::detail` carries the exact message explaining the failure.
//! A [`LinFile`] writes back through [`LinFile::as_lin`], whose output this
//! importer re-reads.

mod parser;
mod types;
pub use parser::*;
pub use types::*;
