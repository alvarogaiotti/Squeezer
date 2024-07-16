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
/// This struct implements all the needed traits linked to a double dummy analysis
/// such as:
/// - Play analysis
/// - Par calculation
/// - Board solution
pub struct DoubleDummySolver;

// NOTE: Ported
extern "C" {
    pub fn SetMaxThreads(userThreads: ::std::os::raw::c_int);
}
// NOTE: Ported
extern "C" {
    pub fn SetThreading(code: ::std::os::raw::c_int) -> ::std::os::raw::c_int;
}

// NOTE: Ported
extern "C" {
    pub fn SetResources(maxMemoryMB: ::std::os::raw::c_int, maxThreads: ::std::os::raw::c_int);
}
// NOTE: Not ported as irrelevant for our usecase.
extern "C" {
    pub fn FreeMemory();
}

// NOTE: Ported
extern "C" {
    pub fn SolveBoard(
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
    pub fn SolveBoardPBN(
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
    pub fn CalcDDtable(
        tableDeal: DdTableDeal,
        tablep: *mut DdTableResults<NotPopulated>,
    ) -> ::std::os::raw::c_int;
}

// NOTE: Not implemented
extern "C" {
    pub fn CalcDDtablePBN(
        tableDealPBN: DdTableDealPbn,
        tablep: *mut DdTableResults<NotPopulated>,
    ) -> ::std::os::raw::c_int;
}

// NOTE: Not implemented
extern "C" {
    pub fn CalcAllTables(
        dealsp: *mut DdTableDeals,
        mode: ::std::os::raw::c_int,
        trumpFilter: *mut ::std::os::raw::c_int,
        resp: *mut DdTablesRes<NotPopulated>,
        presp: *mut AllParResults,
    ) -> ::std::os::raw::c_int;
}

// NOTE: Not implemented
extern "C" {
    pub fn CalcAllTablesPBN(
        dealsp: *mut DdTableDealsPbn,
        mode: ::std::os::raw::c_int,
        trumpFilter: *mut ::std::os::raw::c_int,
        resp: *mut DdTablesRes<NotPopulated>,
        presp: *mut AllParResults,
    ) -> ::std::os::raw::c_int;
}

// NOTE: Implemented
extern "C" {
    pub fn SolveAllBoards(bop: *mut BoardsPbn, solvedp: *mut SolvedBoards)
        -> ::std::os::raw::c_int;
}

// NOTE: Implemented
extern "C" {
    pub fn SolveAllBoardsBin(bop: *mut Boards, solvedp: *mut SolvedBoards)
        -> ::std::os::raw::c_int;
}

// NOTE: Deprecated
extern "C" {
    pub fn SolveAllChunks(
        bop: *mut BoardsPbn,
        solvedp: *mut SolvedBoards,
        chunkSize: ::std::os::raw::c_int,
    ) -> ::std::os::raw::c_int;
}

// NOTE: Deprecated
extern "C" {
    pub fn SolveAllChunksBin(
        bop: *mut Boards,
        solvedp: *mut SolvedBoards,
        chunkSize: ::std::os::raw::c_int,
    ) -> ::std::os::raw::c_int;
}

// NOTE: Alias of SolveAllChunks
extern "C" {
    pub fn SolveAllChunksPBN(
        bop: *mut BoardsPbn,
        solvedp: *mut SolvedBoards,
        chunkSize: ::std::os::raw::c_int,
    ) -> ::std::os::raw::c_int;
}

//  NOTE: Not implemented
extern "C" {
    pub fn Par(
        tablep: *mut DdTableResults<Populated>,
        presp: *mut ParResults,
        vulnerable: ::std::os::raw::c_int,
    ) -> ::std::os::raw::c_int;
}

//  NOTE: Not implemented
extern "C" {
    pub fn CalcPar(
        tableDeal: DdTableDeal,
        vulnerable: ::std::os::raw::c_int,
        tablep: *mut DdTableResults<Populated>,
        presp: *mut ParResults,
    ) -> ::std::os::raw::c_int;
}

//  NOTE: Not implemented
extern "C" {
    pub fn CalcParPBN(
        tableDealPBN: DdTableDealPbn,
        tablep: *mut DdTableResults<Populated>,
        vulnerable: ::std::os::raw::c_int,
        presp: *mut ParResults,
    ) -> ::std::os::raw::c_int;
}

//  NOTE: Not Implemented
extern "C" {
    pub fn SidesPar(
        tablep: *mut DdTableResults<Populated>,
        sidesRes: *mut parResultsDealer,
        vulnerable: ::std::os::raw::c_int,
    ) -> ::std::os::raw::c_int;
}

//  NOTE: Not implemented
extern "C" {
    pub fn DealerPar(
        tablep: *mut DdTableResults<Populated>,
        presp: *mut parResultsDealer,
        dealer: ::std::os::raw::c_int,
        vulnerable: ::std::os::raw::c_int,
    ) -> ::std::os::raw::c_int;
}

//  NOTE: Not implemented
extern "C" {
    pub fn DealerParBin(
        tablep: *mut DdTableResults<Populated>,
        presp: *mut parResultsMaster,
        dealer: ::std::os::raw::c_int,
        vulnerable: ::std::os::raw::c_int,
    ) -> ::std::os::raw::c_int;
}

//  NOTE: Not implemented
extern "C" {
    pub fn SidesParBin(
        tablep: *mut DdTableResults<Populated>,
        sidesRes: *mut parResultsMaster,
        vulnerable: ::std::os::raw::c_int,
    ) -> ::std::os::raw::c_int;
}

//  NOTE: Not yet ported
extern "C" {
    pub fn ConvertToDealerTextFormat(
        pres: *mut parResultsMaster,
        resp: *mut ::std::os::raw::c_char,
    ) -> ::std::os::raw::c_int;
}

//  NOTE: Not yet ported
extern "C" {
    pub fn ConvertToSidesTextFormat(
        pres: *mut parResultsMaster,
        resp: *mut parTextResults,
    ) -> ::std::os::raw::c_int;
}

//  NOTE: Ported
extern "C" {
    pub fn AnalysePlayBin(
        dl: DdsDeal,
        play: PlayTraceBin,
        solved: *mut SolvedPlay,
        thrId: ::std::os::raw::c_int,
    ) -> ::std::os::raw::c_int;
}

//  NOTE: Ported
extern "C" {
    pub fn AnalysePlayPBN(
        dlPBN: DdsDealPbn,
        playPBN: PlayTracePBN,
        solvedp: *mut SolvedPlay,
        thrId: ::std::os::raw::c_int,
    ) -> ::std::os::raw::c_int;
}

//  NOTE: Ported
extern "C" {
    pub fn AnalyseAllPlaysBin(
        bop: *mut Boards,
        plp: *mut PlayTracesBin,
        solvedp: *mut SolvedPlays,
        chunkSize: ::std::os::raw::c_int,
    ) -> ::std::os::raw::c_int;
}

//  NOTE: Ported
extern "C" {
    pub fn AnalyseAllPlaysPBN(
        bopPBN: *mut BoardsPbn,
        plpPBN: *mut PlayTracesPBN,
        solvedp: *mut SolvedPlays,
        chunkSize: ::std::os::raw::c_int,
    ) -> ::std::os::raw::c_int;
}
