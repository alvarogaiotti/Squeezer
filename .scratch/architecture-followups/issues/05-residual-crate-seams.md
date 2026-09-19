# Residual crate seams

Status: resolved

## Problem

- `src/performance_analysis.rs:19` imports `crate::linparser::PlaySequence`
  unconditionally, but `dds` does not imply `lin` (`Cargo.toml:28`), so
  `--no-default-features --features dds` breaks (plain
  `--no-default-features` builds since the `error.rs` gating fix).
- `DealerBuilder` field is still named `hand_descriptors` while storing
  `HandConstraint`s (`src/deal.rs:820`); same staleness in
  `with_hand_descriptors_all`.
- Docs claim simulations "may drive one dealer from many threads"
  (`src/deal.rs`) but no simulation loop parallelizes deal generation —
  the `Send + Sync` capability is latent and oversold.

## Direction

Gate the `PlaySequence` import/usage on `lin` (or make `dds` imply `lin`
via a documented decision); rename the stale field/methods (keeping
back-compat aliases); tone the doc down to capability ("safe to share")
until a simulation actually parallelizes. Verify each feature combo with
`cargo check`.

## Non-goals

Actual parallelization of simulation loops.

## Comments

- Done 2026-09-19: `performance_analysis` now requires
  `all(feature = "dds", feature = "lin")` (it consumes LIN play
  sequences; `dds`-alone stays for `simulation`). `DealerBuilder` field
  renamed `hand_descriptors` -> `hand_constraints` (compat method names
  kept). `BoardNumbering` docs state sharing capability without
  overselling unparallelized loops.
