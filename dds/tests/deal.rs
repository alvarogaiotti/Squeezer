// Copyright (C) 2024 Alvaro Gaiotti
// See end of file for license information

use dds::doubledummy::MultiThreadDoubleDummySolver;
use dds::solver::BridgeSolver;
mod setup;
use setup::*;

#[test]
fn test_linkage() {
    let deal = initialize_test();
    let contract = ContractMock {};
    let solver = MultiThreadDoubleDummySolver::new();
    println!("{}", solver.dd_tricks(&deal, &contract).unwrap());
    println!("{}", dds::utils::dd_score(&deal, &contract).unwrap());
}
