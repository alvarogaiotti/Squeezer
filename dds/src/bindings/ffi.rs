// Copyright (C) 2024 Alvaro Gaiotti
// See end of file for license information

//! Legacy `dll.h` entry points still used by the high-level wrappers.
//!
//! The modern `dds_c_*` context shim and the remaining legacy entry points
//! (`AnalysePlayBin`, `Par`, `DealerPar`, `ErrorMessage`) live in
//! [`crate::context`]. This module keeps only the batch and single-board
//! symbols the pre-3.1 wrappers still call.

use crate::{
    analyseplay::{PlayTracesBin, SolvedPlays},
    deal::{Boards, DdsDeal},
    future_tricks::FutureTricks,
    solver::SolvedBoards,
    tables::*,
};

// NOTE: Not ported as irrelevant for our usecase.
unsafe extern "C" {
    pub(crate) fn FreeMemory();
}

// NOTE: Ported
unsafe extern "C" {
    pub(crate) fn SolveBoard(
        dl: DdsDeal,
        target: ::std::os::raw::c_int,
        solutions: ::std::os::raw::c_int,
        mode: ::std::os::raw::c_int,
        futp: *mut FutureTricks,
        threadIndex: ::std::os::raw::c_int,
    ) -> ::std::os::raw::c_int;
}

// NOTE: Implemented
unsafe extern "C" {
    pub(crate) fn CalcDDtable(
        tableDeal: DdTableDeal,
        tablep: *mut DdTableResults<NotPopulated>,
    ) -> ::std::os::raw::c_int;
}

// NOTE: Implemented
unsafe extern "C" {
    pub(crate) fn CalcAllTables(
        dealsp: *mut DdTableDeals,
        mode: ::std::os::raw::c_int,
        trumpFilter: *mut ::std::os::raw::c_int,
        resp: *mut DdTablesRes<NotPopulated>,
        presp: *mut AllParResults,
    ) -> ::std::os::raw::c_int;
}

// NOTE: Implemented
unsafe extern "C" {
    pub(crate) fn SolveAllBoardsBin(
        bop: *mut Boards,
        solvedp: *mut SolvedBoards,
    ) -> ::std::os::raw::c_int;
}

//  NOTE: Ported
unsafe extern "C" {
    pub(crate) fn AnalyseAllPlaysBin(
        bop: *mut Boards,
        plp: *mut PlayTracesBin,
        solvedp: *mut SolvedPlays,
        chunkSize: ::std::os::raw::c_int,
    ) -> ::std::os::raw::c_int;
}
