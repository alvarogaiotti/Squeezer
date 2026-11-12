// Copyright (C) 2025 Alvaro Gaiotti
// See end of file for license information

//! The DDS-backed [`Solver`], behind the `dds` feature.
//!
//! This is the only place that knows how to turn a [`Position`] into DDS's raw
//! `Deal` and drive `dds_c_solve_board`. The `dds` crate is used as a plain
//! FFI surface: no DDS type appears in the seam's public API, and the raw
//! context handle never leaves this module.

use super::{
    CardSolve, DdTable, DdTableCalculator, Par, ParCalculator, PlayAnalyser, Solutions, SolveMode,
    SolveResults, Solver, SolverContext, SolverError,
};
use crate::adapters::{
    DdsDealAdapter, dd_table_from_dds, dd_table_to_dds, par_from_dds, play_record_to_trace,
    position_to_dds_deal, vulnerability_encoding,
};
use crate::prelude::*;
use ::core::ffi::{c_int, c_void};
use dds::context::{self, DdTableResults, FutureTricks, ParResults, SolvedPlay};
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
        read_results(&future)
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

/// A cold capability: par from a precomputed DD table.
impl ParCalculator for DdsSolver {
    type Table = DdTable;
    type Par = Par;
    type Error = SolverError;

    /// Derives par with DDS's legacy `Par` entry point.
    ///
    /// Par is a pure function of a precomputed table, so the call takes no
    /// deal. DDS exports it only through the legacy `Par` entry point, which
    /// accepts the populated table directly; the `dds_c_*` context shim has no
    /// table-input par call.
    fn par(&self, table: &DdTable, vulnerability: Vulnerability) -> Result<Par, SolverError> {
        let dds_table = dd_table_to_dds(table);
        let mut results = ParResults::default();
        let code = unsafe {
            context::Par(
                &raw const dds_table,
                &raw mut results,
                vulnerability_encoding(vulnerability) as c_int,
            )
        };
        if code != RETURN_NO_FAULT {
            return Err(backend_error(code));
        }
        par_from_dds(&results)
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
        read_play_results(&solved, play.len())
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

/// The single malformed-output policy: a field DDS broke its own contract on is
/// an error, never a silently skipped entry.
///
/// Both output readers funnel through here — [`read_results`] for solves and
/// [`read_play_results`] for play analysis — so they cannot drift back into one
/// erroring and the other clamping.
fn malformed_output(detail: core::fmt::Arguments<'_>) -> SolverError {
    SolverError::Backend(format!("DDS returned malformed output: {detail}"))
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
///
/// Every field is DDS output, so each one is checked against DDS's documented
/// domain: a card count that does not fit the fixed buffer, a suit outside
/// 0..=3, a rank outside 2..=14, a score outside 0..=13 or a negative
/// equivalence mask means DDS broke its contract and is surfaced as an error.
///
/// The score check relies on this seam always asking for the maximum number of
/// tricks ([`TARGET_MAX_TRICKS`]) in a searching mode ([`mode_code`]), where
/// DDS's score is the trick count. DDS's `-1` (target not reached) and `-2`
/// (mode 0 sentinel) are not reachable through these calls.
fn read_results(future: &FutureTricks) -> Result<SolveResults, SolverError> {
    let count = usize::try_from(future.cards)
        .map_err(|_| malformed_output(format_args!("a negative card count: {}", future.cards)))?;
    if count > future.suit.len() {
        return Err(malformed_output(format_args!(
            "a card count of {count} for a {} slot buffer",
            future.suit.len()
        )));
    }
    let mut cards = Vec::with_capacity(count);
    for index in 0..count {
        let suit = Suit::try_from(future.suit[index]).map_err(|_| {
            malformed_output(format_args!(
                "a card suit outside 0..=3: {}",
                future.suit[index]
            ))
        })?;
        let card = Card::new(suit, card_rank(future.rank[index])?);
        cards.push(CardSolve {
            card,
            tricks: trick_count(future.score[index])?,
            equivalents: expand_equals(card, future.equals[index])?,
        });
    }
    Ok(SolveResults { cards })
}

/// Reads a card rank, rejecting values outside a card's `2..=14`.
fn card_rank(value: c_int) -> Result<u8, SolverError> {
    match u8::try_from(value) {
        Ok(rank) if (2..=14).contains(&rank) => Ok(rank),
        _ => Err(malformed_output(format_args!(
            "a card rank outside 2..=14: {value}"
        ))),
    }
}

/// Reads a trick count, rejecting values outside a deal's `0..=13`.
fn trick_count(value: c_int) -> Result<u8, SolverError> {
    match u8::try_from(value) {
        Ok(count) if count <= 13 => Ok(count),
        _ => Err(malformed_output(format_args!(
            "a trick count outside 0..=13: {value}"
        ))),
    }
}

/// Turns DDS's `SolvedPlay` into the seam's one-entry-per-play vector.
///
/// DDS reports the starting position then the result after each card, but caps
/// the sequence at 49 entries ("the last trick holds no excitement"). Every
/// omitted play is in the forced final trick — once 48 cards are down, each
/// hand holds exactly one card — so repeating the last count restores the
/// one-entry-per-play convention.
///
/// A length past `plays + 1` is rejected rather than truncated: padding one
/// entry per play is the only repair the contract allows.
fn read_play_results(solved: &SolvedPlay, plays: usize) -> Result<Vec<u8>, SolverError> {
    let number = usize::try_from(solved.number).map_err(|_| {
        malformed_output(format_args!(
            "a negative play-analysis length: {}",
            solved.number
        ))
    })?;
    let results = solved.tricks.get(..number).ok_or_else(|| {
        malformed_output(format_args!(
            "{number} play-analysis results for a {} slot buffer",
            solved.tricks.len()
        ))
    })?;
    if number > plays + 1 {
        return Err(malformed_output(format_args!(
            "{number} play-analysis results for {plays} recorded plays"
        )));
    }
    let mut tricks = Vec::with_capacity(plays + 1);
    for &trick in results {
        tricks.push(trick_count(trick)?);
    }
    let last = tricks
        .last()
        .copied()
        .ok_or_else(|| malformed_output(format_args!("no play-analysis results")))?;
    tricks.resize(plays + 1, last);
    Ok(tricks)
}

/// Expands DDS's `equals` bitmask (bit *n* = rank *n*) into physical cards of
/// `card`'s suit, including `card` itself.
fn expand_equals(card: Card, equals: c_int) -> Result<Cards, SolverError> {
    let suit = card.suit();
    let bits = u64::try_from(equals)
        .map_err(|_| malformed_output(format_args!("a negative equivalence mask: {equals}")))?;
    let mut cards = Cards::EMPTY.insert(card);
    for rank in 2..=14u8 {
        if bits & (1u64 << rank) != 0 {
            cards = cards.insert(Card::new(suit, rank));
        }
    }
    Ok(cards)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_fixtures::{
        example_board, example_board_hand0_play, example_board_with, example_deal, opening_position,
    };
    use crate::{ContextFactory, DdTableCalculator, ParCalculator};
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
    fn par_from_the_example_hand_dd_table_matches_the_dds_oracle() {
        // DDS's `examples/hands.cpp` hand 0 publishes its DD table and par
        // together. Using the published table isolates par from table
        // calculation: the seam must derive the same contract from known
        // counts, not merely agree with a table it computed itself.
        let table = DdTable::empty()
            .with(Strain::Spades, Seat::North, 5)
            .with(Strain::Spades, Seat::East, 8)
            .with(Strain::Spades, Seat::South, 5)
            .with(Strain::Spades, Seat::West, 8)
            .with(Strain::Hearts, Seat::North, 6)
            .with(Strain::Hearts, Seat::East, 6)
            .with(Strain::Hearts, Seat::South, 6)
            .with(Strain::Hearts, Seat::West, 6)
            .with(Strain::Diamonds, Seat::North, 5)
            .with(Strain::Diamonds, Seat::East, 7)
            .with(Strain::Diamonds, Seat::South, 5)
            .with(Strain::Diamonds, Seat::West, 7)
            .with(Strain::Clubs, Seat::North, 7)
            .with(Strain::Clubs, Seat::East, 5)
            .with(Strain::Clubs, Seat::South, 7)
            .with(Strain::Clubs, Seat::West, 5)
            .with(Strain::NoTrumps, Seat::North, 6)
            .with(Strain::NoTrumps, Seat::East, 6)
            .with(Strain::NoTrumps, Seat::South, 6)
            .with(Strain::NoTrumps, Seat::West, 6);

        let par = DdsSolver
            .par(&table, Vulnerability::None)
            .expect("example hand 0 has a par result");

        // The oracle is DDS's own `par_scores_`/`par_strings_` for hand 0.
        assert_eq!(par.ns.score, -110);
        assert_eq!(par.ns.contracts, "EW 2S");
        assert_eq!(par.ew.score, 110);
        assert_eq!(par.ew.contracts, "EW 2S");
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
    fn read_results_rejects_a_malformed_card_instead_of_skipping_it() {
        // A silent skip used to turn a broken DDS contract into a plausible
        // result with one card missing. Suit 4 (no-trump) is not a card suit.
        let mut future = FutureTricks {
            cards: 1,
            ..Default::default()
        };
        future.suit[0] = 4;
        future.rank[0] = 5;
        future.score[0] = 5;

        let error = read_results(&future).expect_err("suit 4 is not a card suit");

        assert!(matches!(error, SolverError::Backend(_)));
        assert!(error.to_string().contains("suit"), "{error}");
    }

    #[test]
    fn read_results_rejects_a_card_count_past_the_buffer() {
        // `cards` is DDS output: more results than the fixed buffer is a broken
        // contract, not a reason to silently clamp the list.
        let mut future = FutureTricks {
            cards: 14,
            ..Default::default()
        };
        future.suit[0] = 0;
        future.rank[0] = 5;
        future.score[0] = 5;

        let error = read_results(&future).expect_err("14 results do not fit 13 slots");

        assert!(matches!(error, SolverError::Backend(_)));
        assert!(error.to_string().contains("count"), "{error}");
    }

    #[test]
    fn read_play_results_rejects_a_negative_length() {
        // `number` is DDS output; a negative length is a broken contract.
        let mut solved = SolvedPlay::new();
        solved.number = -1;

        let error = read_play_results(&solved, 0).expect_err("a negative length is malformed");

        assert!(matches!(error, SolverError::Backend(_)));
        assert!(error.to_string().contains("length"), "{error}");
    }

    #[test]
    fn read_play_results_rejects_a_trick_count_outside_the_trick_range() {
        // The play analyser shares `read_results`' policy: a count of 14 is not
        // a trick count, so it is an error rather than a plausible wrong value.
        let mut solved = SolvedPlay::new();
        solved.number = 1;
        solved.tricks[0] = 14;

        let error = read_play_results(&solved, 0).expect_err("14 tricks is not a count");

        assert!(matches!(error, SolverError::Backend(_)));
        assert!(error.to_string().contains("trick count"), "{error}");
    }

    #[test]
    fn read_play_results_rejects_more_results_than_the_recorded_plays() {
        // One entry per play plus the starting position is the contract; more
        // entries than that means DDS reported a length the caller cannot use.
        // Truncating them would silently absorb a broken length.
        let mut solved = SolvedPlay::new();
        solved.number = 3;
        solved.tricks[..3].copy_from_slice(&[5, 5, 5]);

        let error = read_play_results(&solved, 1).expect_err("3 results for 1 play is malformed");

        assert!(matches!(error, SolverError::Backend(_)));
        assert!(error.to_string().contains("play"), "{error}");
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
