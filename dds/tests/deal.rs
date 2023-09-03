use dds::BridgeSolver;
mod setup;
use setup::*;

#[test]
fn test_linkage() {
    let deal = initialize_test();
    let contract = ContractMock {};
    let solver = dds::DoubleDummySolver::solver();
    println!("{}", solver.dd_tricks(&deal, &contract).unwrap());
    println!("{}", dds::dd_score(&deal, &contract).unwrap());
}
