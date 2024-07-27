// Copyright (C) 2024 Alvaro Gaiotti
// See end of file for license information

use ddsffi::RETURN_UNKNOWN_FAULT;
use future_tricks::FutureTricks;

use super::*;
use std::sync::{Mutex, OnceLock, TryLockError};

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
    fn dd_tricks<D: AsDDSDeal, C: AsDDSContract>(
        &self,
        deal: &D,
        contract: &C,
    ) -> Result<u8, DDSError> {
        match self.inner.lock() {
            Ok(guard) => guard.dd_tricks(deal, contract),
            Err(error) => {
                let guard = error.into_inner();
                // Try to recover by freeing the memory, hoping to get clean dll slate.
                guard.free_memory();
                guard.dd_tricks(deal, contract)
            }
        }
    }

    fn dd_tricks_parallel<D: AsDDSDeal, C: AsDDSContract>(
        &self,
        number_of_deals: i32,
        deals: &[D; MAXNOOFBOARDS],
        contract: &[C; MAXNOOFBOARDS],
    ) -> Result<Vec<u8>, DDSError> {
        match self.inner.lock() {
            Ok(guard) => guard.dd_tricks_parallel(number_of_deals, deals, contract),
            Err(error) => {
                let guard = error.into_inner();
                // Try to recover by freeing the memory, hoping to get clean dll slate.
                guard.free_memory();
                guard.dd_tricks_parallel(number_of_deals, deals, contract)
            }
        }
    }
}

impl BridgeSolver for DoubleDummySolver {
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

impl DdTableCalculator for MultiThreadDoubleDummySolver {
    fn calculate_complete_table<T>(
        &self,
        table_deal: &T,
    ) -> Result<DdTableResults<Populated>, DDSError>
    where
        for<'a> &'a T: Into<DdTableDeal>,
    {
        loop {
            match self.inner.try_lock() {
                Ok(guard) => break guard.calculate_complete_table(table_deal),
                Err(TryLockError::WouldBlock) => continue,
                Err(TryLockError::Poisoned(guard)) => {
                    break guard.into_inner().calculate_complete_table(table_deal)
                }
            }
        }
    }

    fn calculate_complete_table_pbn<P>(
        &self,
        table_deal_pbn: &P,
    ) -> Result<DdTableResults<Populated>, DDSError>
    where
        for<'a> &'a P: Into<DdTableDealPbn>,
    {
        loop {
            match self.inner.try_lock() {
                Ok(guard) => break guard.calculate_complete_table_pbn(table_deal_pbn),
                Err(TryLockError::WouldBlock) => continue,
                Err(TryLockError::Poisoned(guard)) => {
                    break guard
                        .into_inner()
                        .calculate_complete_table_pbn(table_deal_pbn)
                }
            }
        }
    }

    fn calculate_all_complete_tables<T>(
        &self,
        table_deals: &[T],
        mode: ParCalcMode,
        trump_filter: TrumpFilter,
    ) -> Result<DdTablesRes<Populated>, DDSError>
    where
        for<'a> &'a T: Into<DdTableDeal>,
    {
        match self.inner.lock() {
            Ok(guard) => guard.calculate_all_complete_tables(table_deals, mode, trump_filter),
            Err(error) => {
                error
                    .into_inner()
                    .calculate_all_complete_tables(table_deals, mode, trump_filter)
            }
        }
    }

