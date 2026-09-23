// Copyright (C) 2024 Alvaro Gaiotti
// See end of file for license information

use dds::{analyseplay::PlayAnalyzer, analyseplay::PlayTraceBin, doubledummy::DoubleDummySolver};
use squeezer::{linparser::LinFile, *};
use std::fs;

#[allow(dead_code)]
fn complete_deal_analysis_test() {
    let data = fs::read_to_string("tests/resources/test_hand.lin").expect("fixture is readable");
    let lin_file = LinFile::from_str(&data).expect("should be able to parse lin files");
    let board = lin_file.boards.first().expect("fixture has a board");
    let contract = board.contract().expect("board has a contract");
    let play_record = board.play().expect("board has a play");
    println!("{play_record:?}");
    let playtrace: PlayTraceBin = play_record
        .try_into()
        .expect("the play record converts to a DDS trace");
    let solver = DoubleDummySolver::new();
    let _analyzed_plays = solver.analyze_play(board.deal(), &contract, playtrace);
}
