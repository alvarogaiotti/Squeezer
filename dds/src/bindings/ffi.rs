// Copyright (C) 2024 Alvaro Gaiotti
// See end of file for license information

use super::ddsffi::{parResultsDealer, parResultsMaster, parTextResults};
use super::tables::*;
use crate::{
    Boards, BoardsPbn, DdsDealPbn, PlayTraceBin, PlayTracePBN, PlayTracesBin, PlayTracesPBN,
    SolvedBoards, SolvedPlay, SolvedPlays,
};

use super::future_tricks::FutureTricks;
use super::DdsDeal;

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
/// # use dds::{DoubleDummySolver, BridgeSolver};
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
/// # impl dds::AsDDSDeal for DealMock {
/// #     fn as_dds_deal(&self) -> dds::DDSDealRepr {
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
/// # impl dds::ContractScorer for ContractMock {
/// #     fn score(&self, _tricks: u8) -> i32 {
/// #         0
/// #     }
/// # }
/// #
/// # impl dds::AsDDSContract for ContractMock {
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
pub struct DoubleDummySolver;

impl DoubleDummySolver {
    /// Function to free the dinamically allocated memory of the DDS's DLL.
    /// Use this if you are not gonna use DDS for a long time and want to free some memory.
    pub fn free_memory(&self) {
        unsafe { FreeMemory() }
    }

    pub fn new() -> Self {
        Self {}
    }
}

// NOTE: Ported
extern "C" {
    pub(crate) fn SetMaxThreads(userThreads: ::std::os::raw::c_int);
}
// NOTE: Ported
extern "C" {
    pub(crate) fn SetThreading(code: ::std::os::raw::c_int) -> ::std::os::raw::c_int;
}

// NOTE: Ported
extern "C" {
    pub(crate) fn SetResources(
        maxMemoryMB: ::std::os::raw::c_int,
        maxThreads: ::std::os::raw::c_int,
    );
}
// NOTE: Not ported as irrelevant for our usecase.
extern "C" {
    pub(crate) fn FreeMemory();
}

// NOTE: Ported
extern "C" {
    pub(crate) fn SolveBoard(
        dl: DdsDeal,
        target: ::std::os::raw::c_int,
        solutions: ::std::os::raw::c_int,
        mode: ::std::os::raw::c_int,
        futp: *mut FutureTricks,
        threadIndex: ::std::os::raw::c_int,
    ) -> ::std::os::raw::c_int;
}

// NOTE: Ported
extern "C" {
    pub(crate) fn SolveBoardPBN(
        dlpbn: DdsDealPbn,
        target: ::std::os::raw::c_int,
        solutions: ::std::os::raw::c_int,
        mode: ::std::os::raw::c_int,
        futp: *mut FutureTricks,
        thrId: ::std::os::raw::c_int,
    ) -> ::std::os::raw::c_int;
}

// NOTE: Not implemented
extern "C" {
    pub(crate) fn CalcDDtable(
        tableDeal: DdTableDeal,
        tablep: *mut DdTableResults<NotPopulated>,
    ) -> ::std::os::raw::c_int;
}

// NOTE: Not implemented
extern "C" {
    pub(crate) fn CalcDDtablePBN(
        tableDealPBN: DdTableDealPbn,
        tablep: *mut DdTableResults<NotPopulated>,
    ) -> ::std::os::raw::c_int;
}

// NOTE: Not implemented
extern "C" {
    pub(crate) fn CalcAllTables(
        dealsp: *mut DdTableDeals,
        mode: ::std::os::raw::c_int,
        trumpFilter: *mut ::std::os::raw::c_int,
        resp: *mut DdTablesRes<NotPopulated>,
        presp: *mut AllParResults,
    ) -> ::std::os::raw::c_int;
}

// NOTE: Not implemented
extern "C" {
    pub(crate) fn CalcAllTablesPBN(
        dealsp: *mut DdTableDealsPbn,
        mode: ::std::os::raw::c_int,
        trumpFilter: *mut ::std::os::raw::c_int,
        resp: *mut DdTablesRes<NotPopulated>,
        presp: *mut AllParResults,
    ) -> ::std::os::raw::c_int;
}

// NOTE: Implemented
extern "C" {
    pub(crate) fn SolveAllBoards(
        bop: *mut BoardsPbn,
        solvedp: *mut SolvedBoards,
    ) -> ::std::os::raw::c_int;
}

// NOTE: Implemented
extern "C" {
    pub(crate) fn SolveAllBoardsBin(
        bop: *mut Boards,
        solvedp: *mut SolvedBoards,
    ) -> ::std::os::raw::c_int;
}

