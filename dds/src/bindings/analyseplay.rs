use crate::{bindings::ddserror::DDSErrorKind, DDSDealConstructionError, DDSError};

use super::{
    ddsffi::{deal, playTraceBin, solvedPlay, AnalysePlayBin},
    AsDDSContract, AsDDSDeal, PlayTraceBin, RawDDS, SolvedPlay,
};
use std::ffi::c_int;

pub trait PlayAnalyzer {
    fn analyze_play<D: AsDDSDeal, C: AsDDSContract>(
        deal: &D,
        contract: C,
        play: &PlayTraceBin,
    ) -> SolvedPlay;
    fn analyze_all_play<D: AsDDSDeal, C: AsDDSContract>(
        deal: Vec<&D>,
        contract: Vec<C>,
        play: &PlayTraceBin,
    ) -> SolvedPlay;
}
struct DDSPlayAnalyzer {}
impl PlayAnalyzer for DDSPlayAnalyzer {
    fn analyze_all_play<D: AsDDSDeal, C: AsDDSContract>(
        deals: Vec<&D>,
        contracts: Vec<C>,
        play: &PlayTraceBin,
    ) -> Result<SolvedPlay, Box<DDSDealConstructionError>> {
        let no_of_boards = deals.len();
        if no_of_boards != contracts.len() {
            return Err(Box::new(DDSDealConstructionError::DealNotLoaded));
        }
        todo!()
    }
    fn analyze_play<D: AsDDSDeal, C: AsDDSContract>(
        deal: &D,
        contract: C,
        play: &PlayTraceBin,
    ) -> SolvedPlay {
        let (trump, first) = contract.as_dds_contract();
        let c_deal = deal {
            trump: trump as c_int,
            first: first as c_int,
            currentTrickSuit: [0; 3],
            currentTrickRank: [0; 3],
            remainCards: deal.as_dds_deal().as_slice(),
        };
        let solved_play = SolvedPlay::new();
        {
            let solved: *mut solvedPlay = &mut solved_play.get_raw();
            let play_trace: *const playTraceBin = &play.get_raw();
            unsafe { AnalysePlayBin(c_deal, *play_trace, solved, 0) };
        }
        solved_play
    }
}
