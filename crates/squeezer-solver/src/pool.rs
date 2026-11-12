// Copyright (C) 2025 Alvaro Gaiotti
// See end of file for license information

//! The worker pool that pairs solver jobs with reusable solver contexts.
//!
//! [`SolverPool`] owns a dedicated, fixed-size [`rayon::ThreadPool`], so the
//! caller — not some unrelated library using the process-global pool — decides
//! how much transposition-table memory is in flight. It holds a boxed
//! [`ContextFactory`] rather than a concrete solver: the object-safe factory is
//! what simulations keep anyway (ADR-0005), one implementation serves both
//! static and runtime-selected backends, and the indirect call per job and per
//! solve is noise against a double-dummy solve.
//!
//! # Context lifetime: one per job, not one per worker
//!
//! [`SolverPool::map`] slices `0..len` into at most one chunk per worker and
//! runs the chunks in parallel. Each chunk creates exactly one context and
//! reuses it for every index in that chunk; the context is dropped when the
//! chunk finishes. A worker that handles a second chunk creates a second
//! context sequentially, so one thread can create more than one context over a
//! whole call, but **at most one context is live per worker at any instant**.
//!
//! This is a per-job context lifetime, not a thread-local worker table.
//! Creating and destroying the context inside the chunk closure keeps a
//! context on the single worker that uses it, and returns its memory as soon as
//! the chunk ends instead of holding it until the pool drops. The bound that
//! matters is unchanged: `workers × table capacity` limits instantaneous
//! memory, so a caller choosing a worker count chooses the memory ceiling.
//!
//! The chunk count is fixed by this module (`ceil(len / ceil(len / workers))`,
//! never more than the worker count) rather than left to rayon's adaptive
//! splitter, so the bound is a property of the pool and not of the scheduler's
//! current splitting strategy. The pool tests measure all of it: a 64-item call
//! on two workers creates two contexts and each handles 32 solves; no more than
//! `workers` contexts are live at once; and every context drops when `map`
//! returns.
//!
//! # Transposition-table reuse
//!
//! Reuse only pays off within a chunk, and only while the deal and strain stay
//! the same: a trump change resets the backend's table. Callers therefore issue
//! solves deal-major with same-strain contracts adjacent, which the contract
//! payoff matrix already does (ADR-0005).

use super::{ContextFactory, SolverContext};
use rayon::prelude::*;

/// A fixed-size thread pool paired with a boxed [`ContextFactory`].
///
/// The pool slices a call into at most one chunk per worker, gives each chunk
/// one context, and drops every context when its chunk ends; `workers × table
/// capacity` therefore bounds instantaneous memory.
pub struct SolverPool {
    factory: Box<dyn ContextFactory>,
    pool: rayon::ThreadPool,
}

impl SolverPool {
    /// Builds a pool with `workers` worker threads.
    ///
    /// `workers` is clamped to at least one; a zero would otherwise let rayon
    /// pick the global default and silently break the memory bound.
    ///
    /// # Panics
    ///
    /// Panics only if rayon rejects the always-valid positive thread count,
    /// which it does not in practice.
    #[must_use]
    pub fn new(factory: Box<dyn ContextFactory>, workers: usize) -> Self {
        let workers = workers.max(1);
        let pool = rayon::ThreadPoolBuilder::new()
            .num_threads(workers)
            .build()
            .expect("a positive thread count always builds a rayon pool");
        Self { factory, pool }
    }

    /// Builds a pool sized to the machine's available parallelism.
    #[must_use]
    pub fn with_available_workers(factory: Box<dyn ContextFactory>) -> Self {
        let workers = std::thread::available_parallelism().map_or(1, std::num::NonZeroUsize::get);
        Self::new(factory, workers)
    }

    /// The number of worker threads in the pool.
    #[must_use]
    pub fn workers(&self) -> usize {
        self.pool.current_num_threads()
    }

    /// Applies `f` to every index in `0..len`, in parallel.
    ///
    /// The result is in index order, so callers can zip it back against their
    /// inputs. Each chunk of indices gets one context, reused for the whole
    /// chunk; see [`SolverPool`] for why that is not exactly one context per
    /// worker.
    #[must_use]
    pub fn map<T, F>(&self, len: usize, f: F) -> Vec<T>
    where
        F: Fn(&mut dyn SolverContext, usize) -> T + Send + Sync,
        T: Send,
    {
        if len == 0 {
            return Vec::new();
        }
        let workers = self.workers().max(1);
        // Slicing here, rather than asking rayon for a minimum length, makes
        // the context count deterministic: one context per chunk, and no more
        // chunks than workers. `chunk * chunk_len` cannot overflow because
        // `chunk < chunks = ceil(len / chunk_len)`, so the product is `< len`.
        let chunk_len = len.div_ceil(workers).max(1);
        let chunks = len.div_ceil(chunk_len);
        let factory = &self.factory;
        let f = &f;
        self.pool.install(|| {
            (0..chunks)
                .into_par_iter()
                .flat_map_iter(|chunk| {
                    let start = chunk * chunk_len;
                    let end = start + (len - start).min(chunk_len);
                    let mut context = factory.new_context();
                    (start..end).map(move |index| f(context.as_mut(), index))
                })
                .collect()
        })
    }

