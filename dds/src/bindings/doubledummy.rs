// Copyright (C) 2024 Alvaro Gaiotti
// See end of file for license information

use ddsffi::RETURN_UNKNOWN_FAULT;
use future_tricks::FutureTricks;

use super::*;
use std::sync::{Mutex, OnceLock};

use crate::{DoubleDummySolver, ThreadIndex};

#[non_exhaustive]
pub struct MultiThreadDoubleDummySolver {
    inner: &'static Mutex<DoubleDummySolver>,
}

impl Default for MultiThreadDoubleDummySolver {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl MultiThreadDoubleDummySolver {
    #[inline]
    #[must_use]
    /// Creates a new `DDSPlayAnalyzer` ready to be used
    pub fn new() -> Self {
        /// The Singleton instance of the raw DDS library
        static INSTANCE: OnceLock<Mutex<DoubleDummySolver>> = OnceLock::new();
        Self {
            inner: INSTANCE.get_or_init(|| Mutex::new(DoubleDummySolver {})),
        }
    }
    fn set_max_threads(user_threads: ThreadIndex) {
        unsafe { super::ffi::SetMaxThreads(user_threads.into()) }
    }

    fn set_resources(max_memory_mb: i32, max_threads: ThreadIndex) {
        unsafe { super::ffi::SetResources(max_memory_mb, max_threads.into()) }
    }
}

impl BridgeSolver for MultiThreadDoubleDummySolver {
    #[inline]
    fn dd_tricks<D: AsDDSDeal, C: AsDDSContract>(
        &self,
        deal: &D,
        contract: &C,
    ) -> Result<u8, DDSError> {
        let c_deal = build_c_deal((contract, deal))?;
        let mut future_tricks = FutureTricks::new();
        let futp: *mut FutureTricks = &mut future_tricks;
        let result;
        unsafe {
            result = SolveBoard(
                c_deal,
                Target::MaxTricks.into(),
                Solutions::Best.into(),
                Mode::AutoSearchAlways.into(),
                futp,
                ThreadIndex::Auto.into(),
            );
        };
        if result != 1 {
            return Err(DDSError::from(result));
        }
        return Ok(13 - future_tricks.score()[0] as u8);
    }

    fn dd_tricks_parallel<D: AsDDSDeal, C: AsDDSContract>(
        &self,
        number_of_deals: i32,
        deals: &[D; MAXNOOFBOARDS],
        contract: &[C; MAXNOOFBOARDS],
    ) -> Result<Vec<u8>, DDSError> {
        assert!(number_of_deals <= MAXNOOFBOARDS as i32);
        let mut boards = Boards::new(
            number_of_deals,
            deals,
            contract,
            &[Target::MaxTricks; MAXNOOFBOARDS],
            &[Solutions::Best; MAXNOOFBOARDS],
            &[Mode::Auto; MAXNOOFBOARDS],
        );
        let mut solved_boards = SolvedBoards::new(number_of_deals);
        let result;
        {
            let bop: *mut Boards = &mut boards;
            let solved_boards_ptr: *mut SolvedBoards = &mut solved_boards;
            unsafe {
                result = SolveAllChunksBin(bop, solved_boards_ptr, 1);
            }
        };
        if result != 1 {
            return Err(result.into());
        }
        Ok(solved_boards
            .into_iter()
            .map(|ft| ft.score[0] as u8)
            .take(number_of_deals as usize)
            .collect())
    }
}

impl PlayAnalyzer for MultiThreadDoubleDummySolver {
    #[inline]
    fn analyze_play<D: AsDDSDeal, C: AsDDSContract>(
        &self,
        deal: &D,
        contract: &C,
        play: PlayTraceBin,
    ) -> Result<SolvedPlay, DDSError> {
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
        deals: &Vec<D>,
        contracts: &Vec<C>,
        plays: &mut PlayTracesBin,
    ) -> Result<SolvedPlays, DDSError> {
        if let Ok(inner) = self.inner.lock() {
            inner.analyze_all_plays(deals, contracts, plays)
        } else {
            Err(RETURN_UNKNOWN_FAULT.into())
        }
    }
}

impl PlayAnalyzer for DoubleDummySolver {
    #[allow(clippy::unwrap_in_result, clippy::unwrap_used)]
    #[inline]
    fn analyze_all_plays<D: AsDDSDeal, C: AsDDSContract>(
        &self,
        deals: &Vec<D>,
        contracts: &Vec<C>,
        plays: &mut PlayTracesBin,
    ) -> Result<SolvedPlays, DDSError> {
        let deals_len = i32::try_from(deals.len().clamp(0, MAXNOOFBOARDS)).unwrap();
        let contracts_len = i32::try_from(contracts.len().clamp(0, MAXNOOFBOARDS)).unwrap();
        if deals_len != contracts_len
            || deals_len == 0i32
            || contracts_len == 0i32
            || deals_len != plays.len() as i32
        {
            return Err(RETURN_UNKNOWN_FAULT.into());
        }
        let mut c_deals: Vec<DdsDeal> = match contracts
            .iter()
            .zip(deals.iter())
            .map(build_c_deal)
            .collect::<Result<Vec<_>, DDSDealConstructionError>>()
        {
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
        let mut solved_plays = SolvedPlays {
            no_of_boards: deals_len,
            solved: [SolvedPlay::new(); MAXNOOFBOARDS],
        };

        let bop: *mut Boards = &mut boards;
        let solved: *mut SolvedPlays = &mut solved_plays;
        let play_trace: *mut PlayTracesBin = plays;

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
        let dds_deal = match build_c_deal((contract, deal)) {
            Ok(dds_deal) => dds_deal,
            Err(_) => return Err(RETURN_UNKNOWN_FAULT.into()),
        };
        let mut solved_play = SolvedPlay::new();
        let solved: *mut SolvedPlay = &mut solved_play;
        let play_trace = play;
        // SAFETY: calling an external C function
        let result = unsafe { AnalysePlayBin(dds_deal, play_trace, solved, 0) };
        match result {
            // RETURN_NO_FAULT == 1i32
            1i32 => Ok(solved_play),
            n => Err(n.into()),
        }
    }
}
