// Copyright (C) 2024 Alvaro Gaiotti
// See end of file for license information

use dds::{
    analyseplay::PlayAnalyzer,
    analyseplay::PlayTraceBin,
    doubledummy::DoubleDummySolver,
    utils::{RankSeq, SuitSeq},
};
use squeezer::{linparser::LinFile, *};
use std::fs;

#[allow(dead_code)]
fn complete_deal_analysis_test() {
    let data = fs::read_to_string("tests/resources/test_hand.lin").unwrap();
    let lin_file = LinFile::from_str(&data).expect("should be able to parse lin files");
    let deal = lin_file.deals.first().unwrap();
    let contract = deal.contract();
    assert!(contract.is_some());
    let contract = contract.unwrap();
    let play_sequence = deal.play_sequence();
    assert!(play_sequence.is_some());
    println!("{:?}", play_sequence.unwrap());
    let (suitseq, rankseq): (SuitSeq, RankSeq) = play_sequence.unwrap().try_into().unwrap();
    let playtrace = PlayTraceBin::from_sequences(suitseq, rankseq);
    let solver = DoubleDummySolver::new();
    let _analyzed_plays = solver.analyze_play(deal, &contract, playtrace);
}
