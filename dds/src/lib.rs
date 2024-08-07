// Copyright (C) 2024 Alvaro Gaiotti
// See end of file for license information

#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(dead_code)]
#![allow(deref_nullptr)]
#![allow(clippy::missing_docs_in_private_items)]
#![allow(clippy::unseparated_literal_suffix)]
#![allow(clippy::all)]
#![allow(clippy::pedantic)]
#![allow(clippy::restriction)]
#![allow(dead_code)]
//#![warn(clippy::restriction, clippy::pedantic)]
#![allow(clippy::unseparated_literal_suffix, clippy::implicit_return)]

pub mod analyseplay;
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
