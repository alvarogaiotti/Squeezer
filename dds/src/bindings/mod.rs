use squeezer_macros::{AsRawDDS, RawDDSRef, RawDDSRefMut};

mod analyseplay;
mod ddserror;
mod ddsffi;
mod deal;
mod ffi;
mod future_tricks;
mod solver;
mod traits;
mod utils;
pub use analyseplay::*;
pub use ddserror::{DDSError, DDSErrorKind};
pub use deal::*;
pub use ffi::*;
pub use solver::*;
pub use traits::*;
pub use utils::*;

#[allow(clippy::as_conversions)]
/// Max number of boards set by DDS
const MAXNOOFBOARDS: usize = ddsffi::MAXNOOFBOARDS as usize;

#[non_exhaustive]
pub struct DoubleDummySolver;

pub struct DDSCalc;

impl DoubleDummySolver {
    #[must_use]
    pub fn solver() -> DDSSolver {
        DDSSolver {}
    }

    #[must_use]
    pub fn play_analyzer() -> impl PlayAnalyzer {
        DDSPlayAnalyzer::new()
    }

    #[must_use]
    pub fn calculator() -> DDSCalc {
        DDSCalc {}
    }

    fn set_max_threads(user_threads: ThreadIndex) {
        unsafe { ffi::SetMaxThreads(user_threads.into()) }
    }

    fn set_resources(max_memory_mb: i32, max_threads: ThreadIndex) {
        unsafe { ffi::SetResources(max_memory_mb, max_threads.into()) }
    }
}

#[inline]
/// Some thing
///
/// # Errors
///
/// other
pub fn dd_score<D: AsDDSDeal, C: AsDDSContract + ContractScorer>(
    deal: &D,
    contract: &C,
) -> Result<i32, Box<dyn std::error::Error>> {
    let solver = DoubleDummySolver::solver();
    let tricks = solver.dd_tricks(deal, contract)?;
    Ok(contract.score(tricks))
}