// NOTE: Deprecated
extern "C" {
    pub(crate) fn SolveAllChunks(
        bop: *mut BoardsPbn,
        solvedp: *mut SolvedBoards,
        chunkSize: ::std::os::raw::c_int,
    ) -> ::std::os::raw::c_int;
}

// NOTE: Deprecated
extern "C" {
    pub(crate) fn SolveAllChunksBin(
        bop: *mut Boards,
        solvedp: *mut SolvedBoards,
        chunkSize: ::std::os::raw::c_int,
    ) -> ::std::os::raw::c_int;
}

// NOTE: Alias of SolveAllChunks
extern "C" {
    pub(crate) fn SolveAllChunksPBN(
        bop: *mut BoardsPbn,
        solvedp: *mut SolvedBoards,
        chunkSize: ::std::os::raw::c_int,
    ) -> ::std::os::raw::c_int;
}

//  NOTE: Not implemented
extern "C" {
    pub(crate) fn Par(
        tablep: *mut DdTableResults<Populated>,
        presp: *mut ParResults,
        vulnerable: ::std::os::raw::c_int,
    ) -> ::std::os::raw::c_int;
}

//  NOTE: Not implemented
extern "C" {
    pub(crate) fn CalcPar(
        tableDeal: DdTableDeal,
        vulnerable: ::std::os::raw::c_int,
        tablep: *mut DdTableResults<Populated>,
        presp: *mut ParResults,
    ) -> ::std::os::raw::c_int;
}

//  NOTE: Not implemented
extern "C" {
    pub(crate) fn CalcParPBN(
        tableDealPBN: DdTableDealPbn,
        tablep: *mut DdTableResults<Populated>,
        vulnerable: ::std::os::raw::c_int,
        presp: *mut ParResults,
    ) -> ::std::os::raw::c_int;
}

//  NOTE: Not Implemented
extern "C" {
    pub(crate) fn SidesPar(
        tablep: *mut DdTableResults<Populated>,
        sidesRes: *mut parResultsDealer,
        vulnerable: ::std::os::raw::c_int,
    ) -> ::std::os::raw::c_int;
}

//  NOTE: Not implemented
extern "C" {
    pub(crate) fn DealerPar(
        tablep: *mut DdTableResults<Populated>,
        presp: *mut parResultsDealer,
        dealer: ::std::os::raw::c_int,
        vulnerable: ::std::os::raw::c_int,
    ) -> ::std::os::raw::c_int;
}

//  NOTE: Not implemented
extern "C" {
    pub(crate) fn DealerParBin(
        tablep: *mut DdTableResults<Populated>,
        presp: *mut parResultsMaster,
        dealer: ::std::os::raw::c_int,
        vulnerable: ::std::os::raw::c_int,
    ) -> ::std::os::raw::c_int;
}

//  NOTE: Not implemented
extern "C" {
    pub(crate) fn SidesParBin(
        tablep: *mut DdTableResults<Populated>,
        sidesRes: *mut parResultsMaster,
        vulnerable: ::std::os::raw::c_int,
    ) -> ::std::os::raw::c_int;
}

//  NOTE: Not yet ported
extern "C" {
    pub(crate) fn ConvertToDealerTextFormat(
        pres: *mut parResultsMaster,
        resp: *mut ::std::os::raw::c_char,
    ) -> ::std::os::raw::c_int;
}

//  NOTE: Not yet ported
extern "C" {
    pub(crate) fn ConvertToSidesTextFormat(
        pres: *mut parResultsMaster,
        resp: *mut parTextResults,
    ) -> ::std::os::raw::c_int;
}

//  NOTE: Ported
extern "C" {
    pub(crate) fn AnalysePlayBin(
        dl: DdsDeal,
        play: PlayTraceBin,
        solved: *mut SolvedPlay,
        thrId: ::std::os::raw::c_int,
    ) -> ::std::os::raw::c_int;
}

//  NOTE: Ported
extern "C" {
    pub(crate) fn AnalysePlayPBN(
        dlPBN: DdsDealPbn,
        playPBN: PlayTracePBN,
        solvedp: *mut SolvedPlay,
        thrId: ::std::os::raw::c_int,
    ) -> ::std::os::raw::c_int;
}

//  NOTE: Ported
extern "C" {
    pub(crate) fn AnalyseAllPlaysBin(
        bop: *mut Boards,
        plp: *mut PlayTracesBin,
        solvedp: *mut SolvedPlays,
        chunkSize: ::std::os::raw::c_int,
    ) -> ::std::os::raw::c_int;
}

//  NOTE: Ported
extern "C" {
    pub(crate) fn AnalyseAllPlaysPBN(
        bopPBN: *mut BoardsPbn,
        plpPBN: *mut PlayTracesPBN,
        solvedp: *mut SolvedPlays,
        chunkSize: ::std::os::raw::c_int,
    ) -> ::std::os::raw::c_int;
}
