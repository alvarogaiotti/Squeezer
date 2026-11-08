# 01 — Bench the constrained seat's position

Status: resolved
Blocked by: None (can start immediately)

## Goal

Make the permanent constraint benchmark show where the constrained seat sits in
the deal order. Today every case constrains North, so the benchmark cannot see
that rejecting a candidate after the first seat is much cheaper than rejecting
it after the third. This ticket adds the missing axis and records the current
numbers; it changes no library behaviour.

## Deliverables

- Three new cases in the existing constraint matrix, all using one constraint —
  balanced (4432/4333/5332) with 15-17 HCP — on North, South and West. Name
  them so the seat is visible in Criterion output, e.g. `nt_15_17_north`,
  `nt_15_17_south`, `nt_15_17_west`.
- The existing nine cases and their names stay unchanged.
- Record the current median for each new case in this ticket's Comments when
  resolving, with build and machine context (release profile, feature set,
  whether the process was pinned to one core).

## Design reference

- `docs/research/dealer3-vs-our-dealer.md` §3.1 — the measured seat-position
  effect and the protocol this should be able to reproduce.
- `benches/deal_constraints.rs` — existing matrix style, `sample_size(10)`.
- The dealer rejects candidates in seat order N, E, S, W.

## Verification

- [x] The matrix runs all twelve cases and the three new ones report medians.
- [x] `cargo bench --no-default-features --bench deal_constraints -- --test`
      (or equivalent compile-only check) passes.
- [x] Existing case names and constraints are unchanged.

## Comments

Medians of the three new cases, from one full run of

    taskset -c 0 cargo bench --no-default-features --bench deal_constraints

Build: `bench` profile (release), `--no-default-features`, process pinned to
core 0 with `taskset`. Criterion defaults otherwise (3 s warm-up, sample size
10). Host: 13th Gen Intel Core i7-13700HX, 24 logical CPUs, rustc 1.98.1. Full
log: `tmp/deal_constraints_bench.log`.

Criterion publishes a point estimate and a bootstrap confidence interval, not
a median; the column below is the middle figure of `time: [lower estimate
upper]`, which is what this ticket calls the median.

| case | estimate |
|---|---:|
| `deal_creation_constraints/nt_15_17_north` | 3.2483 µs |
| `deal_creation_constraints/nt_15_17_south` | 3.2809 µs |
| `deal_creation_constraints/nt_15_17_west` | 3.2830 µs |

Each case reported one outlier among ten measurements: high mild for North,
low mild for South, high mild for West.

The three estimates sit within 1.1% of each other, so this matrix cannot yet
distinguish seat position — exactly what the ticket predicted for the current
deal order. For comparison, `docs/research/dealer3-vs-our-dealer.md` §3.1
measured 2987.9 / 2992.2 / 3001.7 ns for the same three scenarios with its own
harness.

After ticket 02, the same bench separates them: North 1,221.2 ns (0.38x),
South 3,120.7 ns (0.95x), West 3,186.1 ns (0.97x). Full before/after table in
ticket 02's Comments.
