use super::{
    ddsffi::{deal, playTraceBin, playTracesBin, solvedPlay, solvedPlays, AnalysePlayBin},
    AsDDSContract, AsDDSDeal, AsRawDDS, Mode, RawDDSRef, RawDDSRefMut, Solutions, Target,
};
use crate::{
    bindings::ddsffi::{boards, AnalyseAllPlaysBin, RETURN_UNKNOWN_FAULT},
    DDSError, RankSeq, SuitSeq,
};
use core::ffi::c_int;
use std::sync::{Mutex, OnceLock};

/// Number of consecutive boards in a sequence a thread gets when we call
/// `AnalyseAllPlaysBin`.
/// 1 means thread1 takes number 1, thread2 takes number 2 and so on
/// 10 means thread1 takes 1..10, thread2 takes 11..20 etc.
const CHUNK_SIZE: i32 = 10;

#[allow(clippy::as_conversions)]
/// Max number of boards set by DDS
const MAXNOOFBOARDS: usize = super::ddsffi::MAXNOOFBOARDS as usize;

#[non_exhaustive]
#[derive(RawDDSRef, RawDDSRefMut)]
/// Wrapper of the `solvedPlays` `DoubleDummySolver` dll.
/// The `solvedPlays` struct is a container of MAXNOOFBOARDS [solvedPlay]
/// and the number of boards effectively to analyze.
pub struct SolvedPlays {
    #[raw]
    /// Wrapped type `solvedPlays`
    solved_plays: solvedPlays,
}

impl SolvedPlays {
    /// Creates a new `SolvedPlays` instance with 0 number of boards
    #[must_use]
    const fn new() -> Self {
        Self {
            solved_plays: solvedPlays {
                noOfBoards: 0i32,
                solved: [solvedPlay::new(); MAXNOOFBOARDS],
            },
        }
    }
}

impl solvedPlay {
    /// Creates a new `solvedPlay` instance
    #[must_use]
    const fn new() -> Self {
        Self {
            number: 0i32,
            tricks: [0i32; 53],
        }
    }
}

impl From<solvedPlay> for SolvedPlay {
    #[inline]
    fn from(value: solvedPlay) -> Self {
        Self { solved_play: value }
    }
}

/// Wrapper around the `solvedPlay` type from DDS dll.
/// The `solvedPlay` struct stores 53 integers representing
/// the optimal number of tricks which can be made by both
/// side in a given contract after a card is played.
#[non_exhaustive]
#[derive(RawDDSRef, Debug)]
pub struct SolvedPlay {
    #[raw]
    pub solved_play: solvedPlay,
}

impl SolvedPlay {
    #[inline]
    #[must_use]
    /// Creates a new `SolvedPlay`
    pub const fn new() -> Self {
        Self {
            solved_play: solvedPlay::new(),
        }
    }

    #[inline]
    #[must_use]
    pub const fn tricks(&self) -> &[i32; 53usize] {
        &self.solved_play.tricks
    }

    #[inline]
    #[must_use]
    pub const fn number(&self) -> i32 {
        self.solved_play.number
    }
}

impl Default for SolvedPlay {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

#[non_exhaustive]
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
    /// Creates a new `playTracesBin`.
    /// The struct is uninitialized, so you'll have to populate it yourself
    pub const fn new() -> Self {
        Self {
            noOfBoards: 0,
            plays: [playTraceBin::new(); MAXNOOFBOARDS],
        }
    }
}

