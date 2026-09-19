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

## Non-goals

Presentation redesign; the batch-solve engine.
