// Copyright (C) 2024 Alvaro Gaiotti
// See end of file for license information

use dds::deal::{AsDDSDeal, DdsDealBuilder};
use dds::doubledummy::MultiThreadDoubleDummySolver;
use dds::solver::BridgeSolver;
mod setup;
use setup::*;

#[test]
fn test_linkage() {
    let deal = initialize_test();
    let contract = ContractMock {};
    let solver = MultiThreadDoubleDummySolver::new();
    let dds_deal = DdsDealBuilder::new()
        .contract(&contract)
        .remain_cards(deal.as_dds_deal())
        .build()
        .unwrap();
    println!("{}", solver.dd_tricks(&dds_deal).unwrap());
    println!("{}", dds::utils::dd_score(&deal, &contract).unwrap());
}