impl PlayTracesBin {
    #[allow(
        clippy::unwrap_in_result,
        clippy::unwrap_used,
        clippy::missing_panics_doc
    )]
    #[inline]
    /// Provide length of the sequence you want to be analyzed against double dummy, the suit of the
    /// cards played and their's rank.
    /// # Errors
    /// Returns an error if suits and ranks have different length
    pub fn from_sequences(suits: Vec<SuitSeq>, ranks: Vec<RankSeq>) -> Result<Self, DDSError> {
        let (suits_len, ranks_len) = (
            suits.len().clamp(0, MAXNOOFBOARDS),
            ranks.len().clamp(0, MAXNOOFBOARDS),
        );
        if suits_len != ranks_len {
            return Err(RETURN_UNKNOWN_FAULT.into());
        }
        let mut plays: Vec<playTraceBin> = suits
            .into_iter()
            .zip(ranks)
            .map(|(suit, rank)| PlayTraceBin::new(suit, rank).play_trace_bin)
            .collect();
        plays.resize(MAXNOOFBOARDS, playTraceBin::new());
        Ok(Self {
            play_trace_bin: playTracesBin {
                // SAFETY: capped at 200
                noOfBoards: suits_len.try_into().unwrap(),
                // SAFETY: We now the length of the Vec
                plays: plays.try_into().unwrap(),
            },
        })
    }
}

