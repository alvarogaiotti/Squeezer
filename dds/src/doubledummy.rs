// Copyright (C) 2024 Alvaro Gaiotti
// See end of file for license information

use std::sync::{Mutex, OnceLock, TryLockError};

use crate::{
    analyseplay::{PlayAnalyzer, PlayTraceBin, PlayTracesBin, SolvedPlay, SolvedPlays, CHUNK_SIZE},
    bindings::{
        ddsffi::RETURN_UNKNOWN_FAULT, AnalyseAllPlaysBin, AnalysePlayBin, SolveAllChunksBin,
        SolveBoard, MAXNOOFBOARDS,
    },
    ddserror::DDSError,
    deal::{AsDDSDeal, Boards, DDSDealConstructionError, DdsDeal},
    future_tricks::FutureTricks,
    solver::{BridgeSolver, SolvedBoards},
    tables::{
        DdTableCalculator, DdTableDeal, DdTableDealPbn, DdTableResults, DdTablesRes, ParCalcMode,
        Populated, TrumpFilter,
    },
    traits::AsDDSContract,
    utils::{build_c_deal, Mode, Solutions, Target, ThreadIndex},
};

/// A single threaded double dummy solver.
/// DO NOT USE this in multithreaded code as it will SEGFAULT.
/// YOU ARE RESPONSIBLE FOR THIS. If you want to be sure and not have to worry,
/// use the [`crate::MultiThreadDoubleDummySolver`], which manages the DLL's
/// resources for you.
/// This struct implements all the needed traits linked to a double dummy analysis
/// such as:
/// - Play analysis [`crate::PlayAnalyzer`]
/// - Par calculation [`crate::DdTableCalculator`]
/// - Board solution [`crate::BridgeSolver`]
///
/// To use it, import the required trait, instantiate this struct and use it.
///
/// # Examples
/// Since this crate is just a wrapper around a DLL it is hard to provide a good example,
/// but the usage is something about like so:
/// ```
/// # use dds::solver::BridgeSolver;
/// # use dds::doubledummy::DoubleDummySolver;
/// # #[derive(Debug, Clone)]
/// # pub struct DealMock {
/// #     pub hands: [[usize; 4]; 4],
/// # }
/// # impl IntoIterator for DealMock {
/// #     type Item = [usize; 4];
/// #     type IntoIter = std::array::IntoIter<[usize; 4], 4>;
/// #     fn into_iter(self) -> Self::IntoIter {
/// #         self.hands.into_iter()
/// #     }
/// # }
/// #
/// # impl dds::deal::AsDDSDeal for DealMock {
/// #     fn as_dds_deal(&self) -> dds::deal::DDSDealRepr {
/// #         let mut remain_cards = [[0; 4]; 4];
/// #         for (seat, hand) in self.clone().into_iter().enumerate() {
/// #             for (index, suit) in hand.into_iter().enumerate() {
/// #                 remain_cards[seat][index] = (suit >> (16 * index)) as u32;
/// #             }
/// #         }
/// #         remain_cards.into()
/// #     }
/// # }
///
/// # #[derive(Debug, Copy, Clone)]
/// # pub struct ContractMock {}
/// #
/// # impl dds::traits::ContractScorer for ContractMock {
/// #     fn score(&self, _tricks: u8) -> i32 {
/// #         0
/// #     }
/// # }
/// #
/// # impl dds::traits::AsDDSContract for ContractMock {
/// #     fn as_dds_contract(&self) -> (i32, i32) {
/// #         (2, 3)
/// #     }
/// # }
///
///    /*
///           ♠K93
///           ♡JT9862
///           ♢9
///           ♣K73
///
///    ♠T4           ♠AQJ
///    ♡Q            ♡75
///    ♢KQT543       ♢AJ2
///    ♣QT85         ♣J9642
///
///           ♠87652
///           ♡AK43
///           ♢876
///           ♣A
///    */
///
///    let deal = DealMock {
///        hands: [
///            [8712, 256114688, 2199023255552, 2344123606046343168],
///            [22528, 10485760, 79182017069056, 744219838422974464],
///            [484, 1612185600, 1924145348608, 4611686018427387904],
///            [1040, 268435456, 57415122812928, 1522216674051227648],
///        ],
///    };
///    let contract = ContractMock {};
///    let solver = DoubleDummySolver::new();
///    println!("{}", solver.dd_tricks(&deal, &contract).unwrap());
pub struct DoubleDummySolver {
    _marker: std::marker::PhantomData<std::cell::Cell<()>>,
}

impl DoubleDummySolver {
    /// Function to free the dinamically allocated memory of the DDS's DLL.
    /// Use this if you are not gonna use DDS for a long time and want to free some memory.
    pub fn free_memory(&self) {
        unsafe { crate::bindings::FreeMemory() }
    }

    #[inline]
    #[must_use]
    pub fn new() -> Self {
        Self {
            _marker: std::marker::PhantomData,
        }
    }

    /// Get infos from the DLL
    ///
    /// # Errors
    /// When the DLL memory gets corrupted and returns a non UTF-8 string.
    pub fn info(&self) -> Result<String, std::string::FromUtf8Error> {
        let mut info = crate::bindings::ddsffi::DDSInfo {
            major: 0,
            minor: 0,
            versionString: [0; 10],
            system: 0,
            compiler: 0,
            constructor: 0,
            threading: 0,
            noOfThreads: 0,
            systemString: [0; 1024],
            patch: 0,
            numBits: 0,
            numCores: 0,
            threadSizes: [0; 128],
        };
        unsafe {
            crate::bindings::ddsffi::GetDDSInfo(std::ptr::from_mut::<
                crate::bindings::ddsffi::DDSInfo,
            >(&mut info));
        }
        #[allow(clippy::cast_sign_loss)]
        String::from_utf8(info.systemString.iter().map(|x| *x as u8).collect())
    }
}

