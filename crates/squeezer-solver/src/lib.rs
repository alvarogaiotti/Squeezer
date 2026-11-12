// Copyright (C) 2025 Alvaro Gaiotti
// See end of file for license information

#![warn(clippy::pedantic)]
#![allow(
    dead_code,
    clippy::module_name_repetitions,
    clippy::integer_division_remainder_used,
    clippy::missing_docs_in_private_items,
    clippy::pub_with_shorthand,
    clippy::question_mark_used,
    clippy::as_conversions,
    clippy::indexing_slicing,
    clippy::missing_trait_methods,
    clippy::single_call_fn,
    clippy::unseparated_literal_suffix,
    clippy::implicit_return,
    clippy::should_panic_without_expect,
    clippy::needless_doctest_main
)]

//! The backend-agnostic double-dummy solver seam, its worker pool, and the
//! DDS adapters and backend behind the `dds` feature.
//!
//! A solver is a *factory* ([`Solver`]) whose product is a stateful, single
//! threaded [`SolverContext`] owning a transposition table (TT). Callers that
//! want runtime backend selection — or a mock in tests — box a
//! [`ContextFactory`], the object-safe view of the factory; callers on the hot
//! path may keep the concrete associated type and pay no indirection.
//!
//! # Why a context, not a solver
//!
//! DDS keeps the TT on the solver context. Reusing it across *similar*
//! positions (same deal, different strain or leader) is where double-dummy
//! speed comes from, and the TT cannot be shared between threads. The seam
//! therefore models "one context, one thread": a context is created once,
//! reused across many solves, and torn down with its worker.
//!
//! # What the seam hides
//!
//! Everything backend-specific. [`SolveMode`] and [`Solutions`] re-express the
//! two knobs a caller needs — how eagerly to reuse the TT, and how many cards
//! to return — without naming DDS's integer codes. [`SolveResults`] and
//! [`CardSolve`] carry a card's DD tricks and its equivalence class, which is
//! all card-EV and lead analysis consume. The DDS types themselves never cross
//! this boundary.
//!
//! # Cold capabilities
//!
//! Table, par and play analysis are rare, per-hand operations, not per-solve
//! ones, so they are separate traits ([`DdTableCalculator`], [`ParCalculator`],
//! [`PlayAnalyser`]) rather than methods on [`SolverContext`]. Play analysis in
//! particular cannot live on a context: DDS's `AnalysePlay` builds a private
//! context of its own, so the operation belongs to the backend.
//!
//! # Where DDS is allowed
//!
//! This crate owns every use of the native library besides the `dds` FFI
//! package's own, behind the `dds` feature. The card and model crates never see
//! a DDS type: the adapters in the `adapters` module translate their records
//! through local newtypes and free functions. While the root package still
//! exists it also names `dds` to keep its historical `squeezer::dds` path, but
//! it forwards all solver work here; the split's target is that only this crate
//! and the FFI package name DDS.

#[cfg(feature = "dds")]
mod adapters;
#[cfg(feature = "dds")]
mod dds;
mod pool;

/// The [`Position`] to raw-DDS-deal adapter, published so callers and
/// benchmarks can measure the conversion without going through a solve.
#[cfg(feature = "dds")]
pub use adapters::position_to_dds_deal;
#[cfg(feature = "dds")]
pub use dds::{DdsContext, DdsSolver};
pub use pool::{DynSolverPool, SolverPool};

use crate::prelude::*;
use std::fmt;

/// The card and model vocabulary the solver modules share, mirroring the root
/// crate's historical prelude so the moved code reads the same.
///
/// This is crate-internal: consumers import from [`squeezer_cards`] and
/// [`squeezer_model`] directly.
mod prelude {
    pub(crate) use squeezer_cards::*;
    pub(crate) use squeezer_model::*;
}

/// How eagerly the backend should reuse its transposition table.
///
/// The transposition table is only valid for *similar* positions, so reuse is
/// a promise the caller makes, not one the backend can verify.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SolveMode {
    /// Let the backend decide when the table must be reset, and always search
    /// a card's score rather than returning a "forced" sentinel.
    #[default]
    AutoReuse,
    /// Force table reuse and always search, even across a change the backend
    /// would normally treat as a reason to reset. The caller guarantees the
    /// positions are similar enough for the table to be correct.
    ForceReuseAlwaysSearch,
}

