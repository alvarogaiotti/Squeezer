// Copyright (C) 2024 Alvaro Gaiotti
// See end of file for license information

use crate::{
    analyseplay::{
        PlayTraceBin, PlayTracePBN, PlayTracesBin, PlayTracesPBN, SolvedPlay, SolvedPlays,
    },
    deal::{Boards, BoardsPbn, DdsDeal, DdsDealPbn},
    future_tricks::FutureTricks,
    par::{ParResultsDealer, ParResultsMaster, ParTextResults},
    solver::SolvedBoards,
    tables::*,
};

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

// NOTE: Implemented
extern "C" {
    pub(crate) fn CalcDDtable(
        tableDeal: DdTableDeal,
        tablep: *mut DdTableResults<NotPopulated>,
    ) -> ::std::os::raw::c_int;
}

// NOTE: Implemented
extern "C" {
    pub(crate) fn CalcDDtablePBN(
        tableDealPBN: DdTableDealPbn,
        tablep: *mut DdTableResults<NotPopulated>,
    ) -> ::std::os::raw::c_int;
}

// NOTE: Implemented
extern "C" {
    pub(crate) fn CalcAllTables(
        dealsp: *mut DdTableDeals,
        mode: ::std::os::raw::c_int,
        trumpFilter: *mut ::std::os::raw::c_int,
        resp: *mut DdTablesRes<NotPopulated>,
        presp: *mut AllParResults,
    ) -> ::std::os::raw::c_int;
}

// NOTE: Implemented
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

//  NOTE: Implemented
extern "C" {
    pub(crate) fn Par(
        tablep: *mut DdTableResults<Populated>,
        presp: *mut ParResults,
        vulnerable: ::std::os::raw::c_int,
    ) -> ::std::os::raw::c_int;
}

//  NOTE: Implemented
extern "C" {
    pub(crate) fn SidesPar(
        tablep: *mut DdTableResults<Populated>,
        sidesRes: *mut ParResultsDealer,
        vulnerable: ::std::os::raw::c_int,
    ) -> ::std::os::raw::c_int;
}

//  NOTE: Implemented
extern "C" {
    pub(crate) fn DealerPar(
        tablep: *mut DdTableResults<Populated>,
        presp: *mut ParResultsDealer,
        dealer: ::std::os::raw::c_int,
        vulnerable: ::std::os::raw::c_int,
    ) -> ::std::os::raw::c_int;
}

//  NOTE: Implemented
extern "C" {
    pub(crate) fn DealerParBin(
        tablep: *mut DdTableResults<Populated>,
        presp: *mut ParResultsMaster,
        dealer: ::std::os::raw::c_int,
        vulnerable: ::std::os::raw::c_int,
    ) -> ::std::os::raw::c_int;
}

//  NOTE: Implemented
extern "C" {
    pub(crate) fn SidesParBin(
        tablep: *mut DdTableResults<Populated>,
        sidesRes: *mut ParResultsMaster,
        vulnerable: ::std::os::raw::c_int,
    ) -> ::std::os::raw::c_int;
}

//  NOTE: Implemented
extern "C" {
    pub(crate) fn ConvertToDealerTextFormat(
        pres: *mut ParResultsMaster,
        resp: *mut ::std::os::raw::c_char,
    ) -> ::std::os::raw::c_int;
}

//  NOTE: Implemented
extern "C" {
    pub(crate) fn ConvertToSidesTextFormat(
        pres: *mut ParResultsMaster,
        resp: *mut ParTextResults,
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
