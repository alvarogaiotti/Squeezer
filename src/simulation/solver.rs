// Copyright (C) 2025 Alvaro Gaiotti
// See end of file for license information

//! # Shared batch-solve engine
//!
//! The single home of DDS FFI knowledge for Monte Carlo simulations. Three
//! `run()` implementations (contract payoff, card payoff, lead) used to
//! each own a copy of the same spine — generate deals, fan out to
//! `DdsDeal`s, solve in `MAXNOOFBOARDS` batches, drain results — with three
//! subtly different batching schemes, three `.build().unwrap()`s, and two
//! different solver types. They now share one private chunking core with
//! two thin typed wrappers:
//!
//! - [`solve_tricks_batches`]: one trick count per variant (`payoff.rs`).
//! - [`solve_all_cards_batches`]: full `FutureTricks` per deal (`lead.rs`).
//!
//! Callers supply only the per-deal fan-out
//! (`FnMut(&Deal, deal_index, &mut Vec<DdsDeal>)`) and keep their own
//! accumulation (scoring, pairwise, histograms). Nothing here is public:
//! the engine concentrates FFI knowledge, it does not export it.
//!
//! # Test discipline
//!
//! Simulation tests **must** run single-threaded
//! (`cargo test -- --test-threads=1`): DDS global state segfaults under
//! parallel test threads. Tests that invoke the solver additionally hold
//! [`dds_test_lock`], serializing solver use even if the harness runs
//! threads.

use crate::prelude::*;
use dds::{
    MAXNOOFBOARDS,
    deal::{DdsDeal, DdsDealBuilder},
    doubledummy::MultiThreadDoubleDummySolver,
    future_tricks::FutureTricks,
    solver::BridgeSolver,
};
use std::sync::{Mutex, MutexGuard};

/// Serializes solver use across simulation tests.
///
/// DDS keeps global state that segfaults when two test threads solve
/// concurrently. Hold the guard for the whole solver-invoking test; see the
/// module docs. Poisoning aborts the test with a clear message rather than
/// hanging on a dead thread's lock.
static DDS_TEST_LOCK: Mutex<()> = Mutex::new(());

/// Acquires the cross-test DDS serialization lock.
pub(crate) fn dds_test_lock() -> MutexGuard<'static, ()> {
    DDS_TEST_LOCK
        .lock()
        .expect("DDS test lock poisoned by a panicking solver test")
}

/// Per-deal trick counts in variant order, with the global deal index.
#[derive(Debug, Clone)]
pub(crate) struct DealTricks {
    /// Index of the deal in `0..num_deals`.
    pub deal_index: usize,
    /// One trick count per variant, in fan-out order.
    pub tricks: Vec<u8>,
}

/// Per-deal all-cards result with the global deal index.
#[derive(Debug, Clone)]
pub(crate) struct DealCards {
    /// Index of the deal in `0..num_deals`.
    pub deal_index: usize,
    /// Full per-lead analysis for the deal.
    pub future: FutureTricks,
}

/// Maps a finished `DdsDealBuilder` into the crate error with context.
///
/// This replaces the old `.build().unwrap()`s: an impossible deal (duplicate
/// card, missing cards) is a bridge-level setup error naming the simulation
/// and deal index, never a panic — matching the `Simulation::run` contract.
pub(crate) fn finish_deal(
    builder: DdsDealBuilder,
    sim: &'static str,
    deal_index: usize,
) -> Result<DdsDeal, SqueezerError> {
    builder.build().map_err(|cause| {
        SqueezerError::DealingError(DealerError::new(format!(
            "{sim}: invalid DDS deal for deal {deal_index}: {cause}"
        )))
    })
}

