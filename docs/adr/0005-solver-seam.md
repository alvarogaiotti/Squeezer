# Solver seam: static factory, object-safe context, DDS behind a feature

The solver interface lives in the `squeezer` core as a `Solver` factory with an
associated `Context` type, an object-safe `SolverContext` hot interface, and an
object-safe `ContextFactory` (blanket-implemented for every `Solver`) so callers
may choose static dispatch or a boxed backend. Our own simulations take a
`Box<dyn ContextFactory>`, which keeps a solver type parameter out of the public
simulation API and makes mocks and runtime backend selection possible; the
dispatch cost is one indirect call per board against a multi-microsecond solve.
The DDS implementation lives in `squeezer` behind the `dds` feature, using the
`dds` crate only as raw FFI, so the core still builds with no native dependency
(ADR-0004). A `SolverPool` owns a dedicated sized rayon pool because worker
count bounds transposition-table memory (`workers × cap`).

## Consequences

A change of trump resets a context's transposition table, so solves must be
issued **deal-major with same-strain contracts adjacent**: the `ContractPayoff`
matrix iterates sampled deals in the outer loop and contracts in the inner loop.
Iterating contract-major would make every solve a fresh deal and reset the table
each time, discarding the very reuse the pool exists to exploit.
