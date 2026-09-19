# Decorative traits and unreferenced formatters surface

Status: ready-for-agent

## Problem

- `Comparable` (`src/simulation/core.rs:22`) is a zero-method marker
  (`Clone + Eq + Hash`) adding generic-parameter noise
  (`RawPayoffData<T: Comparable>`, `PayoffMatrix<T: Comparable>`).
- `TerminalFormattable` (`src/simulation/formatters.rs:578-604`) has two
  ~5-line pass-through impls and no generic consumer.
- `formatters.rs` `export`/`charts` modules are unreferenced in code, yet
  publicly exported via `simulation/mod.rs`.

## Direction

Collapse `Comparable` into plain bounds; delete or hide
`TerminalFormattable`/`export`/`charts` from the public surface. No
behavior change; existing tests stay green.

## Comments

- Done 2026-09-19: `Comparable` collapsed to
  `Clone + Eq + std::hash::Hash` bounds (`core.rs`, `payoff.rs`,
  `formatters.rs`, guide doc); `TerminalFormattable` + impls deleted.
- `export`/`charts` deliberately LEFT in place: they are referenced as
  user-facing features by `bridge_sim_guide.md` (`export::*` import,
  `payoff_to_csv` sections) and `bridge_sim_summary.md`, so the
  "unreferenced" premise does not hold. Hiding or deleting them needs a
  docs decision first — follow up with the maintainer.

## Non-goals

Presentation redesign; the batch-solve engine.