/// Generates deals and fans each out to `DdsDeal`s in `MAXNOOFBOARDS` batches.
///
/// Private chunking core behind both wrappers. Returns one entry per batch:
/// the first global deal index plus that batch's flat `DdsDeal` buffer
/// (deal-major, variant-minor — at most `MAXNOOFBOARDS` entries).
///
/// # Errors
///
/// - `SqueezerError::InvalidInput` when `variants_per_deal` is not in
///   `1..=MAXNOOFBOARDS`, or when a fan-out call pushes any other count
///   (a misaligned buffer would silently scramble results).
/// - Whatever the fan-out closure returns (e.g. [`finish_deal`] failures),
///   or dealer/solve errors via `?`.
fn fill_batches<D, F>(
    dealer: &D,
    num_deals: usize,
    variants_per_deal: usize,
    sim: &'static str,
    mut fan_out: F,
) -> Result<Vec<(usize, Vec<DdsDeal>)>, SqueezerError>
where
    D: Dealer + ?Sized,
    F: FnMut(&Deal, usize, &mut Vec<DdsDeal>) -> Result<(), SqueezerError>,
{
    if !(1..=MAXNOOFBOARDS).contains(&variants_per_deal) {
        return Err(SqueezerError::InvalidInput(format!(
            "{sim}: variants per deal ({variants_per_deal}) must be in 1..={MAXNOOFBOARDS}"
        )));
    }
    // Deals per batch: fills the 200-board buffer without overflow, so the
    // old `200 - (200 % n)` divisibility arithmetic is unnecessary.
    let deals_per_batch = MAXNOOFBOARDS / variants_per_deal;
    let mut batches = Vec::new();
    let mut buffer = Vec::with_capacity(MAXNOOFBOARDS);

    for chunk_start in (0..num_deals).step_by(deals_per_batch) {
        let chunk_length = (num_deals - chunk_start).min(deals_per_batch);
        buffer.clear();
        for i in 0..chunk_length {
            let deal_index = chunk_start + i;
            let deal = dealer.deal()?;
            let before = buffer.len();
            fan_out(&deal, deal_index, &mut buffer)?;
            if buffer.len() - before != variants_per_deal {
                return Err(SqueezerError::InvalidInput(format!(
                    "{sim}: fan-out pushed {} variants for deal {deal_index}, expected {variants_per_deal}",
                    buffer.len() - before
                )));
            }
        }
        batches.push((chunk_start, std::mem::take(&mut buffer)));
    }
    Ok(batches)
}

/// Solves every deal for `variants_per_deal` variants, trick counts only.
///
/// # Errors
///
/// Propagates dealer, fan-out, and DDS solve errors; see [`fill_batches`].
pub(crate) fn solve_tricks_batches<D, F>(
    dealer: &D,
    num_deals: usize,
    variants_per_deal: usize,
    sim: &'static str,
    fan_out: F,
) -> Result<Vec<DealTricks>, SqueezerError>
where
    D: Dealer + ?Sized,
    F: FnMut(&Deal, usize, &mut Vec<DdsDeal>) -> Result<(), SqueezerError>,
{
    // Unified solver (was: single-thread `DoubleDummySolver` on the contract
    // path, multi-thread elsewhere): the single-thread solver segfaults when
    // driven from worker threads, exactly how simulations are used.
    let solver = MultiThreadDoubleDummySolver::new();
    let mut entries = Vec::with_capacity(num_deals);
    for (chunk_start, buffer) in fill_batches(dealer, num_deals, variants_per_deal, sim, fan_out)? {
        let total = i32::try_from(buffer.len()).map_err(|_| {
            SqueezerError::InvalidInput(format!(
                "{sim}: batch of {} exceeds DDS batch limit",
                buffer.len()
            ))
        })?;
        let tricks = solver.dd_tricks_parallel(total, &buffer)?;
        // The solver returns exactly one trick count per board; any other
        // count is a broken invariant, not a shortfall to pad or a surplus
        // to trim.
        if tricks.len() != buffer.len() {
            return Err(SqueezerError::InvalidInput(format!(
                "{sim}: solver returned {} results for {} boards",
                tricks.len(),
                buffer.len()
            )));
        }
        for (i, chunk) in tricks.chunks(variants_per_deal).enumerate() {
            entries.push(DealTricks {
                deal_index: chunk_start + i,
                tricks: chunk.to_vec(),
            });
        }
    }
    Ok(entries)
}

/// Solves every deal for all possible leads.
///
/// # Errors
///
/// Propagates dealer, fan-out, and DDS solve errors; see [`fill_batches`].
pub(crate) fn solve_all_cards_batches<D, F>(
    dealer: &D,
    num_deals: usize,
    sim: &'static str,
    fan_out: F,
) -> Result<Vec<DealCards>, SqueezerError>
where
    D: Dealer + ?Sized,
    F: FnMut(&Deal, usize, &mut Vec<DdsDeal>) -> Result<(), SqueezerError>,
{
    let solver = MultiThreadDoubleDummySolver::new();
    let mut entries = Vec::with_capacity(num_deals);
    // One variant per deal on the lead path.
    for (chunk_start, buffer) in fill_batches(dealer, num_deals, 1, sim, fan_out)? {
        let total = i32::try_from(buffer.len()).map_err(|_| {
            SqueezerError::InvalidInput(format!(
                "{sim}: batch of {} exceeds DDS batch limit",
                buffer.len()
            ))
        })?;
        let boards = solver.dd_tricks_all_cards_parallel(total, &buffer)?;
        // Drain exactly the solved count: `SolvedBoards` always holds 200
        // entries, and iterating all of them overcounts deal indices on a
        // short final batch.
        for (i, future) in boards.into_iter().take(buffer.len()).enumerate() {
            entries.push(DealCards {
                deal_index: chunk_start + i,
                future,
            });
        }
    }
    Ok(entries)
}

