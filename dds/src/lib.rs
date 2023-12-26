#![allow(dead_code)]
#![warn(clippy::restriction, clippy::pedantic)]
#![allow(clippy::unseparated_literal_suffix, clippy::implicit_return)]

pub mod bindings;

pub use bindings::*;
pub use core::ffi::c_int;
