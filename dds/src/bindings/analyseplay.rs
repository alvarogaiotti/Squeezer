use super::{
    ddsffi::{deal, playTraceBin, solvedPlay, solvedPlays, AnalysePlayBin},
    AsDDSContract, AsDDSDeal, RawDDS,
};
use crate::{
    bindings::ddserror::DDSErrorKind, DDSDealConstructionError, DDSError, RankSeq, SuitSeq,
};
use std::ffi::c_int;

#[derive(RawDDS)]
pub struct SolvedPlays {
    #[raw]
    solved_play: solvedPlays,
}

impl SolvedPlays {
    fn new(no_of_boards: c_int) -> Self {
        Self {
            solved_play: solvedPlays {
                noOfBoards: no_of_boards,
                solved: [solvedPlay::new(); 200],
            },
        }
    }
}

impl solvedPlay {
    fn new() -> Self {
        Self {
            number: 0,
            tricks: [0; 53],
        }
    }
}

impl From<solvedPlay> for SolvedPlay {
    fn from(value: solvedPlay) -> Self {
        Self { solved_play: value }
    }
}

#[derive(RawDDS)]
pub struct SolvedPlay {
    #[raw]
    solved_play: solvedPlay,
}

impl SolvedPlay {
    pub fn new() -> Self {
        Self {
            solved_play: solvedPlay::new(),
        }
    }
    pub fn tricks(&self) -> &[i32; 53usize] {
        self.get_tricks()
    }

    fn get_tricks(&self) -> &[i32; 53usize] {
        &self.solved_play.tricks
    }

    pub fn number(&self) -> i32 {
        self.get_number()
    }
    fn get_number(&self) -> i32 {
        self.get_raw().number
    }
}

impl Default for SolvedPlay {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(RawDDS)]
/// Wrapper around DDS `playTraceBin` type
pub struct PlayTraceBin {
    #[raw]
    play_trace_bin: playTraceBin,
}

impl PlayTraceBin {
    /// Provide length of the sequence you want to be analyzed against double dummy, the suit of the
    /// cards played and their's rank.
    pub fn new(suit: SuitSeq, rank: RankSeq) -> Self {
        let length = if suit.length != rank.length {
            i32::min(suit.length, rank.length)
        } else {
            suit.length
        };
        Self {
            play_trace_bin: playTraceBin::new(length, suit.get_raw(), rank.get_raw()),
        }
    }
}

impl playTraceBin {
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
