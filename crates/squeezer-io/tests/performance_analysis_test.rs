// Copyright (C) 2024 Alvaro Gaiotti
// See end of file for license information

//! End-to-end performance-analysis pipeline over the public API.
//!
//! A LIN file becomes a `Board`, a `Position` is derived from it, the solver
//! seam's `PlayAnalyser` produces a backend-agnostic per-play double-dummy
//! sequence, and `analyze_board` reports per-seat statistics. No `dds` type
//! appears on this path: the analyser's result is a plain `Vec<u8>`, and the
//! only solver type named is `squeezer-solver`'s `DdsSolver`.

use std::path::PathBuf;
use std::str::FromStr;

use squeezer_io::linparser::LinFile;
use squeezer_model::{Position, Seat};
use squeezer_sim::performance_analysis::{HandAnalysis, analyze_board};
use squeezer_solver::{DdsSolver, PlayAnalyser};

/// The fixture lives with the importer that reads it, so it resolves from the
/// crate manifest rather than the process working directory.
fn analysis_fixture() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/83469.lin")
}

/// Board 15 of `tests/fixtures/83469.lin`: a 4H contract by West with a full
/// 52-card play. The per-seat baselines are DDS 3.1 (the `AnalysePlay` #156 fix),
/// re-baselined from the 2.9 values the analysis used to encode.
#[test]
fn lin_file_to_board_to_per_seat_statistics() {
    let data = std::fs::read_to_string(analysis_fixture()).expect("fixture is readable");
    let file = LinFile::from_str(&data).expect("83469 parses");
    let board = &file.boards[0];

    // A `Position` is derived from the board on the way to the solver seam;
    // West declares 4H, so North is on opening lead.
    let opening = Position::after_play(board, 0).expect("the board has a contract");
    assert_eq!(opening.to_play(), Seat::North);

    // The seam is backend-agnostic: the result is a plain trick-count vector,
    // one entry per play plus the starting position. DDS caps its internal
    // sequence at 49, so the extra entries are the forced final trick.
    let dd_tricks: Vec<u8> = DdsSolver::new()
        .analyse_play(board)
        .expect("board 15 analyses");
    assert_eq!(dd_tricks.len(), 53, "52 plays plus the starting position");

    // The analysis drives the seam itself and keeps per-seat and per-side
    // statistics.
    let analysis: HandAnalysis =
        analyze_board(&DdsSolver::new(), board).expect("board 15 analyses");

    let north = analysis.player_stats(Seat::North);
    assert_eq!(north.cards_played, 13);
    assert_eq!(north.dd_optimal_plays, 12);
    assert_eq!(north.dd_total_tricks_lost, 1);

    let east = analysis.player_stats(Seat::East);
    assert_eq!(east.cards_played, 13);
    assert_eq!(east.dd_optimal_plays, 13);
    assert_eq!(east.dd_total_tricks_lost, 0);

    let south = analysis.player_stats(Seat::South);
    assert_eq!(south.cards_played, 13);
    assert_eq!(south.dd_optimal_plays, 13);
    assert_eq!(south.dd_total_tricks_lost, 0);

    let west = analysis.player_stats(Seat::West);
    assert_eq!(west.cards_played, 13);
    assert_eq!(west.dd_optimal_plays, 12);
    assert_eq!(west.dd_total_tricks_lost, 1);

    // Per-side statistics: West declares, so EW is the declaring side.
    let declaring = analysis.declaring_side_stats();
    assert_eq!(declaring.cards_played, 26);
    assert_eq!(declaring.dd_optimal_plays, 25);
    assert_eq!(declaring.dd_total_tricks_lost, 1);

    let defending = analysis.defending_side_stats();
    assert_eq!(defending.cards_played, 26);
    assert_eq!(defending.dd_optimal_plays, 25);
    assert_eq!(defending.dd_total_tricks_lost, 1);
}