/// How many of the legal cards a solve should return.
///
/// "Optimal" here is always double-dummy-relative (see ADR-0003 and
/// `CONTEXT.md`); the variant names carry the `Dd` qualifier so a caller can
/// never mistake the result for a real-world best play.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Solutions {
    /// One card: a double-dummy-optimal play.
    #[default]
    OneDdOptimal,
    /// Every card that shares the double-dummy-optimal result.
    AllDdOptimal,
    /// Every legal card, each with its own result.
    AllLegal,
}

/// One legal card's double-dummy outcome.
///
/// `equivalents` is the full equivalence class of `card` — the cards the
/// backend considers interchangeable for DD purposes, including `card` itself.
/// A consumer that wants to know "which physical cards achieve this result"
/// reads the class rather than recomputing it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CardSolve {
    /// The card this outcome is for.
    pub card: Card,
    /// Tricks for the side to play after `card` is played.
    pub tricks: u8,
    /// Every card equivalent to `card` for double-dummy purposes.
    pub equivalents: Cards,
}

/// The backend-agnostic result of one solve.
///
/// It holds whatever the requested [`Solutions`] level produced: one card for
/// [`Solutions::OneDdOptimal`], the DD-optimal class for
/// [`Solutions::AllDdOptimal`], or every legal card for
/// [`Solutions::AllLegal`].
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SolveResults {
    /// The solved cards, in the backend's order.
    pub cards: Vec<CardSolve>,
}

impl SolveResults {
    /// The highest double-dummy trick count among the returned cards, if any.
    #[must_use]
    pub fn max_dd_tricks(&self) -> Option<u8> {
        self.cards.iter().map(|card| card.tricks).max()
    }
}

/// Why a solve or a context operation failed.
///
/// Deliberately free of backend types so the seam carries no `dds` feature.
/// The root crate wraps it in its own error enum at the boundary where a
/// caller needs one error type.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SolverError {
    /// The backend could not allocate a solver context.
    ContextCreation,
    /// The backend rejected the position or failed to solve it.
    Backend(String),
    /// The position could not be translated into the backend's input format.
    Position(String),
    /// A solve succeeded but returned no card for a position that still has
    /// cards to play. This is a broken invariant, never a normal outcome.
    NoCards,
}

impl fmt::Display for SolverError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ContextCreation => write!(f, "the solver backend could not create a context"),
            Self::Backend(message) => write!(f, "the solver backend failed: {message}"),
            Self::Position(message) => write!(f, "the position is not solvable: {message}"),
            Self::NoCards => {
                write!(f, "the solver returned no card for a non-terminal position")
            }
        }
    }
}

impl std::error::Error for SolverError {}

/// A stateful, single-threaded solver handle owning its transposition table.
///
/// One context is used by one thread at a time. Create it once, issue many
/// solves on it, and drop it with its worker; [`SolverPool`] does exactly that
/// on the caller's behalf.
pub trait SolverContext: Send {
    /// Solves `position`, returning the cards selected by `solutions`.
    ///
    /// This is the only required solve method: the convenience methods below
    /// build on it, so a backend implements this call plus [`SolverContext::clear_tt`]
    /// and [`SolverContext::reset`], and nothing else.
    ///
    /// # Errors
    ///
    /// Returns [`SolverError`] if the context is unusable, the position cannot
    /// be translated, or the backend rejects it.
    fn solve(
        &mut self,
        position: &Position,
        mode: SolveMode,
        solutions: Solutions,
    ) -> Result<SolveResults, SolverError>;

    /// The maximum double-dummy tricks available to the side to play.
    ///
    /// # Errors
    ///
    /// See [`SolverContext::solve`].
    fn solve_tricks(&mut self, position: &Position) -> Result<u8, SolverError> {
        self.solve(position, SolveMode::AutoReuse, Solutions::OneDdOptimal)?
            .max_dd_tricks()
            .ok_or(SolverError::NoCards)
    }

    /// Every legal play in `position`, each with its DD tricks and equivalence
    /// class.
    ///
    /// # Errors
    ///
    /// See [`SolverContext::solve`].
    fn solve_all_cards(&mut self, position: &Position) -> Result<SolveResults, SolverError> {
        self.solve(position, SolveMode::AutoReuse, Solutions::AllLegal)
    }

