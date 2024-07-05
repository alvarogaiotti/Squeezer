use super::ddsffi::{parResultsDealer, parResultsMaster, parTextResults};
use super::tables::*;
use crate::{
    Boards, BoardsPbn, DdsDealPbn, PlayTraceBin, PlayTracePBN, PlayTracesBin, PlayTracesPBN,
    SolvedBoards, SolvedPlay, SolvedPlays,
};

use super::future_tricks::FutureTricks;
use super::DdsDeal;

pub struct InnerDds;

extern "C" {
    pub fn SetMaxThreads(userThreads: ::std::os::raw::c_int);
}
extern "C" {
    pub fn SetThreading(code: ::std::os::raw::c_int) -> ::std::os::raw::c_int;
}
extern "C" {
    pub fn SetResources(maxMemoryMB: ::std::os::raw::c_int, maxThreads: ::std::os::raw::c_int);
}
extern "C" {
    pub fn FreeMemory();
}

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
extern "C" {
    pub fn CalcDDtable(
        tableDeal: DdTableDeal,
        tablep: *mut DdTableResults,
    ) -> ::std::os::raw::c_int;
}
extern "C" {
    pub fn CalcDDtablePBN(
        tableDealPBN: DdTableDealPbn,
        tablep: *mut DdTableResults,
    ) -> ::std::os::raw::c_int;
}
extern "C" {
    pub fn CalcAllTables(
        dealsp: *mut DdTableDeals,
        mode: ::std::os::raw::c_int,
        trumpFilter: *mut ::std::os::raw::c_int,
        resp: *mut DdTablesRes,
        presp: *mut AllParResults,
    ) -> ::std::os::raw::c_int;
}
extern "C" {
    pub fn CalcAllTablesPBN(
        dealsp: *mut DdTableDealsPbn,
        mode: ::std::os::raw::c_int,
        trumpFilter: *mut ::std::os::raw::c_int,
        resp: *mut DdTablesRes,
        presp: *mut AllParResults,
    ) -> ::std::os::raw::c_int;
}
extern "C" {
    pub fn SolveAllBoards(bop: *mut BoardsPbn, solvedp: *mut SolvedBoards)
        -> ::std::os::raw::c_int;
}
extern "C" {
    pub fn SolveAllBoardsBin(bop: *mut Boards, solvedp: *mut SolvedBoards)
        -> ::std::os::raw::c_int;
}
extern "C" {
    pub fn SolveAllChunks(
        bop: *mut BoardsPbn,
        solvedp: *mut SolvedBoards,
        chunkSize: ::std::os::raw::c_int,
    ) -> ::std::os::raw::c_int;
}
extern "C" {
    pub fn SolveAllChunksBin(
        bop: *mut Boards,
        solvedp: *mut SolvedBoards,
        chunkSize: ::std::os::raw::c_int,
    ) -> ::std::os::raw::c_int;
}
extern "C" {
    pub fn SolveAllChunksPBN(
        bop: *mut BoardsPbn,
        solvedp: *mut SolvedBoards,
        chunkSize: ::std::os::raw::c_int,
    ) -> ::std::os::raw::c_int;
}
extern "C" {
    pub fn Par(
        tablep: *mut DdTableResults,
        presp: *mut ParResults,
        vulnerable: ::std::os::raw::c_int,
    ) -> ::std::os::raw::c_int;
}
extern "C" {
    pub fn CalcPar(
        tableDeal: DdTableDeal,
        vulnerable: ::std::os::raw::c_int,
        tablep: *mut DdTableResults,
        presp: *mut ParResults,
    ) -> ::std::os::raw::c_int;
}
extern "C" {
    pub fn CalcParPBN(
        tableDealPBN: DdTableDealPbn,
        tablep: *mut DdTableResults,
        vulnerable: ::std::os::raw::c_int,
        presp: *mut ParResults,
    ) -> ::std::os::raw::c_int;
}
extern "C" {
    pub fn SidesPar(
        tablep: *mut DdTableResults,
        sidesRes: *mut parResultsDealer,
        vulnerable: ::std::os::raw::c_int,
    ) -> ::std::os::raw::c_int;
}
extern "C" {
    pub fn DealerPar(
        tablep: *mut DdTableResults,
        presp: *mut parResultsDealer,
        dealer: ::std::os::raw::c_int,
        vulnerable: ::std::os::raw::c_int,
    ) -> ::std::os::raw::c_int;
}
extern "C" {
    pub fn DealerParBin(
        tablep: *mut DdTableResults,
        presp: *mut parResultsMaster,
        dealer: ::std::os::raw::c_int,
        vulnerable: ::std::os::raw::c_int,
    ) -> ::std::os::raw::c_int;
}
extern "C" {
    pub fn SidesParBin(
        tablep: *mut DdTableResults,
        sidesRes: *mut parResultsMaster,
        vulnerable: ::std::os::raw::c_int,
    ) -> ::std::os::raw::c_int;
}
extern "C" {
    pub fn ConvertToDealerTextFormat(
        pres: *mut parResultsMaster,
        resp: *mut ::std::os::raw::c_char,
    ) -> ::std::os::raw::c_int;
}
extern "C" {
    pub fn ConvertToSidesTextFormat(
        pres: *mut parResultsMaster,
        resp: *mut parTextResults,
    ) -> ::std::os::raw::c_int;
}
extern "C" {
    pub fn AnalysePlayBin(
        dl: DdsDeal,
        play: PlayTraceBin,
        solved: *mut SolvedPlay,
        thrId: ::std::os::raw::c_int,
    ) -> ::std::os::raw::c_int;
}
extern "C" {
    pub fn AnalysePlayPBN(
        dlPBN: DdsDealPbn,
        playPBN: PlayTracePBN,
        solvedp: *mut SolvedPlay,
        thrId: ::std::os::raw::c_int,
    ) -> ::std::os::raw::c_int;
}
extern "C" {
    pub fn AnalyseAllPlaysBin(
        bop: *mut Boards,
        plp: *mut PlayTracesBin,
        solvedp: *mut SolvedPlays,
        chunkSize: ::std::os::raw::c_int,
    ) -> ::std::os::raw::c_int;
}
extern "C" {
    pub fn AnalyseAllPlaysPBN(
        bopPBN: *mut BoardsPbn,
        plpPBN: *mut PlayTracesPBN,
        solvedp: *mut SolvedPlays,
        chunkSize: ::std::os::raw::c_int,
    ) -> ::std::os::raw::c_int;
}