#[non_exhaustive]
#[derive(RawDDSRef, AsRawDDS)]
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
    /// Will silently evaluate until the shortest sequence end if their length is different
    pub fn new(suit: SuitSeq, rank: RankSeq) -> Self {
        let length = if suit.length() <= rank.length() {
            suit.length()
        } else {
            rank.length()
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
    pub const fn from(number: c_int, suit: [c_int; 52], rank: [c_int; 52]) -> Self {
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
    /// # Errors
    /// Will return an Error when DDS fails in some way.
    fn analyze_play<D: AsDDSDeal, C: AsDDSContract>(
        &self,
        deal: &D,
        contract: &C,
        play: PlayTraceBin,
    ) -> Result<SolvedPlay, DDSError>;
    /// Analyzes a bunch of hands in paraller.
    /// # Errors
    /// Will return an Error when DDS fails in some way or the deals and contracts vecs have
    /// different length.
    fn analyze_all_plays<D: AsDDSDeal, C: AsDDSContract>(
        &self,
        deals: Vec<&D>,
        contracts: Vec<&C>,
        plays: &mut PlayTracesBin,
    ) -> Result<SolvedPlays, DDSError>;
}

#[non_exhaustive]
pub struct DDSPlayAnalyzer {
    /// Since the functions of DDS are not thread safe, we use a inner raw type
    /// guarded by a `Mutex` to lock the struct and be able to call its methods from different threads
    inner: &'static Mutex<DDSPlayAnalyzerRaw>,
}

impl Default for DDSPlayAnalyzer {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl DDSPlayAnalyzer {
    #[inline]
    #[must_use]
    /// Creates a new `DDSPlayAnalyzer` ready to be used
    pub fn new() -> Self {
        /// The Singleton instance of the raw DDS library
        static INSTANCE: OnceLock<Mutex<DDSPlayAnalyzerRaw>> = OnceLock::new();
        Self {
            inner: INSTANCE.get_or_init(|| Mutex::new(DDSPlayAnalyzerRaw {})),
        }
    }
}

impl PlayAnalyzer for DDSPlayAnalyzer {
    #[inline]
    fn analyze_play<D: AsDDSDeal, C: AsDDSContract>(
        &self,
        deal: &D,
        contract: &C,
        play: PlayTraceBin,
    ) -> Result<SolvedPlay, DDSError> {
        let inner = match self.inner.lock() {
            Ok(guard) => guard,
            Err(poisoned) => {
                use std::fs::File;
                use std::io::Write;
                use std::thread;
                let guard = poisoned.into_inner();
                let mut log = match File::options().create(true).append(true).open("log.txt") {
                    Ok(file) => file,
                    Err(_) => return Err(RETURN_UNKNOWN_FAULT.into()),
                };

                match log.write(
                    format!("Thread {:?} found Mutex poisoned", thread::current().id()).as_bytes(),
                ) {
                    Ok(n) => {
                        if !(0..=70).contains(&n) {
                            return Err(RETURN_UNKNOWN_FAULT.into());
                        }
                    }
                    Err(_) => return Err(RETURN_UNKNOWN_FAULT.into()),
                };
                guard
            }
        };
        inner.analyze_play(deal, contract, play)
    }
    #[inline]
    fn analyze_all_plays<D: AsDDSDeal, C: AsDDSContract>(
        &self,
        deals: Vec<&D>,
        contracts: Vec<&C>,
        plays: &mut PlayTracesBin,
    ) -> Result<SolvedPlays, DDSError> {
        let inner = if let Ok(inner) = self.inner.lock() {
            inner
        } else {
            return Err(RETURN_UNKNOWN_FAULT.into());
        };
        inner.analyze_all_plays(deals, contracts, plays)
    }
}

/// Empty struct for the DDS solver
struct DDSPlayAnalyzerRaw;

impl PlayAnalyzer for DDSPlayAnalyzerRaw {
    #[allow(clippy::unwrap_in_result, clippy::unwrap_used)]
    #[inline]
    fn analyze_all_plays<D: AsDDSDeal, C: AsDDSContract>(
        &self,
        deals: Vec<&D>,
        contracts: Vec<&C>,
        plays: &mut PlayTracesBin,
    ) -> Result<SolvedPlays, DDSError> {
        let deals_len = i32::try_from(deals.len().clamp(0, MAXNOOFBOARDS)).unwrap();
        let contracts_len = i32::try_from(contracts.len().clamp(0, MAXNOOFBOARDS)).unwrap();
        if deals_len != contracts_len || deals_len == 0i32 || contracts_len == 0i32 {
            return Err(RETURN_UNKNOWN_FAULT.into());
        }
        let mut c_deals: Vec<deal> = contracts
            .into_iter()
            .zip(deals)
            .map(construct_dds_deal)
            .collect();
        c_deals.resize(
            MAXNOOFBOARDS,
            deal {
                trump: -1,
                first: -1,
                currentTrickSuit: [-1i32; 3],
                currentTrickRank: [-1i32; 3],
                remainCards: [[0u32; 4]; 4],
            },
        );
        let mut boards = boards {
            noOfBoards: deals_len,
            // We know vec has the right length
            deals: match c_deals.try_into() {
                Ok(ddsdeals) => ddsdeals,
                Err(_) => return Err(RETURN_UNKNOWN_FAULT.into()),
            },
            target: [Target::MaxTricks.into(); MAXNOOFBOARDS],
            solutions: [Solutions::Best.into(); MAXNOOFBOARDS],
            mode: [Mode::Auto.into(); MAXNOOFBOARDS],
        };
        let mut solved_plays = SolvedPlays {
            solved_plays: solvedPlays {
                noOfBoards: deals_len,
                solved: [solvedPlay::new(); MAXNOOFBOARDS],
            },
        };

        let bop: *mut boards = &mut boards;
        let solved: *mut solvedPlays = solved_plays.get_raw_mut();
        let play_trace: *mut playTracesBin = (*plays).get_raw_mut();

        //SAFETY: calling C
        let result = unsafe { AnalyseAllPlaysBin(bop, play_trace, solved, CHUNK_SIZE) };
        match result {
            // RETURN_NO_FAULT == 1i32
            1i32 => Ok(solved_plays),
            n => Err(n.into()),
        }
    }

    #[inline]
    fn analyze_play<D: AsDDSDeal, C: AsDDSContract>(
        &self,
        deal: &D,
        contract: &C,
        play: PlayTraceBin,
    ) -> Result<SolvedPlay, DDSError> {
        let c_deal = construct_dds_deal((contract, deal));
        let mut solved_play = SolvedPlay::new();
        let solved: *mut solvedPlay = &mut solved_play.solved_play;
        let play_trace = play.as_raw();
        // SAFETY: calling an external C function
        let result = unsafe { AnalysePlayBin(c_deal, play_trace, solved, 0) };
        match result {
            1i32 => Ok(solved_play),
            n => Err(n.into()),
        }
    }
}

/// Constructs a DDS deal from a DDS contract and a DDS deal representation
fn construct_dds_deal<D: AsDDSDeal, C: AsDDSContract>(contract_and_deal: (&C, &D)) -> deal {
    let (contract, deal) = contract_and_deal;
    let (trump, first) = contract.as_dds_contract();
    deal {
        trump,
        first,
        currentTrickSuit: [0i32; 3],
        currentTrickRank: [0i32; 3],
        remainCards: deal.as_dds_deal().as_slice(),
    }
}
