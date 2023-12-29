use super::{
    ddsffi::{
        deal, playTraceBin, playTracesBin, solvedPlay, solvedPlays, AnalysePlayBin, RETURN_NO_FAULT,
    },
    AsDDSContract, AsDDSDeal, AsRawDDS, Mode, RawDDSRef, RawDDSRefMut, Solutions, Target,
};
use crate::{
    bindings::ddsffi::{boards, AnalyseAllPlaysBin, RETURN_UNKNOWN_FAULT},
    DDSDealConstructionError, DDSError, RankSeq, SuitSeq,
};
use core::ffi::c_int;

const MAXNOOFBOARDS: usize = super::ddsffi::MAXNOOFBOARDS as usize;

/// Wrapper of the `solvedPlays` `DoubleDummySolver` dll.
/// The `solvedPlays` struct is a container of MAXNOOFBOARDS [solvedPlay]
/// and the number of boards effectively to analyze.
#[derive(RawDDSRef, RawDDSRefMut)]
pub struct SolvedPlays {
    #[raw]
    solved_plays: solvedPlays,
}

impl SolvedPlays {
    fn new(no_of_boards: c_int) -> Self {
        Self {
            solved_plays: solvedPlays {
                noOfBoards: no_of_boards,
                solved: [solvedPlay::new(); MAXNOOFBOARDS as usize],
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

/// Wrapper around the `solvedPlay` type from DDS dll.
/// The `solvedPlay` struct stores 53 integers representing
/// the optimal number of tricks which can be made by both
/// side in a given contract after a card is played.
#[derive(RawDDSRef, Debug)]
pub struct SolvedPlay {
    #[raw]
    pub solved_play: solvedPlay,
}

impl SolvedPlay {
    #[must_use]
    pub fn new() -> Self {
        Self {
            solved_play: solvedPlay::new(),
        }
    }
    #[must_use]
    pub fn tricks(&self) -> &[i32; 53_usize] {
        return self.get_tricks();
    }

    fn get_tricks(&self) -> &[i32; 53usize] {
        &self.solved_play.tricks
    }

    #[must_use]
    pub fn number(&self) -> i32 {
        self.solved_play.number
    }
}

impl Default for SolvedPlay {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(RawDDSRef, RawDDSRefMut)]
/// Wrapper around DDS [`playTracesBin`] type.
/// The `playTraceBin` stores two arrays
/// of 52 element each representing played card's rank
/// and suit, then an integer stating the real lenght of the play sequence.
pub struct PlayTracesBin {
    #[raw]
    pub play_trace_bin: playTracesBin,
}

impl playTracesBin {
    pub const fn new() -> Self {
        Self {
            noOfBoards: 0,
            plays: [playTraceBin::new(); MAXNOOFBOARDS],
        }
    }
}

impl PlayTracesBin {
    #[allow(clippy::unwrap_in_result, clippy::unwrap_used)]
    #[inline]
    #[must_use]
    /// Provide length of the sequence you want to be analyzed against double dummy, the suit of the
    /// cards played and their's rank.
    pub fn from_sequences(suits: Vec<SuitSeq>, ranks: Vec<RankSeq>) -> Result<Self, DDSError> {
        let (suits_len, ranks_len) = (suits.len().clamp(1, 200), ranks.len().clamp(1, 200));
        if suits_len != ranks_len {
            return Err(DDSError::new(RETURN_UNKNOWN_FAULT));
        }
        let mut plays: Vec<playTraceBin> = suits
            .into_iter()
            .zip(ranks)
            .map(|(suit, rank)| PlayTraceBin::new(suit, rank).play_trace_bin)
            .collect();
        plays.resize(MAXNOOFBOARDS, playTraceBin::new());
        let plays = plays.try_into().unwrap();
        Ok(Self {
            play_trace_bin: playTracesBin {
                // SAFETY: capped at 200
                noOfBoards: suits_len.try_into().unwrap(),
                // SAFETY: We now the length of the Vec
                plays,
            },
        })
    }
}

#[derive(RawDDSRef)]
/// Wrapper around DDS [`playTraceBin`] type.
/// The `playTraceBin` stores two arrays
/// of 52 element each representing played card's rank
/// and suit, then an integer stating the real lenght of the play sequence.
pub struct PlayTraceBin {
    #[raw]
    pub play_trace_bin: playTraceBin,
}

impl PlayTraceBin {
    #[inline]
    #[must_use]
    /// Provide length of the sequence you want to be analyzed against double dummy, the suit of the
    /// cards played and their's rank.
    pub fn new(suit: SuitSeq, rank: RankSeq) -> Self {
        let length = if suit.length == rank.length {
            suit.length
        } else {
            i32::min(suit.length, rank.length)
        };
        Self {
            play_trace_bin: {
                let number = length;
                playTraceBin {
                    number,
                    suit: suit.as_raw(),
                    rank: rank.as_raw(),
                }
            },
        }
    }
}

impl playTraceBin {
    #[inline]
    #[must_use]
    /// Creates a new `playTraceBin` from data
    pub fn from(number: c_int, suit: [c_int; 52], rank: [c_int; 52]) -> Self {
        Self { number, suit, rank }
    }
    /// Creates a new `playTraceBin`
    pub const fn new() -> Self {
        Self {
            number: 0,
            suit: [0i32; 52],
            rank: [0i32; 52],
        }
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
    ) -> Result<SolvedPlay, DDSError>;
    /// Analyzes a bunch of hands, theoretically in paraller.
    fn analyze_all_plays<D: AsDDSDeal, C: AsDDSContract>(
        deals: Vec<&D>,
        contracts: Vec<C>,
        plays: &mut PlayTracesBin,
    ) -> Result<SolvedPlays, DDSError>;
}

#[non_exhaustive]
/// Empty struct for DDS solver. We could have other solvers in the future
pub struct DDSPlayAnalyzer;

impl Default for DDSPlayAnalyzer {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}
impl DDSPlayAnalyzer {
    #[inline]
    #[must_use]
    pub fn new() -> Self {
        DDSPlayAnalyzer {}
    }
}

impl PlayAnalyzer for DDSPlayAnalyzer {
    #[inline]
    fn analyze_all_plays<D: AsDDSDeal, C: AsDDSContract>(
        deals: Vec<&D>,
        contracts: Vec<C>,
        plays: &mut PlayTracesBin,
    ) -> Result<SolvedPlays, DDSError> {
        let deals_len = if let Ok(deals_len) = i32::try_from(deals.len()) {
            deals_len
        } else {
            return Err(DDSError::new(RETURN_UNKNOWN_FAULT));
        };
        let contracts_len = if let Ok(contracts_len) = i32::try_from(contracts.len()) {
            contracts_len
        } else {
            return Err(DDSError::new(RETURN_UNKNOWN_FAULT));
        };
        if deals_len != contracts_len {
            return Err(DDSError::new(RETURN_UNKNOWN_FAULT));
        }
        let mut c_deals: Vec<deal> = contracts
            .into_iter()
            .zip(deals)
            .map(|(contract, deal)| construct_dds_deal(contract, deal))
            .collect();
        c_deals.resize(MAXNOOFBOARDS, deal::new());
        let mut boards = boards {
            noOfBoards: deals_len,
            // SAFETY: We now vec has the right length
            deals: match c_deals.try_into() {
                Ok(ddsdeals) => ddsdeals,
                Err(_) => return Err(DDSError::new(RETURN_UNKNOWN_FAULT)),
            },
            target: [Target::MaxTricks.into(); MAXNOOFBOARDS],
            solutions: [Solutions::Best.into(); MAXNOOFBOARDS],
            mode: [Mode::Auto.into(); MAXNOOFBOARDS],
        };
        let bop: *mut boards = &mut boards;
        let mut solved_plays = SolvedPlays {
            solved_plays: solvedPlays {
                noOfBoards: deals_len,
                solved: [solvedPlay::new(); MAXNOOFBOARDS],
            },
        };
        let solved: *mut solvedPlays = solved_plays.get_raw_mut();
        let play_trace: *mut playTracesBin = (*plays).get_raw_mut();

        //SAFETY: calling C
        let result = unsafe { AnalyseAllPlaysBin(bop, play_trace, solved, 1i32) };
        match result {
            success if success == RETURN_NO_FAULT as i32 => Ok(solved_plays),
            n => Err(n.into()),
        }
    }

    #[inline]
    fn analyze_play<D: AsDDSDeal, C: AsDDSContract>(
        deal: &D,
        contract: C,
        play: &PlayTraceBin,
    ) -> Result<SolvedPlay, DDSError> {
        let c_deal = construct_dds_deal(contract, deal);
        let mut solved_play = SolvedPlay::new();
        let solved: *mut solvedPlay = &mut solved_play.solved_play;
        let play_trace = play.get_raw();
        // SAFETY: calling an external C function
        let result = unsafe { AnalysePlayBin(c_deal, *play_trace, solved, 0) };
        match result {
            1i32 => Ok(solved_play),
            n => Err(n.into()),
        }
    }
}

fn construct_dds_deal<D: AsDDSDeal, C: AsDDSContract>(contract: C, deal: &D) -> deal {
    let (trump, first) = contract.as_dds_contract();
    deal {
        trump,
        first,
        currentTrickSuit: [0i32; 3],
        currentTrickRank: [0i32; 3],
        remainCards: deal.as_dds_deal().as_slice(),
    }
}
