// Copyright (C) 2024 Alvaro Gaiotti
// See end of file for license information

//! Deal generation, re-exported from `squeezer-model`.
//!
//! The dealer lives in the model crate now, so `squeezer::dealer::...` and the
//! flat prelude exports keep working without this crate owning the code.

pub use squeezer_model::dealer::*;
