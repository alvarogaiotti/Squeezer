// Copyright (C) 2024 Alvaro Gaiotti
// See end of file for license information

use dds::{
    analyseplay::{PlayAnalyzer, PlayTraceBin},
    doubledummy::DoubleDummySolver,
    utils::{RankSeq, SuitSeq},
};
mod setup;
use setup::*;

#[test]
fn analyse_play_test() {
    let deal = initialize_test();
    let contract = ContractMock {};
    let suitseq = SuitSeq::try_from([0i32, 0i32, 0i32, 0i32]).unwrap();
    let rankseq = RankSeq::try_from([4i32, 3i32, 12i32, 2i32]).unwrap();
    let play = PlayTraceBin::from_sequences(suitseq, rankseq);
    let analyzer = DoubleDummySolver::new();
    let solvedplay = analyzer.analyze_play(&deal, &contract, play).unwrap();
    assert_eq!([2, 2, 2, 2, 2], solvedplay.tricks[..5]);
}
