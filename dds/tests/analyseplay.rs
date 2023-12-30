use dds::PlayAnalyzer;
use dds::*;
mod setup;
use setup::*;

const TRIES: usize = 200;

#[test]
fn analyse_play_test() {
    let deal = initialize_test();
    let contract = ContractMock {};
    let suitseq = SuitSeq::try_from([0i32, 0i32, 0i32, 0i32]).unwrap();
    let rankseq = RankSeq::try_from([4i32, 3i32, 12i32, 2i32]).unwrap();
    let play = PlayTraceBin::new(suitseq, rankseq);
    let solvedplay = DDSPlayAnalyzer::analyze_play(&deal, &contract, &play).unwrap();
    assert_eq!([2, 2, 2, 2, 2], solvedplay.solved_play.tricks[..5]);
}

#[test]
fn analyse_all_play_test() {
    let mut deals_owner = Vec::with_capacity(TRIES);
    deals_owner.resize_with(TRIES, initialize_test);
    let deals = deals_owner.iter().collect();
    let suitseq = SuitSeq::try_from([0, 0, 0, 0]).unwrap();
    let rankseq = RankSeq::try_from([4, 3, 12, 2]).unwrap();
    let mut suitseqs = Vec::with_capacity(TRIES);
    let mut rankseqs = Vec::with_capacity(TRIES);
    suitseqs.resize_with(TRIES, || suitseq.clone());
    rankseqs.resize_with(TRIES, || rankseq.clone());
    let contracts_owner = Vec::from([ContractMock {}; TRIES]);
    let contracts = contracts_owner.iter().collect();
    let mut plays = PlayTracesBin::from_sequences(suitseqs, rankseqs).unwrap();
    let solved_plays = DDSPlayAnalyzer::analyze_all_plays(deals, contracts, &mut plays).unwrap();
    let real_plays = solved_plays.get_raw();
    assert_eq!(TRIES, real_plays.noOfBoards.try_into().unwrap());
    for plays in real_plays.solved {
        assert_eq!([2, 2, 2, 2, 2], plays.tricks[..5]);
    }
}