    /// Fallible [`SolverPool::map`]: the first error (in index order) wins.
    ///
    /// # Errors
    ///
    /// Returns the error produced by `f` for the lowest failing index.
    pub fn try_map<T, E, F>(&self, len: usize, f: F) -> Result<Vec<T>, E>
    where
        F: Fn(&mut dyn SolverContext, usize) -> Result<T, E> + Send + Sync,
        T: Send,
        E: Send,
    {
        self.map(len, f).into_iter().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::prelude::*;
    use crate::{DdTableCalculator, Solutions, SolveMode, SolveResults, SolverError};
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::{Arc, Mutex};

    /// The context ledger a tracking factory keeps for one test.
    #[derive(Default)]
    struct PoolStats {
        created: AtomicUsize,
        live: AtomicUsize,
        max_live: AtomicUsize,
        /// Solves each context handled, appended when the context drops.
        solve_counts: Mutex<Vec<usize>>,
    }

    impl PoolStats {
        fn solve_counts(&self) -> Vec<usize> {
            self.solve_counts
                .lock()
                .expect("the test mutex is never poisoned")
                .clone()
        }
    }

    /// A factory whose contexts record their own lifetime and work.
    struct TrackingFactory(Arc<PoolStats>);

    struct TrackingContext {
        solves: usize,
        stats: Arc<PoolStats>,
    }

    impl ContextFactory for TrackingFactory {
        fn new_context(&self) -> Box<dyn SolverContext> {
            self.0.created.fetch_add(1, Ordering::Relaxed);
            let live = self.0.live.fetch_add(1, Ordering::Relaxed) + 1;
            self.0.max_live.fetch_max(live, Ordering::Relaxed);
            Box::new(TrackingContext {
                solves: 0,
                stats: Arc::clone(&self.0),
            })
        }
    }

    impl DdTableCalculator for TrackingFactory {}

    impl SolverContext for TrackingContext {
        fn solve(
            &mut self,
            _position: &Position,
            _mode: SolveMode,
            _solutions: Solutions,
        ) -> Result<SolveResults, SolverError> {
            self.solves += 1;
            Ok(SolveResults::default())
        }

        fn clear_tt(&mut self) {}

        fn reset(&mut self) {}
    }

    impl Drop for TrackingContext {
        fn drop(&mut self) {
            self.stats
                .solve_counts
                .lock()
                .expect("the test mutex is never poisoned")
                .push(self.solves);
            self.stats.live.fetch_sub(1, Ordering::Relaxed);
        }
    }

    /// A pool whose contexts report creation, concurrent lifetime and solves.
    fn tracking_pool(workers: usize) -> (SolverPool, Arc<PoolStats>) {
        let stats = Arc::new(PoolStats::default());
        let pool = SolverPool::new(Box::new(TrackingFactory(Arc::clone(&stats))), workers);
        (pool, stats)
    }

    #[test]
    fn one_context_serves_a_whole_chunk() {
        let (pool, stats) = tracking_pool(2);
        let position = crate::test_fixtures::opening_position();

        let seen: Vec<usize> = pool.map(64, |context, index| {
            context
                .solve(&position, SolveMode::AutoReuse, Solutions::OneDdOptimal)
                .expect("the tracking context never fails");
            index
        });

        assert_eq!(seen, (0..64).collect::<Vec<_>>());
        // 64 indices over 2 workers is two 32-index chunks. One context serves
        // each chunk, so each context handles 32 of the 64 solves.
        assert_eq!(stats.created.load(Ordering::Relaxed), 2);
        let mut counts = stats.solve_counts();
        counts.sort_unstable();
        assert_eq!(counts, vec![32, 32]);
    }

    #[test]
    fn at_most_one_context_per_worker_is_live_and_all_are_dropped() {
        // Sizes below, at and well past the worker count. The pool's memory
        // story is "workers contexts, each with its own transposition table",
        // so this is the invariant a backend with a table capacity relies on.
        for (len, workers) in [
            (1, 4),
            (3, 4),
            (9, 4),
            (10, 4),
            (16, 4),
            (64, 4),
            (205, 4),
            (1000, 8),
        ] {
            let (pool, stats) = tracking_pool(workers);
            let seen: Vec<usize> = pool.map(len, |_context, index| index);

            assert_eq!(
                seen,
                (0..len).collect::<Vec<_>>(),
                "results must stay in index order for len {len}"
            );
            let created = stats.created.load(Ordering::Relaxed);
            assert!(
                created <= workers,
                "len {len} with {workers} workers created {created} contexts"
            );
            assert!(created >= 1, "a non-empty map creates at least one context");

            let max_live = stats.max_live.load(Ordering::Relaxed);
            assert!(
                max_live <= workers,
                "len {len} with {workers} workers had {max_live} live contexts at once"
            );
            assert_eq!(
                stats.live.load(Ordering::Relaxed),
                0,
                "each context drops when its chunk finishes"
            );
        }
    }

    #[test]
    fn an_empty_map_creates_no_context() {
        let (pool, stats) = tracking_pool(4);

        assert!(pool.map(0, |_context, index| index).is_empty());
        assert_eq!(stats.created.load(Ordering::Relaxed), 0);
        assert_eq!(stats.live.load(Ordering::Relaxed), 0);
    }

    #[test]
    fn zero_workers_are_clamped_to_one() {
        let (pool, stats) = tracking_pool(0);

        assert_eq!(pool.workers(), 1);
        assert_eq!(
            pool.map(8, |_context, index| index),
            (0..8).collect::<Vec<_>>()
        );
        assert_eq!(
            stats.created.load(Ordering::Relaxed),
            1,
            "one worker means one chunk and so one context"
        );
        assert_eq!(stats.live.load(Ordering::Relaxed), 0);
    }

    #[test]
    fn try_map_propagates_the_first_error() {
        let (pool, _stats) = tracking_pool(2);
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
