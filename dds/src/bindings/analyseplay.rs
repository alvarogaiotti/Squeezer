use super::{
    ddsffi::{deal, playTraceBin, solvedPlay, AnalysePlayBin},
    AsDDSContract, AsDDSDeal, RawDDS, SolvedPlay,
};
use crate::{bindings::ddserror::DDSErrorKind, DDSDealConstructionError, DDSError, SolvedPlays};
use std::ffi::c_int;

#[derive(RawDDS)]
pub struct PlayTraceBin {
    #[raw]
    play_trace_bin: playTraceBin,
}

impl PlayTraceBin {
    pub fn new(number: c_int, suit: [c_int; 52], rank: [c_int; 52]) -> Self {
        Self {
            play_trace_bin: playTraceBin::new(number, suit, rank),
        }
    }
}

impl playTraceBin {
    /// Provide length of the sequence you want to be analyzed against double dummy, the suit of the
    /// cards played and their's rank.
    pub fn new(number: c_int, suit: [c_int; 52], rank: [c_int; 52]) -> Self {
        Self { number, suit, rank }
    }
}

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
    ) -> Result<SolvedPlays, DDSDealConstructionError>;
}
pub struct DDSPlayAnalyzer {}
impl PlayAnalyzer for DDSPlayAnalyzer {
    fn analyze_all_play<D: AsDDSDeal, C: AsDDSContract>(
        deals: Vec<&D>,
        contracts: Vec<C>,
        play: &PlayTraceBin,
    ) -> Result<SolvedPlays, DDSDealConstructionError> {
        let no_of_boards = deals.len();
        if no_of_boards != contracts.len() {
            return Err(DDSDealConstructionError::DealNotLoaded);
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
            let play_trace: *const playTraceBin = &(play.get_raw());
            unsafe { AnalysePlayBin(c_deal, *play_trace, solved, 0) };
        }
        solved_play
    }
}
