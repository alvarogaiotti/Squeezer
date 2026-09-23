// Copyright (C) 2025 Alvaro Gaiotti
// See end of file for license information

//! A dedicated worker pool that hands each worker one reused solver context.

use super::Solver;
use rayon::prelude::*;

/// A fixed-size thread pool paired with a [`Solver`] factory.
///
/// The pool owns its own [`rayon::ThreadPool`] rather than borrowing the
/// process-global one, so the caller — not some unrelated library — decides
/// how much solver memory is in flight.
///
/// # Context lifetime, precisely
///
/// Work is issued through [`SolverPool::map`] and [`SolverPool::try_map`],
/// which use rayon's `map_init`: the context initializer runs once per rayon
/// *job* (a contiguous chunk of indices) and that context is reused for every
/// index in the chunk. It is **not** strictly one context per worker: a worker
/// that steals a second chunk runs the initializer again, so a run may create
/// more contexts than there are workers, and a context need not serve every
/// index its worker handles. What *is* guaranteed is that at most one context
/// is live per worker at any instant, so `workers × table capacity` bounds
/// *instantaneous* memory — which is the reason the pool exists.
///
/// `with_min_len` caps rayon's split count so the number of contexts stays
/// close to the worker count; the design's "one context per worker, reused
/// across the run" is therefore an approximation, not a guarantee.
pub struct SolverPool<S: Solver> {
    solver: S,
    pool: rayon::ThreadPool,
}

impl<S: Solver> SolverPool<S> {
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
    pub fn new(solver: S, workers: usize) -> Self {
        let workers = workers.max(1);
        let pool = rayon::ThreadPoolBuilder::new()
            .num_threads(workers)
            .build()
            .expect("a positive thread count always builds a rayon pool");
        Self { solver, pool }
    }

    /// Builds a pool sized to the machine's available parallelism.
    #[must_use]
    pub fn with_available_workers(solver: S) -> Self {
        let workers = std::thread::available_parallelism().map_or(1, std::num::NonZeroUsize::get);
        Self::new(solver, workers)
    }

    /// The number of worker threads in the pool.
    #[must_use]
    pub fn workers(&self) -> usize {
        self.pool.current_num_threads()
    }

    /// The solver factory backing the pool.
    #[must_use]
    pub fn solver(&self) -> &S {
        &self.solver
    }
}

impl<S: Solver + Send + Sync> SolverPool<S> {
    /// Applies `f` to every index in `0..len`, in parallel.
    ///
    /// Rayon creates one context per job (chunk) and reuses it for the indices
    /// in that chunk; see [`SolverPool`] for why this is not exactly one per
    /// worker. The result is in index order, so callers can zip it back
    /// against their inputs.
    #[must_use]
    pub fn map<T, F>(&self, len: usize, f: F) -> Vec<T>
    where
        F: Fn(&mut S::Context, usize) -> T + Send + Sync,
        T: Send,
    {
        if len == 0 {
            return Vec::new();
        }
        // Cap the number of splits so the number of contexts stays bounded by
        // the worker count, rather than rayon's opportunistic over-splitting.
        let workers = self.pool.current_num_threads().max(1);
        let min_chunk = len.div_ceil(workers).max(1);
        let solver = &self.solver;
        self.pool.install(|| {
            (0..len)
                .into_par_iter()
                .with_min_len(min_chunk)
                .map_init(|| solver.new_context(), &f)
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
        F: Fn(&mut S::Context, usize) -> Result<T, E> + Send + Sync,
        T: Send,
        E: Send,
    {
        self.map(len, f).into_iter().collect()
    }
}
