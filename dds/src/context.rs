// Copyright (C) 2024 Alvaro Gaiotti
// See end of file for license information

//! Hand-declared FFI for the DDS 3.1 ABI.
//!
//! DDS 3.1 exposes two entry-point families that this crate binds:
//!
//! * the modern `dds_c_*` context shim over `SolverContext`. Concurrency is
//!   caller-owned: one context per worker, reused across the boards that
//!   worker solves. `dds_c_create_solvercontext_default` allocates the
//!   built-in transposition-table sizes; `dds_c_create_solvercontext` lets the
//!   caller pick them. Every function is C-callable and takes only pointers
//!   and plain-old-data by value, so no C++ types cross the boundary.
//! * the legacy `dll.h` entry points that the shim does not replace:
//!   [`AnalysePlayBin`] (play analysis), [`Par`] (par from a precomputed
//!   table), [`DealerPar`] (declarer-aware par) and [`ErrorMessage`].
//!
//! This module is deliberately a dumb FFI surface: it declares the symbols and
//! re-exports the `#[repr(C)]` structs they traffic in, but builds no solver
//! abstraction on top. The compile-time assertions below pin the v3.1.0
//! layouts so an upstream struct change cannot silently corrupt a call.
//!
//! The `*mut c_void` context handle is opaque; the C header calls it
//! `DDS_C_SOLVER_CTX`. Use one handle per thread, and release it with
//! [`dds_c_destroy_solvercontext`].

use core::ffi::{c_char, c_int, c_void};

pub use crate::analyseplay::{PlayTraceBin, SolvedPlay};
pub use crate::deal::DdsDeal as Deal;
pub use crate::future_tricks::FutureTricks;
pub use crate::par::ParResultsDealer;
use crate::tables::NotPopulated;
use crate::tables::Populated;
pub use crate::tables::{DdTableDeal, DdTableResults, ParResults};

unsafe extern "C" {
    /// Allocates a solver context with DDS's built-in transposition-table
    /// sizes. Returns NULL on failure.
    pub fn dds_c_create_solvercontext_default() -> *mut c_void;

    /// Allocates a solver context with an explicit transposition-table
    /// configuration. `tt_kind` is `0 = Small`, `1 = Large`; `def_mb` and
    /// `max_mb` of `0` normalize to DDS's built-in sizes. Returns NULL on
    /// failure.
    pub fn dds_c_create_solvercontext(tt_kind: c_int, def_mb: c_int, max_mb: c_int) -> *mut c_void;

    /// Releases a context obtained from one of the `dds_c_create_*` functions.
    pub fn dds_c_destroy_solvercontext(ctx: *mut c_void);

    /// Solves a single board using the caller's context. Returns a DDS
    /// `RETURN_*` status code (`1` on success).
    pub fn dds_c_solve_board(
        ctx: *mut c_void,
        dl: *const Deal,
        target: c_int,
        solutions: c_int,
        mode: c_int,
        futp: *mut FutureTricks,
    ) -> c_int;

    /// Computes the double-dummy table for a deal. Returns a DDS `RETURN_*`
    /// status code.
    pub fn dds_c_calc_dd_table(
        ctx: *mut c_void,
        deal: *const DdTableDeal,
        results: *mut DdTableResults<NotPopulated>,
    ) -> c_int;

    /// Computes the par result for a deal, filling the DD table as a
    /// by-product. Returns a DDS `RETURN_*` status code.
    pub fn dds_c_calc_par(
        ctx: *mut c_void,
        deal: *const DdTableDeal,
        vulnerable: c_int,
        results: *mut DdTableResults<NotPopulated>,
        par: *mut ParResults,
    ) -> c_int;

    /// Reconfigures the context's transposition table in place.
    pub fn dds_c_configure_tt(ctx: *mut c_void, tt_kind: c_int, def_mb: c_int, max_mb: c_int);

    /// Clears the context's transposition table.
    pub fn dds_c_clear_tt(ctx: *mut c_void);

    /// Resets the per-solve state, keeping the transposition table.
    pub fn dds_c_reset_for_solve(ctx: *mut c_void);

    /// Legacy play analysis. Builds its own local context per call and
    /// ignores `thr`, so it is safe to call concurrently from our workers.
    pub fn AnalysePlayBin(
        dl: Deal,
        play: PlayTraceBin,
        solved: *mut SolvedPlay,
        thr: c_int,
    ) -> c_int;

    /// Legacy par from a precomputed DD table.
    pub fn Par(
        tablep: *const DdTableResults<Populated>,
        presp: *mut ParResults,
        vulnerable: c_int,
    ) -> c_int;

    /// Legacy declarer-aware par from a precomputed DD table.
    pub fn DealerPar(
        tablep: *const DdTableResults<Populated>,
        presp: *mut ParResultsDealer,
        dealer: c_int,
        vulnerable: c_int,
    ) -> c_int;

    /// Fills `line` (80 bytes) with the text form of a DDS `RETURN_*` code.
    pub fn ErrorMessage(code: c_int, line: *mut c_char);
}

// The sizes below are the v3.1.0 layouts measured in ticket 01 against the
// vendored headers, not the 2.9 sizes. They are asserted at compile time so a
// future vendor bump that changes a struct fails the build here rather than
// corrupting a call at runtime.
const _: () = {
    assert!(core::mem::size_of::<Deal>() == 96);
    assert!(core::mem::size_of::<FutureTricks>() == 216);
    assert!(core::mem::size_of::<PlayTraceBin>() == 420);
    assert!(core::mem::size_of::<SolvedPlay>() == 216);
    assert!(core::mem::size_of::<DdTableDeal>() == 64);
    assert!(core::mem::size_of::<DdTableResults<NotPopulated>>() == 80);
    assert!(core::mem::size_of::<DdTableResults<Populated>>() == 80);
    assert!(core::mem::size_of::<ParResults>() == 288);

    // Field offsets guard against a same-size reordering.
    assert!(core::mem::offset_of!(FutureTricks, cards) == 4);
    assert!(core::mem::offset_of!(FutureTricks, suit) == 8);
    assert!(core::mem::offset_of!(FutureTricks, rank) == 60);
    assert!(core::mem::offset_of!(FutureTricks, equals) == 112);
    assert!(core::mem::offset_of!(FutureTricks, score) == 164);
    assert!(core::mem::offset_of!(PlayTraceBin, suit) == 4);
    assert!(core::mem::offset_of!(PlayTraceBin, rank) == 212);
    assert!(core::mem::offset_of!(SolvedPlay, tricks) == 4);
    assert!(core::mem::offset_of!(DdTableResults<NotPopulated>, res_table) == 0);
    assert!(core::mem::offset_of!(ParResults, par_contracts_string) == 32);
};
