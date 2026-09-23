// Copyright (C) 2024 Alvaro Gaiotti
// See end of file for license information

use dds::context::{self, FutureTricks};
use dds::deal::{AsDDSDeal, DdsDealBuilder};
mod setup;
use setup::*;

#[test]
fn test_linkage() {
    let deal = initialize_test();
    let contract = ContractMock {};
    let dds_deal = DdsDealBuilder::new()
        .contract(&contract)
        .remain_cards(deal.as_dds_deal())
        .build()
        .expect("the mock deal is valid");

    // Solve through a caller-owned context (the parallel-safe path) rather than
    // the legacy `SolveBoard` entry point, and assert the binding linked and
    // returned a plausible score.
    let ctx = unsafe { context::dds_c_create_solvercontext_default() };
    assert!(!ctx.is_null(), "context creation returned NULL");
    let mut future = FutureTricks::default();
    // target = -1 (max tricks), solutions = 1 (best), mode = 1 (auto reuse).
    let result = unsafe { context::dds_c_solve_board(ctx, &dds_deal, -1, 1, 1, &mut future) };
    unsafe { context::dds_c_destroy_solvercontext(ctx) };

    assert_eq!(result, 1);
    assert!(
        (0..=13).contains(&future.score[0]),
        "expected a trick count in 0..=13, got {}",
        future.score[0]
    );
}
