use dds::PlayAnalyzer;
use dds::*;
mod setup;
use setup::*;

#[test]
fn AnalysePlay_test() {
    let deal = initialize_test();
    let contract = ContractMock {};
    let suitseq = SuitSeq::new(&[0, 0, 0, 0]);
    let rankseq = RankSeq::new(&[4, 3, 12, 2]);
    let mut play = PlayTraceBin::new(suitseq, rankseq);
    let solvedplay = DDSPlayAnalyzer::analyze_play(&deal, contract, &mut play);
}
