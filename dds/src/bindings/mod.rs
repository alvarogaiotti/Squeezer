use squeezer_macros::*;
use std::ffi::c_int;

mod analyseplay;
mod ddserror;
mod ddsffi;
mod deal;
mod future_tricks;
mod solver;
mod traits;
mod utils;
pub use analyseplay::*;
pub use ddserror::DDSError;
pub use deal::*;
pub use solver::*;
pub use traits::*;
pub use utils::*;

const MAXNOOFBOARDSEXPORT: usize = ddsffi::MAXNOOFBOARDS as usize;

pub struct DoubleDummySolver {}

pub struct DDSCalc {}

impl DoubleDummySolver {
    pub fn solver() -> DDSSolver {
        DDSSolver {}
    }

    pub fn play_analyzer() -> impl PlayAnalyzer {
        DDSPlayAnalyzer {}
    }

    pub fn calculator() -> DDSCalc {
        DDSCalc {}
    }

    fn set_max_threads(user_threads: ThreadIndex) {
        unsafe { ddsffi::SetMaxThreads(user_threads.into()) }
    }

    fn set_resources(max_memory_mb: i32, max_threads: ThreadIndex) {
        unsafe { ddsffi::SetResources(max_memory_mb, max_threads.into()) }
    }
}

pub fn dd_score<D: AsDDSDeal, C: AsDDSContract + ContractScorer>(
    deal: &D,
    contract: &C,
) -> Result<i32, Box<dyn std::error::Error>> {
    let solver = DoubleDummySolver::solver();
    let tricks = solver.dd_tricks(deal, contract)?;
    Ok(contract.score(tricks))
}
