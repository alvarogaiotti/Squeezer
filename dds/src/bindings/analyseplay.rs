use super::{
    ddsffi::{deal, playTraceBin, solvedPlay, solvedPlays, AnalysePlayBin},
    AsDDSContract, AsDDSDeal, RawDDS,
};
use crate::{DDSDealConstructionError, DDSError, DDSErrorKind, RankSeq, SuitSeq};
use std::ffi::c_int;

/// Wrapper of the [solvedPlays] DoubleDummySolver dll.
/// The `solvedPlays` struct is a container of 200 [solvedPlay]
/// and the number of boards effectively to analyze.
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

/// Wrapper around the [solvedPlay] type from DDS dll.
/// The `solvedPlay` struct stores 53 integers representing
/// the optimal number of tricks which can be made by both
/// side in a given contract after a card is played.
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
/// Wrapper around DDS [playTraceBin] type.
/// The `playTraceBin` stores two arrays
/// of 52 element each representing played card's rank
/// and suit, then an integer stating the real lenght of the play sequence.
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

/// A trait which can be implemented by any stuct capable of doing
/// DD analysis. Simple interface so we can eventually swap other DD solvers
/// in the future. Kinda like a Strategy Pattern. Now depends on dds for the
/// generics with traits used but the idea is to create marker traits for deals and
/// contracts to swap them in.
pub trait PlayAnalyzer {
    /// Analyzes a single hand
    fn analyze_play<D: AsDDSDeal, C: AsDDSContract>(
        deal: &D,
        contract: C,
        play: &PlayTraceBin,
    ) -> SolvedPlay;
    /// Analyzes a bunch of hands, theoretically in paraller.
    fn analyze_all_play<D: AsDDSDeal, C: AsDDSContract>(
        deal: Vec<&D>,
        contract: Vec<C>,
        play: &PlayTraceBin,
    ) -> Result<SolvedPlays, DDSDealConstructionError>;
}

/// Empty struct for DDS solver. We could have other solvers in the future
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
