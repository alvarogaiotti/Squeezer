# Solver seam: static factory, object-safe context, DDS behind a feature

The solver interface lives in `squeezer-solver` as a `Solver` factory with an
associated `Context` type, an object-safe `SolverContext` hot interface, and an
object-safe `ContextFactory` (blanket-implemented for every `Solver`) so callers
may choose static dispatch or a boxed backend. Our own simulations take a
`Box<dyn ContextFactory>`, which keeps a solver type parameter out of the public
simulation API and makes mocks and runtime backend selection possible; the
dispatch cost is one indirect call per board against a multi-microsecond solve.
The factory also carries the cold `DdTableCalculator` capability, so a caller
holding only the boxed factory can request a DD table; a backend without table
support keeps that trait's default error.
The DDS implementation lives in `squeezer-solver` behind the `dds` feature,
using the `dds` crate only as raw FFI, so the rest of the workspace still builds
with no native dependency (ADR-0004). A `SolverPool` owns a dedicated sized
rayon pool because worker count bounds transposition-table memory
(`workers × cap`).

## The pool: one implementation, one context per job

`SolverPool` holds a `Box<dyn ContextFactory>` rather than a concrete `Solver`.
This replaces both the former generic `SolverPool<S>` and the separate
`DynSolverPool`: the object-safe factory is the form simulations already keep,
so there is one pool implementation instead of a static and a boxed sibling
duplicating the thread-count clamp and the chunking; the boxed path costs one
indirect call per job and one per solve, which is noise against a double-dummy
solve. The transposition-table (`TT`) reuse that matters happens inside a chunk,
not across the dispatch boundary.

Work is sliced into at most one chunk per worker. Each chunk creates exactly one
context, reuses it for every index in the chunk, and drops it when the chunk
finishes. This is a per-job context lifetime, not a thread-local worker table,
for two reasons: a context is created and destroyed on the single worker that
uses it, and its memory returns when the chunk ends rather than being held until
the pool drops. The bound is still `workers × TT-cap` for instantaneous memory,
because no more than one chunk per worker is ever live; a worker that takes a
second chunk creates a second context sequentially. The pool fixes the chunk
count itself (`ceil(len / ceil(len / workers))`) so the bound does not depend on
rayon's adaptive splitter. The pool tests measure all of it: a 64-item call on
two workers creates two contexts and each handles 32 solves; at most `workers`
contexts are live at once; and every context is dropped when `map` returns.

## Naming: `create_context` versus `new_context`

`Solver::create_context` returns the backend's associated context type;
`ContextFactory::new_context` returns `Box<dyn SolverContext>`. The names
deliberately differ. Both traits are implemented by the same backend, and while
they shared the name `new_context`, calling it on a concrete solver was
ambiguous and needed `Solver::new_context(&solver)` to disambiguate. The
object-safe boxing method keeps the familiar `new_context` name; the static
primitive says what it creates.

## Consequences

A change of trump resets a context's transposition table, so solves must be
issued **deal-major with same-strain contracts adjacent**: the `ContractPayoff`
matrix iterates sampled deals in the outer loop and contracts in the inner loop.
Iterating contract-major would make every solve a fresh deal and reset the table
each time, discarding the very reuse the pool exists to exploit.