    /// Clears the transposition table, keeping the context alive.
    fn clear_tt(&mut self);

    /// Resets per-solve state while keeping the transposition table.
    fn reset(&mut self);
}

/// A factory that hands out fresh [`SolverContext`]s.
///
/// Object-safe, so a simulation can take a `Box<dyn ContextFactory>` and stay
/// free of any solver type parameter. The blanket implementation below makes
/// every [`Solver`] a `ContextFactory`.
pub trait ContextFactory: Send + Sync {
    /// Creates a context with an empty transposition table.
    fn new_context(&self) -> Box<dyn SolverContext>;
}

impl<S> ContextFactory for S
where
    S: Solver + Send + Sync,
{
    fn new_context(&self) -> Box<dyn SolverContext> {
        Box::new(Solver::new_context(self))
    }
}

/// The static factory for a backend.
///
/// The associated type keeps this trait non-object-safe on purpose; use
/// [`ContextFactory`] when runtime dispatch is wanted.
pub trait Solver {
    /// The stateful context this solver hands out.
    type Context: SolverContext + 'static;

    /// Creates a context with an empty transposition table.
    fn new_context(&self) -> Self::Context;
}

/// A cold capability: computing a full double-dummy table for a deal.
pub trait DdTableCalculator {
    /// The backend's DD-table representation.
    type Table;
    /// The backend's error type.
    type Error;

    /// Computes the DD table for `deal`.
    ///
    /// # Errors
    ///
    /// Returns the backend's error when the deal is rejected.
    fn dd_table(&self, deal: &Deal) -> Result<Self::Table, Self::Error>;
}

/// A cold capability: deriving par from a precomputed DD table.
pub trait ParCalculator {
    /// The DD-table representation consumed as input.
    type Table;
    /// The backend's par representation.
    type Par;
    /// The backend's error type.
    type Error;

    /// Computes the par contract and score for `vulnerability`.
    ///
    /// # Errors
    ///
    /// Returns the backend's error when the table is rejected.
    fn par(
        &self,
        table: &Self::Table,
        vulnerability: Vulnerability,
    ) -> Result<Self::Par, Self::Error>;
}

/// A cold capability: analysing a played hand against double-dummy.
///
/// This lives on the backend rather than on [`SolverContext`] because DDS's
/// play analysis builds its own context internally.
pub trait PlayAnalyser {
    /// The backend's error type.
    type Error;

    /// Returns the double-dummy trick count after each recorded play.
    ///
    /// The vector has one entry per play plus the starting position, so a
    /// board with `n` recorded plays yields `n + 1` entries. A backend may
    /// omit the forced final trick internally (DDS caps its sequence at 49);
    /// it is responsible for restoring the full length.
    ///
    /// The result is a raw `Vec<u8>` of trick counts in `0..=13`, not a typed
    /// wrapper: the core seam must not name `performance_analysis::Tricks`
    /// (ADR-0005 keeps the solver module independent of the analysis module),
    /// so the count contract is documented here and on
    /// `performance_analysis::AnalysisInput` instead.
    ///
    /// # Errors
    ///
    /// Returns the backend's error when the board is rejected.
    fn analyse_play(&self, board: &Board) -> Result<Vec<u8>, Self::Error>;
}

