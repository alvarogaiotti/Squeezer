use dds::{
    MultiThreadDoubleDummySolver, PlayAnalyzer, PlayTraceBin, PlayTracesBin, RankSeq, RawDDSRef,
    SuitSeq,
};
mod setup;
use setup::*;

const TRIES: usize = 200;

#[test]
fn analyse_play_test() {
    let deal = initialize_test();
    let contract = ContractMock {};
    let suitseq = SuitSeq::try_from([0i32, 0i32, 0i32, 0i32]).unwrap();
    let rankseq = RankSeq::try_from([4i32, 3i32, 12i32, 2i32]).unwrap();
    let play = PlayTraceBin::from_sequences(suitseq, rankseq);
    let analyzer = MultiThreadDoubleDummySolver::new();
    let solvedplay = analyzer.analyze_play(&deal, &contract, play).unwrap();
    assert_eq!([2, 2, 2, 2, 2], solvedplay.tricks[..5]);
}

#[test]
fn analyse_all_play_test() {
    let mut deals_owner = Vec::with_capacity(TRIES);
    deals_owner.resize_with(TRIES, initialize_test);
    let suitseq = SuitSeq::try_from([0, 0, 0, 0]).unwrap();
    let rankseq = RankSeq::try_from([4, 3, 12, 2]).unwrap();
    let mut suitseqs = Vec::with_capacity(TRIES);
    let mut rankseqs = Vec::with_capacity(TRIES);
    suitseqs.resize_with(TRIES, || suitseq.clone());
    rankseqs.resize_with(TRIES, || rankseq.clone());
    let contracts_owner = Vec::from([ContractMock {}; TRIES]);
    let mut plays = PlayTracesBin::from_sequences(suitseqs, rankseqs).unwrap();
    let analyzer = MultiThreadDoubleDummySolver::new();
    let solved_plays = analyzer
        .analyze_all_plays(&deals_owner, &contracts_owner, &mut plays)
        .unwrap();
    assert_eq!(TRIES, solved_plays.no_of_boards.try_into().unwrap());
    for plays in solved_plays.solved {
        assert_eq!([2, 2, 2, 2, 2], plays.tricks[..5]);
    }
}