    fn calculate_all_complete_tables_pbn<P>(
        &self,
        table_deals_pbn: &[P],
        mode: ParCalcMode,
        trump_filter: TrumpFilter,
    ) -> Result<DdTablesRes<Populated>, DDSError>
    where
        for<'a> &'a P: Into<DdTableDealPbn>,
    {
        match self.inner.lock() {
            Ok(guard) => {
                guard.calculate_all_complete_tables_pbn(table_deals_pbn, mode, trump_filter)
            }
            Err(error) => error.into_inner().calculate_all_complete_tables_pbn(
                table_deals_pbn,
                mode,
                trump_filter,
            ),
        }
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

mod test {
    use super::*;

    const HOLDINGS: [[[u32; 4]; 4]; 3] = [
        [
            // Spades
            [
                // North
                1 << 12 | 1 << 11 | 1 << 6,
                // East
                1 << 8 | 1 << 7 | 1 << 3,
                // South
                1 << 13 | 1 << 5,
                // South
                1 << 14 | 1 << 10 | 1 << 9 | 1 << 4 | 1 << 2,
            ],
            // Hearts
            [
                1 << 13 | 1 << 6 | 1 << 5 | 1 << 2,
                1 << 11 | 1 << 9 | 1 << 7,
                1 << 10 | 1 << 8 | 1 << 3,
                1 << 14 | 1 << 12 | 1 << 4,
            ],
            // Diamonds
            [
                1 << 11 | 1 << 8 | 1 << 5,
                1 << 14 | 1 << 10 | 1 << 7 | 1 << 6 | 1 << 4,
                1 << 13 | 1 << 12 | 1 << 9,
                1 << 3 | 1 << 2,
            ],
            // Clubs
            [
                1 << 10 | 1 << 9 | 1 << 8,
                1 << 12 | 1 << 4,
                1 << 14 | 1 << 7 | 1 << 6 | 1 << 5 | 1 << 2,
                1 << 13 | 1 << 11 | 1 << 3,
            ],
        ],
        [
            [
                1 << 14 | 1 << 13 | 1 << 9 | 1 << 6,
                1 << 12 | 1 << 11 | 1 << 10 | 1 << 5 | 1 << 4 | 1 << 3 | 1 << 2,
                0,
                1 << 8 | 1 << 7,
            ],
            [
                1 << 13 | 1 << 12 | 1 << 8,
                1 << 10,
                1 << 11 | 1 << 9 | 1 << 7 | 1 << 5 | 1 << 4 | 1 << 3,
                1 << 14 | 1 << 6 | 1 << 2,
            ],
            [
                1 << 14 | 1 << 9 | 1 << 8,
                1 << 6,
                1 << 13 | 1 << 7 | 1 << 5 | 1 << 3 | 1 << 2,
                1 << 12 | 1 << 11 | 1 << 10 | 1 << 4,
            ],
            [
                1 << 13 | 1 << 6 | 1 << 3,
                1 << 12 | 1 << 11 | 1 << 8 | 1 << 2,
                1 << 9 | 1 << 4,
                1 << 14 | 1 << 10 | 1 << 7 | 1 << 5,
            ],
        ],
        [
            [
                1 << 7 | 1 << 3,
                1 << 12 | 1 << 10 | 1 << 6,
                1 << 5,
                1 << 14 | 1 << 13 | 1 << 11 | 1 << 9 | 1 << 8 | 1 << 4 | 1 << 2,
            ],
            [
                1 << 12 | 1 << 11 | 1 << 10,
                1 << 8 | 1 << 7 | 1 << 6,
                1 << 14 | 1 << 9 | 1 << 5 | 1 << 4 | 1 << 3 | 1 << 2,
                1 << 13,
            ],
            [
                1 << 14 | 1 << 12 | 1 << 5 | 1 << 4,
                1 << 13 | 1 << 11 | 1 << 9,
                1 << 7 | 1 << 6 | 1 << 3 | 1 << 2,
                1 << 10 | 1 << 8,
            ],
            [
                1 << 10 | 1 << 7 | 1 << 5 | 1 << 2,
                1 << 14 | 1 << 12 | 1 << 8 | 1 << 4,
                1 << 13 | 1 << 6,
                1 << 11 | 1 << 9 | 1 << 3,
            ],
        ],
    ];

    const DDTABLE: [[i32; 20]; 3] = [
        [5, 8, 5, 8, 6, 6, 6, 6, 5, 7, 5, 7, 7, 5, 7, 5, 6, 6, 6, 6],
        [4, 9, 4, 9, 10, 2, 10, 2, 8, 3, 8, 3, 6, 7, 6, 7, 9, 3, 9, 3],
        [3, 10, 3, 10, 9, 4, 9, 4, 8, 4, 8, 4, 3, 9, 3, 9, 4, 8, 4, 8],
    ];

    fn check_all_tables(table: &DdTablesRes<Populated>) {
        for index in 0..3 as usize {
            check_table(&table.results[index], index)
        }
    }

    fn check_table(table: &DdTableResults<Populated>, hand_no: usize) {
        for strain in 0..5 {
            for player in 0..4 {
                assert_eq!(
                    table.res_table[strain][player],
                    DDTABLE[hand_no][4 * strain + player]
                );
            }
        }
    }

    #[test]
    fn test_multithread_calculate_table_works() {
        let mut table_deal = [[0; 4]; 4];
        let solver = MultiThreadDoubleDummySolver::new();
        for deal in 0..3 {
            for h in 0..4 {
                for s in 0..4 {
                    table_deal[h][s] = HOLDINGS[deal][s][h];
                }
            }
            let table = solver.calculate_complete_table(&table_deal).unwrap();
            check_table(&table, deal);
        }
    }

    #[test]
    fn test_multithread_calculate_all_table_works() {
        let mut table_deal = [[[0; 4]; 4]; 3];
        let solver = MultiThreadDoubleDummySolver::new();
        for deal in 0..3 {
            for h in 0..4 {
                for s in 0..4 {
                    table_deal[deal][h][s] = HOLDINGS[deal][s][h];
                }
            }
        }
        let table = solver
            .calculate_all_complete_tables(&table_deal, ParCalcMode::None, [0, 0, 0, 0, 0])
            .unwrap();
        check_all_tables(&table);
    }
}
