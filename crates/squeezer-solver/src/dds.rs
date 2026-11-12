// Copyright (C) 2025 Alvaro Gaiotti
// See end of file for license information

//! The DDS-backed [`Solver`], behind the `dds` feature.
//!
//! This is the only place that knows how to turn a [`Position`] into DDS's raw
//! `Deal` and drive `dds_c_solve_board`. The `dds` crate is used as a plain
//! FFI surface: no DDS type appears in the seam's public API, and the raw
//! context handle never leaves this module.

use super::{
    CardSolve, DdTable, DdTableCalculator, PlayAnalyser, Solutions, SolveMode, SolveResults,
    Solver, SolverContext, SolverError,
};
use crate::adapters::{
    DdsDealAdapter, dd_table_from_dds, play_record_to_trace, position_to_dds_deal,
};
use crate::prelude::*;
use ::core::ffi::{c_int, c_void};
use dds::context::{self, DdTableResults, FutureTricks, SolvedPlay};
use dds::tables::NotPopulated;

/// DDS's "no fault" return code; every other code is an error.
const RETURN_NO_FAULT: c_int = 1;
/// `target = -1` asks DDS for the maximum number of tricks.
const TARGET_MAX_TRICKS: c_int = -1;

/// The DDS solver factory.
#[derive(Debug, Clone, Copy, Default)]
pub struct DdsSolver;

impl DdsSolver {
    /// Creates the DDS factory.
    #[must_use]
    pub const fn new() -> Self {
        Self
    }
}

impl Solver for DdsSolver {
    type Context = DdsContext;

    fn create_context(&self) -> Self::Context {
        // A NULL handle is kept rather than panicking: creation failure is a
        // resource error the caller should be able to observe from `solve`,
        // not a reason to abort a worker thread.
        DdsContext {
            raw: unsafe { context::dds_c_create_solvercontext_default() },
        }
    }
}

/// A DDS solver context: one transposition table, used by one thread.
pub struct DdsContext {
    raw: *mut c_void,
}

// SAFETY: `DdsContext` owns its handle exclusively. DDS contexts are not
// shared between threads, every method takes `&mut self`, and `Drop` releases
// the handle on the owning thread, so moving the handle to another thread
// carries no aliasing.
unsafe impl Send for DdsContext {}

impl Drop for DdsContext {
    fn drop(&mut self) {
        if !self.raw.is_null() {
            unsafe { context::dds_c_destroy_solvercontext(self.raw) };
        }
    }
}

impl DdsContext {
    /// The raw handle, or [`SolverError::ContextCreation`] if it is null.
    fn handle(&self) -> Result<*mut c_void, SolverError> {
        if self.raw.is_null() {
            Err(SolverError::ContextCreation)
        } else {
            Ok(self.raw)
        }
    }
}

impl SolverContext for DdsContext {
    fn solve(
        &mut self,
        position: &Position,
        mode: SolveMode,
        solutions: Solutions,
    ) -> Result<SolveResults, SolverError> {
        let raw = self.handle()?;
        let deal = position_to_dds_deal(position)
            .map_err(|error| SolverError::Position(error.to_string()))?;
        let mut future = FutureTricks::default();
        let code = unsafe {
            context::dds_c_solve_board(
                raw,
                &raw const deal,
                TARGET_MAX_TRICKS,
                solutions_code(solutions),
                mode_code(mode),
                &raw mut future,
            )
        };
        if code != RETURN_NO_FAULT {
            return Err(backend_error(code));
        }
        Ok(read_results(&future))
    }

    fn clear_tt(&mut self) {
        if let Ok(raw) = self.handle() {
            unsafe { context::dds_c_clear_tt(raw) };
        }
    }

    fn reset(&mut self) {
        if let Ok(raw) = self.handle() {
            unsafe { context::dds_c_reset_for_solve(raw) };
        }
    }
}

/// A cold capability: the full DD table for an unplayed deal.
impl DdTableCalculator for DdsSolver {
    /// Computes the table on a fresh context of its own.
    ///
    /// A DD table is a per-deal operation, not a per-solve one, so it does not
    /// borrow a worker's context: this keeps the factory usable from any thread
    /// and leaves the worker's transposition table untouched. The new context
    /// is dropped with this call.
    fn dd_table(&self, deal: &Deal) -> Result<DdTable, SolverError> {
        let context = Solver::create_context(self);
        let raw = context.handle()?;
        let table_deal = DdsDealAdapter(deal).to_table_deal();
        let mut results = DdTableResults::<NotPopulated>::default();
        let code =
            unsafe { context::dds_c_calc_dd_table(raw, &raw const table_deal, &raw mut results) };
        if code != RETURN_NO_FAULT {
            return Err(backend_error(code));
        }
        // `dds_c_calc_dd_table` returned success, so the buffer it filled is
        // complete: this is the only place that may claim the marker.
        dd_table_from_dds(&results.populated())
    }
}

