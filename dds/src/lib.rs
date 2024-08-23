// Copyright (C) 2024 Alvaro Gaiotti
// See end of file for license information
#![warn(clippy::all)]
#![warn(clippy::pedantic)]
#![allow(dead_code, clippy::implicit_return, clippy::module_name_repetitions)]

/// This crate contains all the bindings to Bo Haglund"s DDS library.
/// For further documentation regarding the underlying library, head to [DDS Github repository](https://github.com/dds-bridge/dds)
pub mod analyseplay;

#[allow(
    clippy::unseparated_literal_suffix,
    clippy::missing_docs_in_private_items,
    clippy::unseparated_literal_suffix,
    non_snake_case,
    non_upper_case_globals,
    clippy::pedantic,
    deref_nullptr,
    non_camel_case_types
)]
pub(crate) mod bindings;

pub mod ddserror;
pub mod deal;
pub mod doubledummy;
pub mod future_tricks;
pub mod par;
pub mod solver;
pub mod tables;
pub mod traits;
pub mod utils;
pub use bindings::MAXNOOFBOARDS;
pub use core::ffi::c_int;