impl Default for DoubleDummySolver {
    fn default() -> Self {
        Self::new()
    }
}

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
            inner: INSTANCE.get_or_init(|| Mutex::new(DoubleDummySolver::new())),
        }
    }
    fn set_max_threads(user_threads: ThreadIndex) {
        unsafe { crate::bindings::ffi::SetMaxThreads(user_threads.into()) }
    }

    fn set_resources(max_memory_mb: i32, max_threads: ThreadIndex) {
        unsafe { crate::bindings::ffi::SetResources(max_memory_mb, max_threads.into()) }
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

    fn dd_tricks_all_cards<D: AsDDSDeal, C: AsDDSContract>(
        &self,
        deal: &D,
        contract: &C,
    ) -> Result<FutureTricks, DDSError> {
        match self.inner.lock() {
            Ok(guard) => guard.dd_tricks_all_cards(deal, contract),
            Err(error) => {
                let guard = error.into_inner();
                // Try to recover by freeing the memory, hoping to get clean dll slate.
                guard.free_memory();
                guard.dd_tricks_all_cards(deal, contract)
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
    fn dd_tricks_all_cards_parallel<D: AsDDSDeal, C: AsDDSContract>(
        &self,
        number_of_deals: i32,
        deals: &[D; MAXNOOFBOARDS],
        contract: &[C; MAXNOOFBOARDS],
    ) -> Result<SolvedBoards, DDSError> {
        match self.inner.lock() {
            Ok(guard) => guard.dd_tricks_all_cards_parallel(number_of_deals, deals, contract),
            Err(error) => {
                let guard = error.into_inner();
                // Try to recover by freeing the memory, hoping to get clean dll slate.
                guard.free_memory();
                guard.dd_tricks_all_cards_parallel(number_of_deals, deals, contract)
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
        let future_tricks = self.dd_tricks_all_cards(deal, contract)?;
        #[allow(
            clippy::cast_possible_truncation,
            clippy::cast_possible_wrap,
            clippy::cast_sign_loss
        )]
        Ok(13 - future_tricks.score()[0] as u8)
    }

    #[inline]
    fn dd_tricks_all_cards<D: AsDDSDeal, C: AsDDSContract>(
        &self,
        deal: &D,
        contract: &C,
    ) -> Result<FutureTricks, DDSError> {
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
        if result == 1 {
            Ok(future_tricks)
        } else {
            Err(DDSError::from(result))
        }
    }

    #[inline]
    fn dd_tricks_parallel<D: AsDDSDeal, C: AsDDSContract>(
        &self,
        number_of_deals: i32,
        deals: &[D; MAXNOOFBOARDS],
        contract: &[C; MAXNOOFBOARDS],
    ) -> Result<Vec<u8>, DDSError> {
        let solved_boards = self.dd_tricks_all_cards_parallel(number_of_deals, deals, contract)?;
        #[allow(
            clippy::cast_possible_truncation,
            clippy::cast_possible_wrap,
            clippy::cast_sign_loss
        )]
        Ok(solved_boards
            .into_iter()
            .map(|ft| 13 - ft.score[0] as u8)
            .take(number_of_deals as usize)
            .collect())
    }
    fn dd_tricks_all_cards_parallel<D: AsDDSDeal, C: AsDDSContract>(
        &self,
        number_of_deals: i32,
        deals: &[D; MAXNOOFBOARDS],
        contract: &[C; MAXNOOFBOARDS],
    ) -> Result<SolvedBoards, DDSError> {
        #[allow(
            clippy::cast_possible_truncation,
            clippy::cast_possible_wrap,
            clippy::cast_sign_loss
        )]
        {
            assert!(number_of_deals <= MAXNOOFBOARDS as i32);
        }
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
        Ok(solved_boards)
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
        let Ok(guard) = self.inner.lock() else {
            #[allow(clippy::print_stderr, clippy::use_debug)]
            {
                use std::thread;
                eprintln!("Thread {:?} found Mutex poisoned", thread::current().id());
                return Err(RETURN_UNKNOWN_FAULT.into());
            }
        };
        guard.analyze_play(deal, contract, play)
    }

    #[inline]
    fn analyze_all_plays<D: AsDDSDeal, C: AsDDSContract>(
        &self,
        deals: &[D],
        contracts: &[C],
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
        deals: &[D],
        contracts: &[C],
        plays: &mut PlayTracesBin,
    ) -> Result<SolvedPlays, DDSError> {
        let deals_len = i32::try_from(deals.len().clamp(0, MAXNOOFBOARDS)).unwrap();
        let contracts_len = i32::try_from(contracts.len().clamp(0, MAXNOOFBOARDS)).unwrap();
        #[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
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
        let Ok(dds_deal) = build_c_deal((contract, deal)) else {
            return Err(RETURN_UNKNOWN_FAULT.into());
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

#[allow(unused_imports, clippy::wildcard_imports)]
mod test {
    use doubledummy::MultiThreadDoubleDummySolver;
    use tables::{DdTableCalculator, DdTableResults, DdTablesRes, ParCalcMode, Populated};

    use crate::*;

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
        for index in 0..3 {
            check_table(&table.results[index], index);
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
    #[allow(clippy::needless_range_loop)]
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