/// Shared test fixtures: DDS's `examples/hands.cpp` hand 0, used by both the
/// backend-agnostic mock tests and the DDS seam tests.
///
/// Gated so a normal build never publishes test data; the root crate enables
/// the `test-support` feature from its dev-dependencies.
#[cfg(any(test, feature = "test-support"))]
pub mod test_fixtures;

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    /// Assigns each created context a distinct id so the tests can count and
    /// reuse them without a real backend.
    static NEXT_ID: AtomicUsize = AtomicUsize::new(0);

    #[derive(Debug, Default)]
    struct MockSolver;

    struct MockContext {
        id: usize,
    }

    impl Solver for MockSolver {
        type Context = MockContext;

        fn new_context(&self) -> Self::Context {
            MockContext {
                id: NEXT_ID.fetch_add(1, Ordering::Relaxed),
            }
        }
    }

    impl SolverContext for MockContext {
        fn solve(
            &mut self,
            position: &Position,
            _mode: SolveMode,
            solutions: Solutions,
        ) -> Result<SolveResults, SolverError> {
            // A trivially "solved" position: one card per legal play, trick
            // count equal to the number of legal plays, each its own class.
            let tricks = position.legal_plays().len();
            let cards: Vec<Card> = if matches!(solutions, Solutions::OneDdOptimal) {
                position.legal_plays().into_iter().take(1).collect()
            } else {
                position.legal_plays().into_iter().collect()
            };
            Ok(SolveResults {
                cards: cards
                    .into_iter()
                    .map(|card| CardSolve {
                        card,
                        tricks,
                        equivalents: Cards::EMPTY.insert(card),
                    })
                    .collect(),
            })
        }

        fn clear_tt(&mut self) {}

        fn reset(&mut self) {}
    }

    #[test]
    fn a_boxed_factory_drives_a_trivial_simulation() {
        let factory: Box<dyn ContextFactory> = Box::new(MockSolver);
        let mut context = factory.new_context();
        let position = test_fixtures::opening_position();

        // North is on lead with all 13 cards legal, so the mock reports 13.
        assert_eq!(
            context.solve_tricks(&position).expect("mock never fails"),
            13
        );
        let all = context
            .solve_all_cards(&position)
            .expect("mock never fails");
        assert_eq!(all.cards.len(), 13);
    }

    #[test]
    fn the_pool_reuses_a_context_within_each_job() {
        let pool = SolverPool::new(MockSolver, 2);
        let position = test_fixtures::opening_position();

        let seen: Vec<usize> = pool.map(64, |context, _index| {
            let _ = context.solve_tricks(&position);
            context.id
        });

        assert_eq!(seen.len(), 64);
        let distinct: std::collections::HashSet<_> = seen.iter().copied().collect();
        // One context per rayon job. `with_min_len` keeps the job count near
        // the 2-worker count, but a stolen chunk could add one more, so we do
        // not assert exactly 2 — only that reuse happens and the count is
        // small.
        assert!(
            distinct.len() <= 3,
            "expected a small per-job context count, saw {distinct:?}"
        );
        assert!(
            seen.len() > distinct.len(),
            "contexts should be reused across items"
        );
    }

    #[test]
    fn try_map_propagates_the_first_error() {
        let pool = SolverPool::new(MockSolver, 2);
        let result: Result<Vec<usize>, &'static str> =
            pool.try_map(
                8,
                |_context, index| {
                    if index == 3 { Err("boom") } else { Ok(index) }
                },
            );
        assert_eq!(result, Err("boom"));
    }

    #[test]
    fn the_dyn_pool_reuses_a_context_within_each_job() {
        use std::sync::Arc;

        /// Counts only the contexts this factory creates, so the assertion is
        /// unaffected by other tests creating `MockContext`s in parallel.
        struct CountingFactory {
            created: Arc<AtomicUsize>,
        }

        impl ContextFactory for CountingFactory {
            fn new_context(&self) -> Box<dyn SolverContext> {
                self.created.fetch_add(1, Ordering::Relaxed);
                Box::new(MockContext {
                    id: NEXT_ID.fetch_add(1, Ordering::Relaxed),
                })
            }
        }

        let created = Arc::new(AtomicUsize::new(0));
        let pool = DynSolverPool::new(
            Box::new(CountingFactory {
                created: Arc::clone(&created),
            }),
            2,
        );
        let position = test_fixtures::opening_position();

        let seen = pool.map(64, |context, index| {
            let _ = context.solve_tricks(&position);
            index
        });

        assert_eq!(seen.len(), 64);
        let created = created.load(Ordering::Relaxed);
        // One context per rayon job; a stolen chunk can add one more, so we
        // assert a small count rather than exactly two.
        assert!(
            created <= 3,
            "expected a small per-job context count, created {created}"
        );
        assert!(created < 64, "contexts should be reused across items");
    }

    #[test]
    fn the_dyn_pool_try_map_propagates_the_first_error() {
        let factory: Box<dyn ContextFactory> = Box::new(MockSolver);
        let pool = DynSolverPool::new(factory, 2);
        let result: Result<Vec<usize>, &'static str> =
            pool.try_map(
                8,
                |_context, index| {
                    if index == 3 { Err("boom") } else { Ok(index) }
                },
            );
        assert_eq!(result, Err("boom"));
    }
}
