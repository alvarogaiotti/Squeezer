# Prune dead weight: giubilo, SIMD, winner impls, benches

Status: resolved

## Problem

- `tests/giubilo_simulation.rs`: 66 lines, 100% commented out, references
  long-removed APIs. Pure dead weight.
- `PreprocessedDeal` SIMD stack (`src/performance_analysis.rs:393-711`,
  ~320 lines SSE2/AVX2/NEON): `analyze_hand` has the SIMD calls commented
  out (`:858, :880`) and uses `find_trick_winner_naive` (`:903`).
- Trick-winner logic in ~4 forms: live `find_trick_winner_naive`
  (`:759-796`), dead `find_trick_winner` (`:724-754`, only benches call
  it), older signature in commented tests (`:1030-1081`), plus
  `benches/my_benchmark.rs` hand-rolled variants (AVX2 branch references
  undefined `input`, won't compile).
- `benches/trick_winner_bench.rs`: ~185 of 307 lines commented out,
  duplicates `simd_benchmark.rs`; `benches/deck_pick_uniformity.rs` is not
  registered in `Cargo.toml` so never compiles.

## Direction

Delete the dead test, the commented bench bodies, the unregistered bench
(or register it), and the broken AVX2 branch. For the SIMD stack: deleting
is blocked on deciding SSE2-vs-scalar (`new_sse2` truncates the trump bit,
`new_neon` has a type error) — either fix dispatch or delete the stack;
do not rewire it into `analyze_hand` in this ticket.

## Non-goals

SIMD performance work; the batch-solve engine.

## Comments

- Done 2026-09-19: deleted `tests/giubilo_simulation.rs`,
  `benches/my_benchmark.rs` (+ Cargo stanza),
  `benches/deck_pick_uniformity.rs` (unregistered); stripped commented
  bodies/headers in `trick_winner_bench.rs` and `simd_benchmark.rs`.
- SIMD: `PreprocessedDeal::new()` is scalar-only (deleted AVX2/SSE2/NEON
  auto-dispatch); deleted uncompilable `new_neon`, ARM bench falls back
  to scalar; removed debug-print scaffolding and dead commented lines in
  `analyze_hand`. `new_sse2`/`new_avx2` stay as bench harnesses only.
- Revived commented winner tests as live
  `test_preprocessed_winner_agrees_with_naive` (fast == naive + expected
  positions, trump and NT).
