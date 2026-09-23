// Copyright (C) 2024 Alvaro Gaiotti
// See end of file for license information

//! Legacy `dll.h` entry points still used by the high-level wrappers.
//!
//! The modern `dds_c_*` context shim and the remaining legacy entry points
//! (`AnalysePlayBin`, `Par`, `DealerPar`, `ErrorMessage`) live in
//! [`crate::context`]. This module keeps only the single-board and table
//! symbols the wrappers still call; the batch solve/analyse entry points were
//! removed when concurrency moved to the caller's context pool.

use crate::{deal::DdsDeal, future_tricks::FutureTricks, tables::*};

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
