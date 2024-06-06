use super::{
    ddsffi::{
        playTraceBin, playTracesBin, solvedPlay, solvedPlays, AnalyseAllPlaysBin, AnalysePlayBin,
        RETURN_UNKNOWN_FAULT,
    },
    utils::build_c_deal,
    AsDDSContract, AsDDSDeal, AsRawDDS, Boards, DdsDeal, Mode, RawDDSRef, RawDDSRefMut, Solutions,
    Target, MAXNOOFBOARDS,
};
use crate::{DDSError, RankSeq, SuitSeq};
use core::{ffi::c_int, slice::Iter};
use std::sync::{Mutex, OnceLock};

/// Number of consecutive boards in a sequence a thread gets when we call
/// `AnalyseAllPlaysBin`.
/// 1 means thread1 takes number 1, thread2 takes number 2 and so on
/// 10 means thread1 takes 1..10, thread2 takes 11..20 etc.
const CHUNK_SIZE: i32 = 10;

impl solvedPlay {
    /// Creates a new `solvedPlay` instance
    #[inline]
    #[must_use]
    const fn new() -> Self {
        Self {
            number: 0i32,
            tricks: [-1i32; 53],
        }
    }

    #[inline]
    /// Returns a `core::slice::Iter` over the tricks.
    fn iter(&self) -> Iter<'_, i32> {
        self.tricks[..self
            .number
            .try_into()
            .expect("it's a lenght so it's always positive")]
            .iter()
    }
}

impl IntoIterator for solvedPlay {
    type Item = i32;
    type IntoIter = core::array::IntoIter<Self::Item, 53>;

    fn into_iter(self) -> Self::IntoIter {
        self.tricks.into_iter()
    }
}

impl solvedPlay {
    #[inline]
    #[must_use]
    pub const fn tricks(&self) -> &[i32; 53usize] {
        &self.tricks
    }

    #[inline]
    #[must_use]
    pub const fn number(&self) -> i32 {
        self.number
    }

    /// Function for testing purposes and should not be used.
    #[must_use]
    pub fn from_seq(mut seq: Vec<i32>) -> Self {
        let number = seq.len() as i32;
        seq.resize(53, -1i32);
        Self {
            number,
            tricks: seq.try_into().expect("just resized"),
        }
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

impl Default for playTracesBin {
    fn default() -> Self {
        Self::new()
    }
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
    #[inline]
    pub fn len(&self) -> usize {
        self.play_trace_bin.noOfBoards.try_into().unwrap()
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

impl Default for playTraceBin {
    fn default() -> Self {
        Self::new()
    }
}

impl playTraceBin {
    #[inline]
    #[must_use]
    /// Creates a new `playTraceBin` from data
    const fn from(number: c_int, suit: [c_int; 52], rank: [c_int; 52]) -> Self {
        Self { number, suit, rank }
    }
    /// Creates a new `playTraceBin`
    pub const fn new() -> Self {
        Self {
            number: 0,
            suit: [-1i32; 52],
            rank: [-1i32; 52],
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
    ) -> Result<solvedPlay, DDSError>;
    /// Analyzes a bunch of hands in paraller.
    /// # Errors
    /// Will return an Error when DDS fails in some way or the deals and contracts vecs have
    /// different length.
    fn analyze_all_plays<D: AsDDSDeal, C: AsDDSContract>(
        &self,
        deals: Vec<&D>,
        contracts: Vec<&C>,
        plays: &mut PlayTracesBin,
    ) -> Result<solvedPlays, DDSError>;
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
    ) -> Result<solvedPlay, DDSError> {
        let inner = if let Ok(guard) = self.inner.lock() {
            guard
        } else {
            #[allow(clippy::print_stderr, clippy::use_debug)]
            {
                use std::thread;
                eprintln!("Thread {:?} found Mutex poisoned", thread::current().id());
                return Err(RETURN_UNKNOWN_FAULT.into());
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
    ) -> Result<solvedPlays, DDSError> {
        if let Ok(inner) = self.inner.lock() {
            inner.analyze_all_plays(deals, contracts, plays)
        } else {
            Err(RETURN_UNKNOWN_FAULT.into())
        }
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
    ) -> Result<solvedPlays, DDSError> {
        let deals_len = i32::try_from(deals.len().clamp(0, MAXNOOFBOARDS)).unwrap();
        let contracts_len = i32::try_from(contracts.len().clamp(0, MAXNOOFBOARDS)).unwrap();
        if deals_len != contracts_len
            || deals_len == 0i32
            || contracts_len == 0i32
            || deals_len != plays.len() as i32
        {
            return Err(RETURN_UNKNOWN_FAULT.into());
        }
        let mut c_deals: Vec<DdsDeal> =
            match contracts.into_iter().zip(deals).map(build_c_deal).collect() {
                Ok(vec) => vec,
                Err(_) => return Err(RETURN_UNKNOWN_FAULT.into()),
            };
        c_deals.resize(MAXNOOFBOARDS, DdsDeal::new());
        let mut boards = Boards {
            no_of_boards: deals_len,
            // We know vec has the right length
            deals: match c_deals.into_iter().collect::<Vec<DdsDeal>>().try_into() {
                Ok(ddsdeals) => ddsdeals,
                Err(_) => return Err(RETURN_UNKNOWN_FAULT.into()),
            },
            target: [Target::MaxTricks.into(); MAXNOOFBOARDS],
            solutions: [Solutions::Best.into(); MAXNOOFBOARDS],
            mode: [Mode::Auto.into(); MAXNOOFBOARDS],
        };
        let mut solved_plays = solvedPlays {
            noOfBoards: deals_len,
            solved: [solvedPlay::new(); MAXNOOFBOARDS],
        };

        let bop: *mut Boards = &mut boards;
        let solved: *mut solvedPlays = &mut solved_plays;
        let play_trace: *mut playTracesBin = plays.get_raw_mut();

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
    ) -> Result<solvedPlay, DDSError> {
        let dds_deal = match build_c_deal((contract, deal)) {
            Ok(dds_deal) => dds_deal,
            Err(_) => return Err(RETURN_UNKNOWN_FAULT.into()),
        };
        let mut solved_play = solvedPlay::new();
        let solved: *mut solvedPlay = &mut solved_play;
        let play_trace = play.as_raw();
        // SAFETY: calling an external C function
        let result = unsafe { AnalysePlayBin(dds_deal, play_trace, solved, 0) };
        match result {
            // RETURN_NO_FAULT == 1i32
            1i32 => Ok(solved_play),
            n => Err(n.into()),
        }
    }
}
