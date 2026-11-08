# 02 — Reject a candidate at the first failed seat

Status: resolved
Blocked by: None (can start immediately; ticket 01 supplies the bench cases the
verification compares against, but nothing here needs its code)

## Goal

A candidate deal whose seat fails its per-hand constraint stops being dealt at
that seat: the remaining seats are never filled, and the cross-hand accept
function only ever sees candidates that passed every per-seat constraint. The
documentation states that contract instead of contradicting it.

## Background

The rejection loop currently fills all four seats, builds the `Deal`, calls the
accept function, and only then checks the per-seat constraints. Three
documentation sites disagree about this:

- `with_function` promises the function is "called after hand constraints are
  checked";
- the same doc also promises it is "called for every candidate deal";
- `deal()`'s algorithm lists constraints before the accept function.

Only one of the first two can hold. The decision is: **after the constraints**,
and only for candidates that pass them.

Measured on the prototype: 0.31-0.80x on first-seat constraints, 0.99-1.00x on
late-seat constraints, +0.8% unconstrained. See
`docs/research/dealer3-vs-our-dealer.md` §3 row 1 and §3.1.

## Deliverables

- Check each seat's `HandConstraint` immediately after that seat's cards are
  set. A failure discards the candidate and starts the next one; the remaining
  seats are not dealt and the accept function is not called.
- Keep the existing seat order (N, E, S, W). Reordering to fill constrained
  seats first is out of scope and needs its own measurement.
- Acceptance rate and deal distribution must not change: same condition, same
  candidates per produced deal.
- Documentation:
  - `with_function` says it runs after the per-seat constraints and only for
    candidates that pass them; delete the "every candidate deal" claim.
  - `deal()`'s algorithm text matches the loop (per-seat checks interleaved
    with dealing, then the accept function).
- `HandConstraint::check_cached`'s doc claims it is "the dealer's rejection
  loop, which checks one candidate against several seats". It has no caller
  but `check` (the four seats are four different hands, so there is no summary
  to share). Either inline it into `check` or re-document it as the body of
  `check`; do not add a dealer caller.
- Two regression tests at the public `Dealer`/`DealerBuilder` seam (this is the
  pre-agreed seam; do not test private helpers):
  1. a closure that records every `Deal` it is handed, combined with a
     per-seat constraint that rejects most candidates — every recorded deal
     must satisfy the constraint;
  2. a constraint on an early seat (North) and one on a late seat (West) are
     both enforced, and a predealt seat keeps its cards and its constraint.
- Remove the two now-stale entries from `SESSION.md` (the accept-closure order
  note and the `check_cached` doc note). `SESSION.md` records open oddities,
  not fixed ones.

## Design reference

- `src/record/deal.rs` — `StandardDealer::deal`, `constraints_respected`,
  `with_function`, the `Deal` reuse across candidate iterations.
- `src/hand_constraint.rs` — `check` / `check_cached`.
- `src/record/deal.rs` doc comments: "called after hand constraints are
  checked", "called for every candidate deal", and the algorithm list.
- `tests/deal_generation.rs` — existing public-seam dealer tests.
- `docs/research/dealer3-vs-our-dealer.md` §3.1 — measured expectations.

## Verification

- [x] Test 1 fails before the change and passes after (red before green).
- [x] Test 2 passes and would catch a `break` that skips later seats.
- [x] `cargo test`, `cargo test --no-default-features`, `cargo fmt --all`.
- [x] With ticket 01's cases present, `cargo bench --no-default-features
      --bench deal_constraints` shows the North-constrained cases faster, the
      South/West cases unchanged, and unconstrained within noise.
- [x] No new clippy warnings in the touched code.

## Comments

Resolved 2026-11-08. Each seat's constraint is checked immediately after its
cards are set; a failure continues the candidate loop before the remaining
seats, the `Deal`, and the accept closure. `constraints_respected` is gone,
`check_cached` is inlined into `check`, the contract is stated at the five doc
sites that had contradicted each other, and the two tests live in
`tests/deal_generation.rs`.

Red before green: before the loop change,
`accept_function_only_sees_candidates_that_passed_the_seat_constraints` failed
with "accept closure saw a candidate whose North constraint failed: 5 HCP".
Test 2 passed before and after, as intended — it guards a future `break` that
skips a late seat.

Bench, twelve-case matrix, release, `--no-default-features`, `taskset -c 0`,
Criterion 10 samples, three runs, median of run estimates. The before column is
ticket 01's single pre-change run of the same bench; after-run spreads were
under 0.2%.

| Case | Before (ns) | After (ns) | After/Before |
| --- | ---: | ---: | ---: |
| `unconstrained` | 155.6 | 152.9 | 0.98 |
| `shape_balanced` | 350.4 | 236.7 | 0.68 |
| `hcp_12_17` | 526.5 | 290.7 | 0.55 |
| `shape_hcp_12_14` | 1,679.6 | 702.6 | 0.42 |
| `nt_15_17_north` | 3,248.3 | 1,221.2 | 0.38 |
| `nt_15_17_south` | 3,280.9 | 3,120.7 | 0.95 |
| `nt_15_17_west` | 3,283.0 | 3,186.1 | 0.97 |
| `function_ns_fit` | 334.6 | 330.9 | 0.99 |
| `shape_plus_function` | 1,934.3 | 1,369.4 | 0.71 |
| `two_seat_hcp` | 1,503.5 | 840.8 | 0.56 |
| `predeal_plus_hcp` | 262.2 | 196.2 | 0.75 |
| `polish_club` | 967.4 | 904.7 | 0.94 |

Geometric mean 0.703 over all twelve cases, 0.657 excluding the two the change
cannot help (`unconstrained` pays one extra `Option` test per seat; the
cross-hand closure has no per-seat constraint). The late-seat 1NT cases gain
3-5% rather than 60% because the candidate has already paid for three hands;
filling constrained seats first is the unfiled follow-on.

`deck_pick_performance` and `bench_handdescriptor` were rerun and are
unaffected, as expected: 47.2 ns fast-sampler / 62.4 ns uniform-sampler for a
13-card draw, and 1.2573 ns for one `HandConstraint::check`.

Verification: `cargo test`, `cargo test --no-default-features`,
`cargo fmt --all -- --check`, `cargo clippy --no-default-features --all-targets`
— all clean.
