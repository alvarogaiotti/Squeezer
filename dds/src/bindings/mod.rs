use std::sync::Mutex;

use squeezer_macros::{AsRawDDS, RawDDSRef, RawDDSRefMut};

mod analyseplay;
mod ddserror;
mod ddsffi;
mod deal;
mod doubledummy;
mod ffi;
mod future_tricks;
mod solver;
mod tables;
mod traits;
mod utils;
pub use analyseplay::*;
pub use ddserror::{DDSError, DDSErrorKind};
pub use deal::*;
pub use doubledummy::*;
pub use ffi::*;
pub use solver::*;
pub use traits::*;
pub use utils::*;

#[allow(clippy::as_conversions)]
/// Max number of boards set by DDS
const MAXNOOFBOARDS: usize = ddsffi::MAXNOOFBOARDS as usize;
