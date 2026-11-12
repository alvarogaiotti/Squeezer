// Copyright (C) 2024 Alvaro Gaiotti
// See end of file for license information

//! Outbound serialization for the record types.
//!
//! The [`Deal`](crate::Deal) and [`Board`](crate::Board) writers live in
//! `squeezer-model`; this module re-exports them and keeps the container
//! writers for `LinFile` and `PbnFile`, whose types belong to the importers in
//! this crate.

pub use squeezer_model::record::serialize::*;

mod containers;

#[cfg(test)]
mod tests;
