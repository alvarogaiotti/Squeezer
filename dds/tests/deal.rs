// Copyright (C) 2024 Alvaro Gaiotti
// See end of file for license information

use dds::context::{self, FutureTricks};
use dds::deal::{AsDDSDeal, ConstructDdsDealError, DdsDealBuilder, DdsHandEncoding, DdsSuit};
mod setup;
use setup::*;

#[test]
fn builder_rejects_a_card_dealt_to_two_hands() {
    // North and East both hold the spade ace: this is the condition DDS reports
    // as RETURN_DUPLICATE_CARDS, so the builder must reject it before the FFI
    // call and say which card is duplicated. Rank bit 14 is the ace.
    const SPADE_ACE: u32 = 1 << 14;
    let mut hands = [[0u32; 4]; 4];
    hands[DdsHandEncoding::North as usize][DdsSuit::Spades as usize] = SPADE_ACE;
    hands[DdsHandEncoding::East as usize][DdsSuit::Spades as usize] = SPADE_ACE;

    let error = DdsDealBuilder::new()
        .contract(&ContractMock {})
        .remain_cards(hands.into())
        .build()
        .expect_err("a duplicated card is not a legal deal");

    let expected_rank = i32::try_from(SPADE_ACE).expect("the ace bit fits an i32");
    assert!(matches!(
        error,
        ConstructDdsDealError::DuplicatedCard(0, rank) if rank == expected_rank
    ));
    assert_eq!(error.to_string(), "duplicated card: ♠A");
}

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
