// Copyright (C) 2024 Alvaro Gaiotti
// See end of file for license information

//! DDS 3.1 example-hand tests.
//!
//! Every expected value here is ported from DDS's own `examples/hands.cpp`
//! (hand 0) via the linking spike in `tmp/dds3-spike/crate/src/main.rs`, so
//! the tests use the solver as an oracle rather than recomputing results.

use core::ffi::{c_int, c_void};
use std::ffi::CStr;

use dds::context::{
    self, AnalysePlayBin, DdTableDeal, DdTableResults, Deal, ErrorMessage, FutureTricks,
    ParResults, PlayTraceBin, SolvedPlay,
};
use dds::deal::{DDSDealRepr, DdsDealBuilder, DdsHandEncoding, DdsSuit};
use dds::tables::NotPopulated;

// DDS suit encoding used by its examples: 0=Spades .. 3=Clubs.
const SPADES: usize = 0;
const HEARTS: usize = 1;
const DIAMONDS: usize = 2;
const CLUBS: usize = 3;

/// Holding bitmask: bit N set means rank N is present.
fn holding(ranks: &[u32]) -> u32 {
    ranks.iter().fold(0, |acc, &rank| acc | (1 << rank))
}

/// DDS example hand 0: `N:QJ6.K652.J85.T98 873.J97.AT764.Q4 K5.T83.KQ9.A7652
/// AT942.AQ4.32.KJ3`, indexed `[hand][suit]`.
fn example_deal() -> Deal {
    let spades: [&[u32]; 4] = [&[12, 11, 6], &[8, 7, 3], &[13, 5], &[14, 10, 9, 4, 2]];
    let hearts: [&[u32]; 4] = [&[13, 6, 5, 2], &[11, 9, 7], &[10, 8, 3], &[14, 12, 4]];
    let diamonds: [&[u32]; 4] = [&[11, 8, 5], &[14, 10, 7, 6, 4], &[13, 12, 9], &[3, 2]];
    let clubs: [&[u32]; 4] = [&[10, 9, 8], &[12, 4], &[14, 7, 6, 5, 2], &[13, 11, 3]];

    let mut remain_cards = [[0u32; 4]; 4];
    for hand in 0..4 {
        remain_cards[hand][SPADES] = holding(spades[hand]);
        remain_cards[hand][HEARTS] = holding(hearts[hand]);
        remain_cards[hand][DIAMONDS] = holding(diamonds[hand]);
        remain_cards[hand][CLUBS] = holding(clubs[hand]);
    }

    DdsDealBuilder::new()
        .trump(DdsSuit::Spades)
        .first(DdsHandEncoding::North)
        .remain_cards(DDSDealRepr::from(remain_cards))
        .build()
        .expect("example hand 0 is a valid deal")
}

/// Play trace for example hand 0 (45 cards), from `hands.cpp`.
fn example_play() -> PlayTraceBin {
    const C: c_int = CLUBS as c_int;
    const D: c_int = DIAMONDS as c_int;
    const H: c_int = HEARTS as c_int;
    const S: c_int = SPADES as c_int;
    let suit: [c_int; 45] = [
        C, C, C, C, H, H, H, H, D, D, D, D, S, S, S, S, D, D, D, D, H, H, H, H, C, C, C, C, S, S,
        S, S, H, H, H, H, C, C, D, C, S, S, S, C, D,
    ];
    let rank: [c_int; 45] = [
        10, 4, 14, 11, 8, 4, 13, 9, 5, 14, 9, 2, 7, 5, 2, 12, 8, 4, 12, 3, 3, 14, 6, 7, 3, 8, 12,
        2, 3, 13, 14, 6, 12, 5, 11, 10, 13, 9, 6, 5, 4, 11, 8, 6, 11,
    ];

    let mut play = PlayTraceBin::new();
    play.number = 45;
    play.suit[..45].copy_from_slice(&suit);
    play.rank[..45].copy_from_slice(&rank);
    play
}

/// Creates a default context, runs `body`, and always destroys the context.
fn with_context<T>(body: impl FnOnce(*mut c_void) -> T) -> T {
    let ctx = unsafe { context::dds_c_create_solvercontext_default() };
    assert!(!ctx.is_null(), "context creation returned NULL");
    let result = body(ctx);
    unsafe { context::dds_c_destroy_solvercontext(ctx) };
    result
}

#[test]
fn solve_board_returns_every_legal_card_for_example_hand() {
    let deal = example_deal();
    let mut fut = FutureTricks::default();

    let rc = with_context(|ctx| {
        // target = -1 (max tricks), solutions = 3 (all legal cards), mode = 0.
        unsafe { context::dds_c_solve_board(ctx, &deal, -1, 3, 0, &mut fut) }
    });

    assert_eq!(rc, 1);
    assert_eq!(fut.cards, 9);
    assert_eq!(fut.suit[..9], [2, 2, 2, 3, 0, 0, 1, 1, 1]);
    assert_eq!(fut.rank[..9], [5, 8, 11, 10, 6, 12, 2, 6, 13]);
    assert_eq!(fut.score[..9], [5, 5, 5, 5, 5, 5, 4, 4, 4]);
    assert_eq!(fut.equals[..9], [0, 0, 0, 768, 0, 2048, 0, 32, 0]);
}

#[test]
fn calc_dd_table_matches_example_hand() {
    let deal = example_deal();
    let table_deal = DdTableDeal {
        cards: deal.remain_cards(),
    };
    let mut table = DdTableResults::<NotPopulated>::default();

    let rc =
        with_context(|ctx| unsafe { context::dds_c_calc_dd_table(ctx, &table_deal, &mut table) });

    assert_eq!(rc, 1);
    assert_eq!(
        table.res_table,
        [
            [5, 8, 5, 8],
            [6, 6, 6, 6],
            [5, 7, 5, 7],
            [7, 5, 7, 5],
            [6, 6, 6, 6],
        ]
    );
}

#[test]
fn calc_par_matches_example_hand() {
    let deal = example_deal();
    let table_deal = DdTableDeal {
        cards: deal.remain_cards(),
    };
    let mut table = DdTableResults::<NotPopulated>::default();
    let mut par = ParResults::default();

    // vulnerable = 0 (none).
    let rc = with_context(|ctx| unsafe {
        context::dds_c_calc_par(ctx, &table_deal, 0, &mut table, &mut par)
    });

    assert_eq!(rc, 1);
    let score = unsafe { CStr::from_ptr(par.par_score[0].as_ptr()) };
    let contracts = unsafe { CStr::from_ptr(par.par_contracts_string[0].0.as_ptr()) };
    assert_eq!(score.to_str().expect("par score is ASCII"), "NS -110");
    assert_eq!(
        contracts.to_str().expect("par contracts are ASCII"),
        "NS:EW 2S"
    );
}

#[test]
fn legacy_analyse_play_bin_matches_example_hand() {
    let deal = example_deal();
    let mut solved = SolvedPlay::new();

    let rc = unsafe { AnalysePlayBin(deal, example_play(), &mut solved, 0) };

    assert_eq!(rc, 1);
    assert_eq!(solved.number, 46);
    assert!(solved.tricks[..46].iter().all(|&tricks| tricks == 8));
}

#[test]
fn error_message_binding_resolves() {
    let mut line = [0i8; 80];
    unsafe { ErrorMessage(1, line.as_mut_ptr()) };
    let text = unsafe { CStr::from_ptr(line.as_ptr()) };
    assert_eq!(text.to_str().expect("error text is ASCII"), "Success");
}
