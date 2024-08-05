// Copyright (C) 2024 Alvaro Gaiotti
// See end of file for license information

pub(crate) mod ddsffi;
pub(crate) mod ffi;
pub use ffi::*;

#[allow(clippy::as_conversions)]
/// Max number of boards set by DDS
pub const MAXNOOFBOARDS: usize = ddsffi::MAXNOOFBOARDS as usize;