#[cfg(test)]
mod test {
    use super::*;
    use dds::deal::AsDDSDeal;
    use std::str::FromStr;

    /// Fully predealt dealer: every `deal()` returns the same hands, so
    /// batching and indices are deterministic without seeding any RNG.
    fn fixed_dealer() -> impl Dealer {
        DealerBuilder::new()
            .predeal(
                Seat::North,
                Cards::from_str("SAKQJT98765432").expect("fixed cards parse"),
            )
            .expect("fixed cards do not overlap")
            .predeal(
                Seat::East,
                Cards::from_str("HAKQJT98765432").expect("fixed cards parse"),
            )
            .expect("fixed cards do not overlap")
            .predeal(
                Seat::South,
                Cards::from_str("DAKQJT98765432").expect("fixed cards parse"),
            )
            .expect("fixed cards do not overlap")
            .predeal(
                Seat::West,
                Cards::from_str("CAKQJT98765432").expect("fixed cards parse"),
            )
            .expect("fixed cards do not overlap")
            .build()
            .expect("fixed dealer builds")
    }

    fn fan_out_two(
        deal: &Deal,
        deal_index: usize,
        out: &mut Vec<DdsDeal>,
    ) -> Result<(), SqueezerError> {
        for contract in [
            Contract::from_str("1NN").expect("fixed contract parses"),
            Contract::from_str("2SN").expect("fixed contract parses"),
        ] {
            out.push(finish_deal(
                DdsDealBuilder::new()
                    .remain_cards(deal.as_dds_deal())
                    .contract(&contract),
                "test",
                deal_index,
            )?);
        }
        Ok(())
    }

    #[test]
    fn tricks_batches_cover_every_deal_with_contiguous_indices() {
        let _guard = dds_test_lock();
        // 250 deals x 2 variants: batches of 100 deals, indices stay global.
        let solved = solve_tricks_batches(&fixed_dealer(), 250, 2, "test", fan_out_two)
            .expect("fixed deals solve");
        assert_eq!(solved.len(), 250);
        for (i, entry) in solved.iter().enumerate() {
            assert_eq!(entry.deal_index, i);
            assert_eq!(entry.tricks.len(), 2);
        }
    }

    #[test]
    fn tricks_batches_reject_bad_variant_counts() {
        // Zero variants would divide by zero; over-limit overflows the DDS
        // buffer. Both are errors, never panics.
        assert!(solve_tricks_batches(&fixed_dealer(), 10, 0, "test", fan_out_two).is_err());
        assert!(
            solve_tricks_batches(&fixed_dealer(), 10, MAXNOOFBOARDS + 1, "test", fan_out_two)
                .is_err()
        );
    }

    #[test]
    fn tricks_batches_detect_misaligned_fan_out() {
        let _guard = dds_test_lock();
        let misaligned = |deal: &Deal, deal_index: usize, out: &mut Vec<DdsDeal>| {
            // Declares 2 variants but pushes 1: must error, not scramble.
            out.push(finish_deal(
                DdsDealBuilder::new()
                    .remain_cards(deal.as_dds_deal())
                    .contract(&Contract::from_str("1NN").expect("fixed contract parses")),
                "test",
                deal_index,
            )?);
            Ok(())
        };
        assert!(solve_tricks_batches(&fixed_dealer(), 3, 2, "test", misaligned).is_err());
    }

    #[test]
    fn tricks_batches_propagate_fan_out_errors() {
        let failing = |_: &Deal, _: usize, _: &mut Vec<DdsDeal>| -> Result<(), SqueezerError> {
            Err(SqueezerError::InvalidInput("boom".into()))
        };
        assert!(solve_tricks_batches(&fixed_dealer(), 3, 2, "test", failing).is_err());
    }

    #[test]
    fn all_cards_batches_cover_every_deal() {
        let _guard = dds_test_lock();
        // 205 deals: full 200 batch plus a short 5 batch with exact indices.
        let solved = solve_all_cards_batches(
            &fixed_dealer(),
            205,
            "test",
            |deal: &Deal, deal_index: usize, out: &mut Vec<DdsDeal>| {
                out.push(finish_deal(
                    DdsDealBuilder::new()
                        .remain_cards(deal.as_dds_deal())
                        .contract(&Contract::from_str("1NN").expect("fixed contract parses")),
                    "test",
                    deal_index,
                )?);
                Ok(())
            },
        )
        .expect("fixed deals solve");
        assert_eq!(solved.len(), 205);
        for (i, entry) in solved.iter().enumerate() {
            assert_eq!(entry.deal_index, i);
        }
        // Every solved board saw the same fixed deal: identical analysis.
        assert_eq!(solved[0].future.cards, solved[204].future.cards);
    }
}
