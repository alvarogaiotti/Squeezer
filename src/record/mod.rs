// Copyright (C) 2024 Alvaro Gaiotti
// See end of file for license information

//! Format-independent bridge record types, re-exported from `squeezer-model`.
//!
//! The deal, board, auction, play and position vocabulary and their `Deal` and
//! `Board` writers live in the model crate. This module keeps the historical
//! `squeezer::record::...` paths for the parsers and solver that still build
//! these types, including their module paths (`record::auction::Call`,
//! `record::position::trick_winner`).
//!
//! The `LinFile` and `PbnFile` container writers stay in [`serialize`]: their
//! types belong to the importers, which this crate still owns.

/// The auction vocabulary; see [`squeezer_model::record::auction`].
pub use squeezer_model::record::auction;
/// The board record; see [`squeezer_model::record::board`].
pub use squeezer_model::record::board;
/// The deal and its four-seat invariant; see [`squeezer_model::record::deal`].
pub use squeezer_model::record::deal;
/// The recorded play; see [`squeezer_model::record::play`].
pub use squeezer_model::record::play;
/// Partially played positions and the trick-winner rule; see
/// [`squeezer_model::record::position`].
pub use squeezer_model::record::position;
/// The `Deal`/`Board` writers, plus this crate's file-container writers.
pub mod serialize;

pub use squeezer_model::record::*;
