// Copyright (C) 2024 Alvaro Gaiotti
// See end of file for license information

//! Format-independent bridge record types.
//!
//! This module is deliberately free of import parsers and `dds`: it models a
//! deal, a recorded board, its auction and its play using only core card
//! types, so the same types can be built by the LIN importer, by the dealer, or
//! by hand without pulling in a parser or a solver.

pub mod auction;
pub mod board;
pub mod deal;
pub mod play;
pub mod position;
pub mod serialize;

pub use auction::*;
pub use board::*;
pub use deal::*;
pub use play::*;
pub use position::*;

/// Highest board count a LIN or PBN file may carry.
///
/// This bounds a recorded board, so it lives with the record layer rather than
/// with the hand model.
pub const MAX_N_OF_BOARDS: u8 = 128;