impl PlayAnalyser for DdsSolver {
    type Error = SolverError;

    /// Wraps the legacy `AnalysePlayBin`, which builds its own solver context
    /// per call and ignores the thread index, so it is safe to call from our
    /// workers. The board must carry a contract and a recorded play: the
    /// opening [`Position`] supplies the full deal and the leader, and the
    /// [`PlayRecord`] becomes the seat-free `PlayTraceBin`.
    fn analyse_play(&self, board: &Board) -> Result<Vec<u8>, SolverError> {
        let position = Position::after_play(board, 0)
            .map_err(|error| SolverError::Position(error.to_string()))?;
        let deal = position_to_dds_deal(&position)
            .map_err(|error| SolverError::Position(error.to_string()))?;
        let play = board
            .play()
            .ok_or_else(|| SolverError::Position("the board records no play".to_owned()))?;
        let trace =
            play_record_to_trace(play).map_err(|error| SolverError::Position(error.to_string()))?;

        let mut solved = SolvedPlay::new();
        let code = unsafe { context::AnalysePlayBin(deal, trace, &raw mut solved, 0) };
        if code != RETURN_NO_FAULT {
            return Err(backend_error(code));
        }
        // DDS reports the starting position then the result after each card,
        // but caps the sequence at 49 entries ("the last trick holds no
        // excitement"). Every omitted play is in the forced final trick — once
        // 48 cards are down, each hand holds exactly one card — so repeating
        // the last count restores the one-entry-per-play convention.
        //
        // `number` and the counts are DDS output, not caller input: a negative
        // length, a length past the fixed buffer, a count outside 0..=13 or an
        // empty result means DDS broke its contract, so surface it rather than
        // defaulting to a plausible-but-wrong `0`.
        let number = usize::try_from(solved.number).map_err(|_| {
            SolverError::Backend(format!(
                "DDS reported a negative play-analysis length: {}",
                solved.number
            ))
        })?;
        let results = solved.tricks.get(..number).ok_or_else(|| {
            SolverError::Backend(format!(
                "DDS reported {number} play-analysis results for a {} slot buffer",
                solved.tricks.len()
            ))
        })?;
        let mut tricks = Vec::with_capacity(play.len() + 1);
        for &trick in results {
            tricks.push(u8::try_from(trick).map_err(|_| {
                SolverError::Backend(format!("DDS returned an out-of-range trick count: {trick}"))
            })?);
        }
        let last = tricks.last().copied().ok_or_else(|| {
            SolverError::Backend("DDS returned no play-analysis results".to_owned())
        })?;
        tricks.resize(play.len() + 1, last);
        Ok(tricks)
    }
}

/// Wraps a non-success DDS return code as the seam's backend error.
///
/// Callers check the code for [`RETURN_NO_FAULT`] first; this keeps the
/// conversion (and the DDS error vocabulary) out of every DDS call site.
fn backend_error(code: c_int) -> SolverError {
    let error = dds::ddserror::DdsError::from(code);
    SolverError::Backend(error.to_string())
}

/// Translates the seam's solution level into DDS's `solutions` code.
fn solutions_code(solutions: Solutions) -> c_int {
    match solutions {
        Solutions::OneDdOptimal => 1,
        Solutions::AllDdOptimal => 2,
        Solutions::AllLegal => 3,
    }
}

/// Translates the seam's reuse knob into DDS's `mode` code.
///
/// `AutoReuse` is DDS mode 1 (automatic reuse, always search), not mode 0:
/// mode 0 returns a sentinel score for a position with a single legal card,
/// which would turn a forced play into a bogus zero.
fn mode_code(mode: SolveMode) -> c_int {
    match mode {
        SolveMode::AutoReuse => 1,
        SolveMode::ForceReuseAlwaysSearch => 2,
    }
}

