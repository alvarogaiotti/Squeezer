// Copyright (C) 2024 Alvaro Gaiotti
// See end of file for license information

//! Outbound serialization for the record types.
//!
//! The [`Deal`](squeezer_model::Deal) and [`Board`](squeezer_model::Board)
//! writers live in `squeezer-model` and are re-exported here. The `LinFile` and
//! `PbnFile` container writers live with their importer types in `squeezer-io`,
//! behind the `lin` and `pbn` features.

pub use squeezer_model::record::serialize::*;