/// Converts a populated `FutureTricks` into the seam's result type.
fn read_results(future: &FutureTricks) -> SolveResults {
    let count = usize::try_from(future.cards)
        .unwrap_or(0)
        .min(future.suit.len());
    let mut cards = Vec::with_capacity(count);
    for index in 0..count {
        // A malformed entry is skipped rather than aborting the whole solve;
        // DDS only ever emits valid `(suit, rank)` pairs here.
        let (Ok(suit), Ok(rank)) = (
            Suit::try_from(future.suit[index]),
            u8::try_from(future.rank[index]),
        ) else {
            continue;
        };
        if !(2..=14).contains(&rank) {
            continue;
        }
        let card = Card::new(suit, rank);
        cards.push(CardSolve {
            card,
            tricks: u8::try_from(future.score[index]).unwrap_or(0),
            equivalents: expand_equals(card, future.equals[index]),
        });
    }
    SolveResults { cards }
}

/// Expands DDS's `equals` bitmask (bit *n* = rank *n*) into physical cards of
/// `card`'s suit, including `card` itself.
fn expand_equals(card: Card, equals: c_int) -> Cards {
    let suit = card.suit();
    let bits = u64::try_from(equals).unwrap_or(0);
    let mut cards = Cards::EMPTY.insert(card);
    for rank in 2..=14u8 {
        if bits & (1u64 << rank) != 0 {
            cards = cards.insert(Card::new(suit, rank));
        }
    }
    cards
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_fixtures::{
        example_board, example_board_hand0_play, example_board_with, example_deal, opening_position,
    };
    use crate::{ContextFactory, DdTableCalculator};
    use std::str::FromStr;

    fn card(suit: Suit, rank: u8) -> Card {
        Card::new(suit, rank)
    }

    /// A fresh concrete DDS context.
    fn context() -> DdsContext {
        Solver::create_context(&DdsSolver)
    }

    #[test]
    fn dd_table_matches_the_dds_example_hand() {
        let table = DdsSolver
            .dd_table(&example_deal())
            .expect("example hand 0 has a DD table");

        // DDS's `examples/hands.cpp` hand 0, with one row per strain and one
        // column per seat. This is DDS's own oracle, not a recomputation of the
        // adapter's output.
        let expected = [
            [5, 8, 5, 8], // spades
            [6, 6, 6, 6], // hearts
            [5, 7, 5, 7], // diamonds
            [7, 5, 7, 5], // clubs
            [6, 6, 6, 6], // no trumps
        ];
        for (strain, row) in Strain::ALL.into_iter().zip(expected) {
            for (seat, tricks) in Seat::iter().zip(row) {
                assert_eq!(table.tricks(strain, seat), tricks, "{strain} by {seat}");
            }
        }
    }

    #[test]
    fn a_boxed_factory_calculates_a_dd_table_through_the_seam() {
        // The surface a simulation holds: only the object-safe factory, with no
        // backend type in sight, must still answer the cold table capability.
        let factory: Box<dyn ContextFactory> = Box::new(DdsSolver::new());
        let table = factory
            .dd_table(&example_deal())
            .expect("example hand 0 has a DD table");
        assert_eq!(table.tricks(Strain::Spades, Seat::North), 5);

        let mut context = factory.new_context();
        assert_eq!(
            context
                .solve_tricks(&opening_position())
                .expect("example hand 0 solves"),
            5
        );
    }

    #[test]
    fn solve_all_cards_matches_the_dds_example_hand() {
        let board = example_board();
        let position = Position::after_play(&board, 0).expect("contract present");
        let mut context = context();

        let results = context
            .solve_all_cards(&position)
            .expect("example hand 0 solves");

        // Values ported from DDS's own examples via `dds/tests/context.rs`.
        let expected = [
            (Suit::Diamonds, 5, 5),
            (Suit::Diamonds, 8, 5),
            (Suit::Diamonds, 11, 5),
            (Suit::Clubs, 10, 5),
            (Suit::Spades, 6, 5),
            (Suit::Spades, 12, 5),
            (Suit::Hearts, 2, 4),
            (Suit::Hearts, 6, 4),
            (Suit::Hearts, 13, 4),
        ];
        let actual: Vec<_> = results
            .cards
            .iter()
            .map(|solve| (solve.card.suit(), solve.card.rank(), solve.tricks))
            .collect();
        assert_eq!(actual, expected);
    }

    #[test]
    fn solve_all_cards_expands_equivalence_classes() {
        let board = example_board();
        let position = Position::after_play(&board, 0).expect("contract present");
        let mut context = context();
        let results = context
            .solve_all_cards(&position)
            .expect("example hand 0 solves");

        let class = |suit, rank| {
            results
                .cards
                .iter()
                .find(|solve| solve.card == card(suit, rank))
                .expect("card was returned")
                .equivalents
        };
        // DDS `equals` for C10 is {C9, C8}, for SQ is {SJ}, for H6 is {H5}.
        assert_eq!(
            class(Suit::Clubs, 10),
            Cards::from_str("CT98").expect("parses")
        );
        assert_eq!(
            class(Suit::Spades, 12),
            Cards::from_str("SQJ").expect("parses")
        );
        assert_eq!(
            class(Suit::Hearts, 6),
            Cards::from_str("H65").expect("parses")
        );
        // A card with no equivalents is its own class.
        assert_eq!(
            class(Suit::Diamonds, 5),
            Cards::from_str("D5").expect("parses")
        );
    }

    #[test]
    fn solve_tricks_matches_the_example_hand() {
        let board = example_board();
        let position = Position::after_play(&board, 0).expect("contract present");
        let mut context = context();

        assert_eq!(
            context
                .solve_tricks(&position)
                .expect("example hand 0 solves"),
            5
        );
    }

    #[test]
    fn position_bridge_uses_the_trick_leader_as_first() {
        // North led S6, East followed S3; South is next. DDS's `first` must
        // name the *leader* (North), not the seat to play (South), and the
        // current-trick ranks are sequential (2..14), not bit-flags.
        let board = example_board_with(vec![
            Play::new(Seat::North, Card::S6),
            Play::new(Seat::East, Card::S3),
        ]);
        let position = Position::after_play(&board, 2).expect("contract present");
        assert_eq!(position.to_play(), Seat::South);

        let deal = position_to_dds_deal(&position).expect("mid-trick position converts");
        assert_eq!(deal.first(), Seat::North as i32);
        assert_eq!(deal.trump(), 0);
        assert_eq!(deal.current_trick_suit(), [0, 0, 0]);
        assert_eq!(deal.current_trick_rank(), [6, 3, 0]);

        // South holds SK and S5 and must follow suit, so exactly two cards.
        let mut context = context();
        let results = context
            .solve_all_cards(&position)
            .expect("mid-trick position solves");
        assert_eq!(results.cards.len(), 2);
    }

    #[test]
    fn position_bridge_encodes_every_suit_rank_bit() {
        let board = example_board();
        let position = Position::after_play(&board, 0).expect("contract present");
        let actual = position_to_dds_deal(&position)
            .expect("opening position converts")
            .remain_cards();

        // Keep an independent card-by-card reference here: the optimized
        // bridge must preserve the rank-bit layout for all four suits, not
        // merely produce a deal that the DDS oracle happens to accept.
        let mut expected = [[0u32; 4]; 4];
        for (seat, hand) in position.hands().iter().enumerate() {
            for card in *hand {
                expected[seat][usize::from(card.suit())] |= 1u32 << card.rank();
            }
        }

        assert_eq!(actual, expected);
    }

    #[test]
    fn clear_and_reset_are_callable() {
        let mut context = context();
        context.clear_tt();
        context.reset();
    }

    #[test]
    fn play_analysis_matches_the_dds_example_hand() {
        // DDS's own `examples/hands.cpp` oracle for hand 0: 45 cards, 46
        // results, every one 8. This is the re-baselined DDS 3.1 semantics
        // (issue #156), not the under-counted 2.9 values.
        let board = example_board_hand0_play();
        let results = DdsSolver
            .analyse_play(&board)
            .expect("example hand 0 analyses");
        assert_eq!(results, vec![8u8; 46]);
    }

    #[test]
    fn concurrent_contexts_match_sequential_results() {
        use rayon::prelude::*;

        // Three positions from the example deal: opening lead, after one play,
        // after two. They differ enough that a shared transposition table would
        // be caught by a wrong trick count.
        let boards = [
            example_board(),
            example_board_with(vec![Play::new(Seat::North, Card::S6)]),
            example_board_with(vec![
                Play::new(Seat::North, Card::S6),
                Play::new(Seat::East, Card::S3),
            ]),
        ];
        let positions: Vec<Position> = boards
            .iter()
            .enumerate()
            .map(|(n, board)| Position::after_play(board, n).expect("contract present"))
            .collect();

        // The oracle solves every position on one context, sequentially.
        let mut oracle = context();
        let sequential: Vec<u8> = positions
            .iter()
            .map(|position| oracle.solve_tricks(position).expect("example hand solves"))
            .collect();

        // Each concurrent solve gets its own context, so no solver state is
        // shared between threads: this is the property the global mutex used to
        // stand in for.
        let concurrent: Vec<u8> = positions
            .par_iter()
            .map(|position| {
                let mut context = context();
                context.solve_tricks(position).expect("example hand solves")
            })
            .collect();

        assert_eq!(sequential, concurrent);
    }
}
