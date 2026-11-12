# Runtime performance report

## Method

The working tree already contained an uncommitted performance candidate, so I compared each change
with a clean detached worktree at `HEAD` (`00d47b2`) rather than treating existing numbers as a
baseline. The public API was not changed.

Profiling was done before accepting each optimization:

- `perf record` on the clean `full_analysis/complete_hand` Criterion benchmark attributed 45.31% of
  cycles to `performance_analysis::analyze_hand`, 27.49% to `record::position::trick_winner`, and
  16.02% to the `malloc`/`cfree` pair. The allocator samples justified removing temporary vectors.
- A focused DDS-adapter profile attributed 97.79% of an isolated position-to-`DdsDeal` conversion to
  `TryFrom<&Position>`. The old implementation walked every set card through `SuitIterator`, even
  though the hand bitboard already contains the four DDS rank masks.
- `perf record` on `bench_payoff` (7,463 samples over five seconds) attributed essentially all
  end-to-end time to native DDS routines: `Moves::MoveGen123` 10.67%, `QuickTricks` 8.70%,
  `Moves::MakeNext` 7.53%, and `weight_alloc_trump0` 5.91%, followed by alpha-beta and transposition-table
  routines. Native LTO was tried separately and discarded because it failed to link; the Rust
  adapter optimization below is intentionally measured in isolation rather than misrepresented as a
  native-solver speedup.
- `perf record` on the 1,000-deal lead pipeline (`crates/squeezer-sim/benches/bench_leadsim.rs`, 43,750 samples over
  five seconds, 44.8 s per iteration) confirmed the same picture for the all-legal-card path: no Rust
  symbol reached the 0.5% reporting threshold, and the top entries were native search routines
  (`Moves::MoveGen123` 10.72%, `QuickTricks`, alpha-beta, transposition-table lookups).

Because native search cannot be improved from Rust without rewriting DDS, the simulation paths were
profiled in two layers. A new mock-solver benchmark, `crates/squeezer-sim/benches/simulation_paths.rs`, substitutes a
`SolverContext` that returns one legal-card result per card. That makes the cost we can actually
change — position derivation, equivalence-class expansion, denomination baselines, statistics, and
report construction — measurable on its own instead of being hidden behind a 7–45 s native solve.
Every mock case reports the post-processing of 1,000 deals.

Criterion timings used `taskset -c 0` and three independent runs. The earlier microbenchmarks used a
one-second warm-up, a three-second measurement, and 20 samples; the deal matrix used the same
warm-up with a two-second measurement and 10 samples because its tight cases are much slower (nine
cases in §4; twelve in §6, after three seat-position cases were added).
Each table reports the median of the three run medians; the range is the range of those run medians,
not Criterion's within-run confidence interval.

## 1. Allocation-free fixed buffers in `analyze_hand`

**Changed:** `crates/squeezer-sim/src/performance_analysis.rs` now copies play cards directly into its existing
`[Card; 52]` scratch array. Complete-trick DD results use a four-byte stack array, and the short
final trick consumes the existing DD iterator directly. This removes one `Vec<Card>` allocation per
hand and one `Vec<u8>` allocation per complete trick.

**Coverage:** empty and partial play records (including rotating seats and DD deltas), a missing DD
result for a complete trick (now a typed `AnalysisInputError` rather than a panic), and the
existing full-hand/DDS-backed analysis tests.

| Version | Median | Run-median range |
| --- | ---: | ---: |
| Clean `HEAD` | 281.92 ns/hand | 280.62–290.67 ns (3.56%) |
| Allocation change only | 217.04 ns/hand | 214.87–221.32 ns (2.97%) |

This is a **23.0% improvement**, comfortably outside the observed run-to-run variance. The change is
kept: it is a small, safe allocation reduction with no allocation or hot-path behaviour change.
Malformed DD input later became a typed error instead of a panic, which does not touch this path.

## 2. Fixed-size scan for `trick_winner`

**Changed:** `crates/squeezer-model/src/record/position.rs` replaces the closure plus `max_by_key` state machine with a
local winning-reference loop over the four-card trick. The `>=` comparison deliberately preserves
`max_by_key`'s last-element behavior when strengths tie, including the existing degenerate
duplicate-card behavior of this public helper.

**Coverage:** the existing trump, no-trump, led-suit, discard, and position-derivation cases remain.
A regression test was added before the implementation change for the last-equal-strength/duplicate
card case.

The allocation-only build is the before measurement for this change; it uses the same benchmark and
machine as the final build.

| Version | Median | Run-median range |
| --- | ---: | ---: |
| Allocation change only | 217.04 ns/hand | 214.87–221.32 ns (2.97%) |
| Allocation change + fixed winner loop | 195.49 ns/hand | 195.02–196.68 ns (0.85%) |

This is a **9.9% incremental improvement** and a **30.7% total improvement** over clean `HEAD`.
It adds no unsafe code or public surface, so it is kept.

## 3. Direct rank-lane extraction in the DDS position bridge

**Changed:** `crates/squeezer-solver/src/adapters.rs` now copies the four 16-bit lanes of each `Cards` bitboard directly
into DDS's `remain_cards[hand][suit]` masks. The previous implementation iterated over every set bit
and performed a suit match and rank shift for each card. The full `0xffff` lane mask is intentional:
it preserves even malformed rank bits that the old conversion passed through.

**Coverage:** a new reference-comparison test checks every suit/rank bit in the example deal. Existing
mid-trick leader/current-card tests and DDS oracle tests remain in place.

The permanent regression benchmark is `crates/squeezer-solver/benches/dds_conversion.rs`:

```text
cargo bench -p squeezer-solver --bench dds_conversion -- --noplot --warm-up-time 1 --measurement-time 3 --sample-size 20
```

| Version | Median | Run-median range |
| --- | ---: | ---: |
| Card-by-card conversion | 77.92 ns/conversion | 77.56–78.30 ns (0.96%) |
| Four-lane extraction | 8.31 ns/conversion | 8.28–8.33 ns (0.63%) |

This is an **89.3% improvement**, far outside noise, with no unsafe code or API change. The native
`bench_payoff` sanity run moved from 7.268 s to 7.227 s (about 0.6%, within end-to-end noise), so the
report does not claim that this adapter change materially accelerates the native solver; it removes a
real, repeatable conversion cost for every DDS call.

## 4. Deterministic final-seat fast path in deal creation

**Profile:** the constraint-matrix benchmark showed that the rejection loop's first three
`Cards::pick_rng::<FastSampler>` calls and `rand_chacha` refill dominated representative cases. In
the post-change `shape_hcp_12_14` profile they still accounted for 64.97% and 21.70% of samples;
the benchmark/dealer closure accounted for 12.09%. In an unconditioned deal the final seat asks for
exactly the cards left in the deck; more generally, any seat whose fill equals the remaining deck
has no sampling decision left.

**Changed:** `crates/squeezer-model/src/dealer.rs` now takes the remaining deck directly when `fill == deck.len()`,
using a `Cards::take_all` helper in `squeezer-cards`' `card` module. The public `Cards::pick_rng` and
`FastSampler` behavior, including caller-supplied RNG consumption, is unchanged.

**Coverage:** the new `take_all` unit test covers non-empty and empty decks, and the existing dealer
suite covers unconstrained, predealt, conditioned, constrained, and accept-function generation.

The permanent matrix benchmark is `crates/squeezer-model/benches/deal_constraints.rs`. Each case ran three independent
Criterion measurements with a one-second warm-up, two-second measurement, and 10 samples:

| Case | `HEAD` median (range) | Fast path median (range) | Improvement |
| --- | ---: | ---: | ---: |
| Unconstrained | 176.17 ns (176.00–176.62) | 143.53 ns (143.46–143.56) | 18.5% |
| Balanced shape | 397.24 ns (396.84–397.29) | 328.10 ns (327.94–328.89) | 17.4% |
| HCP 12–17 | 597.26 ns (596.62–597.30) | 488.04 ns (487.74–488.81) | 18.3% |
| Shape + HCP 12–14 | 1.8988 µs (1.8966–1.8996) | 1.5542 µs (1.5525–1.5562) | 18.2% |
| Cross-hand function | 370.94 ns (370.83–371.27) | 303.70 ns (303.57–304.05) | 18.1% |
| Shape + function | 2.1833 µs (2.1825–2.1843) | 1.7947 µs (1.7945–1.8010) | 17.8% |
| Two-seat HCP | 1.7045 µs (1.7028–1.7052) | 1.4080 µs (1.4071–1.4090) | 17.4% |
| Predeal + HCP | 298.57 ns (298.49–298.89) | 246.20 ns (246.02–246.23) | 17.5% |
| Polish Club | 1.0785 µs (1.0777–1.0798) | 0.89083 µs (0.89052–0.89133) | 17.4% |

The gain is consistent across loose, tight, cross-hand, multi-seat, and conditioned cases, so it
is kept. The remaining first-three-pick sampler/RNG cost is documented above as the next possible
investigation, but changing public sampler semantics would be a separate, riskier task.

## 5. Simulation post-processing: reusable Card EV buffers and a validated-opening shortcut

**Profile:** the mock-solver profile of the 1,000-deal card-EV post-processing attributed roughly half
of its samples to allocator traffic (`realloc` 18.23%, plus the `malloc`/`free` pair) and another 9%
to the per-card `other_scores` `Vec::from_iter`; `Denomination::value` accounted for 15%. The mock
profile of contract payoff attributed 56% to `Position::new`, because every contract of every sampled
deal derived its opening position through the general validation path, which rescans all 52 cards for
duplicates and recomputes the trailing-trick arithmetic for a table that is empty by construction.

**Changed, Card EV:** `crates/squeezer-sim/src/simulation/card_ev.rs` now hoists the three per-deal vectors
(`physical`, `scores`, and `other_scores`) out of the deal loop and reuses them by `clear`/`extend`.
The allocation that remains per physical card is gone, as is the per-deal growth of the two other
buffers; the loop now iterates the reused `physical` buffer by reference and copies the
`Copy` outcome instead of consuming the vector.

**Changed, positions:** `crates/squeezer-model/src/record/position.rs` adds a private `Position::from_valid_opening`, and
`from_deal_prefix` returns it directly for the empty prefix. `Deal` already guarantees four disjoint
13-card hands, so the duplicate scan and the played-count arithmetic that `Position::new` performs
cannot fail for this input. The general path is untouched: a four-play prefix still scans the plays,
removes the cards, derives the trick winner, and validates the snapshot.

**Coverage:** the existing opening-lead test already checks all five strains, all four leaders, and
that every hand equals the deal's, and the mid-trick and completed-trick tests exercise the unchanged
validation path. A regression test was added before the Card EV change to check that compared
denomination values stay correct across repeated deals, which is the case a stale reused buffer would
break. The new `position_from_prefix` benchmark case pins the unchanged four-play path so the
shortcut cannot silently widen into it.

Both sides of the table were measured with the same benchmark binary, protocol, and machine: the two
library files were backed up, hand-reverted to remove exactly this session's edits, rebuilt in release
mode, measured three times, and then restored. Each case used `taskset -c 0`, a one-second warm-up, a
two-second measurement, and 10 samples.

| Case (1,000 deals unless noted) | Before median (range) | After median (range) | Improvement |
| --- | ---: | ---: | ---: |
| Card EV, raw points | 1.2685 ms (1263.1–1271.0) | 334.40 µs (333.93–335.95) | 73.6% |
| Card EV, imps vs. average | 1.4564 ms (1448.4–1457.7) | 536.28 µs (534.39–538.25) | 63.2% |
| Lead post-processing | 1.3048 ms (1300.5–1306.0) | 351.29 µs (350.59–351.70) | 73.1% |
| Contract payoff post-processing | 629.89 µs (628.37–630.44) | 395.72 µs (394.82–397.19) | 37.2% |
| Opening position (single derivation) | 66.10 ns (66.03–66.17) | 7.82 ns (7.41–7.83) | 88.2% |
| Four-play prefix (single derivation) | 72.45 ns (72.05–72.45) | 72.43 ns (72.35–72.79) | unchanged |

Both changes are kept: every gain is an order of magnitude above the 0.2–0.7% run-to-run spread, no
unsafe code or public surface was added, and the general position path measures unchanged.

**End-to-end effect, stated honestly.** The native 1,000-deal payoff benchmark is 7.17 s, of which the
entire post-processing that these changes speed up is about 0.4 ms. An A/B with the same binary and
protocol gave 7.1697 s (7.0497–7.2863) for the candidate against 7.2273 s (7.1102–7.3499) before, and
Criterion reported no change (p = 0.53). The 1,000-deal lead pipeline is 44.8 s against 0.35 ms of
post-processing, about 0.0008%. These are real, repeatable reductions in the simulation-only work, but
they are invisible next to DDS search and this report does not claim a native speedup from them.

## 6. Seat-early rejection in the deal loop

**Changed:** `StandardDealer::deal` now checks a seat's `HandConstraint` as soon as that seat's cards
are filled. A failing seat discards the candidate before the remaining seats are dealt, before the
`Deal` is built, and before the cross-hand accept closure runs. At that point the seat order was
still N, E, S, W, so a constraint on a late seat still paid for the seats before it; §6.1 closes that
gap.
The accept closure's documented contract changed with it: it now runs after the per-seat constraints,
on candidates that passed them, and no longer claims to see every candidate.

**Why:** the rejection loop used to fill all four hands and build the `Deal` before testing anything,
so a candidate doomed by its first seat still paid for the other three. The rejection factor is not
small — 20.5 candidates per produced deal for a 15-17 1NT, 114 for a weak two with a closure — so most
of that work was thrown away.

**Method:** the twelve-case matrix in `crates/squeezer-model/benches/deal_constraints.rs`, released build,
`--no-default-features`, `taskset -c 0`, Criterion with a two-second measurement and 10 samples, three
independent runs, median of the three run estimates. The before column is ticket 01's pre-change run
of the same bench on the same machine; after-run spreads stayed under 0.2%.

| Case | Before (ns) | After (ns) | After/Before |
| --- | ---: | ---: | ---: |
| Unconstrained | 155.6 | 152.9 | 0.98 |
| Balanced shape | 350.4 | 236.7 | 0.68 |
| HCP 12-17 | 526.5 | 290.7 | 0.55 |
| Shape + HCP 12-14 | 1,679.6 | 702.6 | 0.42 |
| 1NT 15-17, North | 3,248.3 | 1,221.2 | 0.38 |
| 1NT 15-17, South | 3,280.9 | 3,120.7 | 0.95 |
| 1NT 15-17, West | 3,283.0 | 3,186.1 | 0.97 |
| Cross-hand fit closure | 334.6 | 330.9 | 0.99 |
| Shape + function | 1,934.3 | 1,369.4 | 0.71 |
| Two-seat HCP | 1,503.5 | 840.8 | 0.56 |
| Predeal + HCP | 262.2 | 196.2 | 0.75 |
| Polish Club (South) | 967.4 | 904.7 | 0.94 |

The geometric mean is 0.703 across the twelve cases and 0.657 excluding the two the change cannot
help: `unconstrained`, where the only added cost is one `Option` test per seat, and the cross-hand
closure, which has no per-seat constraint to reject on. The seat-position row is the point: the same
1NT constraint is 2.7x faster when it sits on the first seat and only 3-5% faster on the last, because
the candidate has already paid for three hands by then.

### 6.1 Constrained-first fill order

**Changed:** the dealer now computes a **fill order** once, when it is built: seats carrying a
`HandConstraint` first, then the unconstrained seats, keeping North, East, South, West inside each
tier. The rejection loop iterates that order instead of the canonical one, so a doomed candidate dies
after the first hand rather than the third or fourth. Known cards never change a seat's tier, and the
last seat still takes the remaining deck directly when nothing is left to sample. The order is an
implementation detail: there is no builder option and no public API change. Any fixed fill order draws
the hands of the same partition in a different order, so the distribution of accepted deals is
unchanged; only where the check cost lands moves.

The same twelve-case matrix, same machine and protocol as §6 (release, `--no-default-features`,
`taskset -c 0`, Criterion with a one-second warm-up, two-second measurement and 10 samples, three
independent runs, median of the run estimates; run spreads stayed at or under 0.20%). The before
column is §6's seat-early run.

| Case | Seat-early (ns) | Constrained-first (ns) | Shipped/Before |
| --- | ---: | ---: | ---: |
| Unconstrained (control) | 152.9 | 150.7 | 0.99 |
| 1NT 15-17, North (reference) | 1,221.2 | 1,208.9 | 0.99 |
| 1NT 15-17, South | 3,120.7 | 1,208.4 | 0.39 |
| 1NT 15-17, West | 3,186.1 | 1,209.8 | 0.38 |
| Polish Club (South) | 904.7 | 449.1 | 0.50 |
| Two-seat HCP | 840.8 | 740.0 | 0.88 |

The six cases omitted from the table (balanced shape, HCP 12-17, shape + HCP 12-14, cross-hand fit
closure, shape + function, predeal + HCP) all landed within 1% of the seat-early run, in both
directions.

Every threshold held: the late-seat 1NT cases are 1.00-1.01x of North, two-seat HCP is 11-12% better
than the seat-early order, the unconstrained control is within 1%, and no case is worse. Multiplying
the two measured ratios puts the twelve-case geometric mean at about 0.56 of the original loop (0.703
for seat-early, times roughly 0.80 for this change); that composite is derived, not a fresh run
against the old loop. Candidates per produced deal are unchanged: the fill order changes only how
much work a rejected candidate costs, and ticket 01 measured identical acceptance on every case
(largest deviation 0.3%, inside sampling error).

## 7. Candidate throughput, the dealer's documented time, and post-processing cost

**Method.** `Dealer::deal_with_attempts` is a new provided method: it returns
the deal together with the number of candidates generated for that call,
counting the accepted one. `StandardDealer` overrides it with its real rejection
loop; the default reports `1` for dealers that do not observe their own loop, so
existing implementations keep compiling. `crates/squeezer-model/benches/deal_constraints.rs`
measures the mean candidates per accepted deal over 1,000 accepted deals only to
scale Criterion's integral `Throughput` display, then times the same calls. The
timed run itself accumulates the exact candidate count and elapsed time and
prints candidates per accepted deal, nanoseconds per candidate and candidates
per second, so the printed line is the authoritative throughput and the
`Throughput` value is only a display scale. The group is named
`deal_candidate_throughput` (formerly `deal_creation_constraints`) so the
element is not mistaken for a deal; that rename resets stored Criterion
baselines, which is why this section records the numbers. Because the dealer
draws from a thread-local RNG, the attempt ratio itself varies a few percent
between runs; each row below is the median of three independent runs, with the
same protocol as §6.1 (release, `taskset -c 0`, one-second warm-up, two-second
measurement, ten samples).

| Case | Candidates per accepted deal | ns per accepted deal | ns per candidate |
| --- | ---: | ---: | ---: |
| Unconstrained (control) | 1.00 | 155.4 | 155.7 |
| Balanced shape | 2.10 | 240.4 | 114.8 |
| HCP 12-17 | 3.25 | 293.6 | 90.3 |
| Shape + HCP 12-14 | 10.32 | 706.7 | 68.8 |
| 1NT 15-17, North | 20.55 | 1,226.2 | 59.7 |
| 1NT 15-17, South | 20.55 | 1,225.5 | 59.8 |
| 1NT 15-17, West | 20.55 | 1,226.5 | 59.7 |
| Cross-hand fit closure | 2.05 | 335.5 | 164.1 |
| Shape + function | 11.88 | 1,390.4 | 117.5 |
| Two-seat HCP | 8.92 | 747.0 | 83.8 |
| Predeal + HCP | 1.61 | 199.2 | 123.8 |
| Polish Club (South) | 5.71 | 458.5 | 80.1 |

The two axes matter independently. The 1NT row matches ticket 01's ~20.5
candidates per accepted deal, but its average candidate costs only ~60 ns: an
early seat rejection abandons the candidate after one hand. The unconstrained
row has one candidate per deal and therefore pays the full four-hand fill,
~155 ns. A single "ns per deal" number hides whether a change moved the
rejection ratio or the per-candidate work; the benchmark now reports both and
the report divides them.

The unconstrained row is 3% above §6.1's 150.7 ns, and the rest of the matrix
drifted similarly on this run. That is machine/run drift, not the new
`deal`/`deal_with_attempts` split: a controlled A/B on the same machine replaced
the delegating `deal` with the pre-change loop and measured 156.4 ns versus
155.4 ns unconstrained, and 1.2269 µs versus 1.2262 µs for 1NT North. The
delegating form is kept; it adds no measurable cost and keeps one rejection
loop.

**Dealer documentation re-baselined.** The `StandardDealer` docs claimed 1-5
microseconds per unconstrained deal and 10-50 microseconds with light
constraints. The benchmark contradicts both by roughly an order of magnitude on
every matrix row, so the claim was replaced with the measured medians above and
a pointer to this report and the benchmark. The docs state that the figures are
machine-specific rather than a contract.

**`CardStatistics::from_outcomes`.** A new case,
`simulation_paths_mock/card_statistics_from_outcomes`, feeds 1,000 outcomes with
14 distinct declarer-trick/score values: the median is 8.05 µs per card
(8 ns per outcome; run medians 8.01-8.05 µs). Card-EV calls it once per
physical legal card, and the opening-lead fixture has 13 legal leads, so a
1,000-deal evaluation makes 13 calls: about 105 µs, or 31% of the 334 µs mock
card-EV run, under varied outcomes.

That 31% is the conservative bound, not the fixture's measured difference. The
mock dealer is fully predealt and the mock solver returns the same trick count
for every card, so each card's 1,000 outcomes are identical: sorting and the
`BTreeMap` collapse to a trivial case. Stubbing `from_outcomes` to return
`Self::default()` and re-running the same benchmark moved
`mock_card_ev_raw` from 334.26 µs to 332.56 µs, a 0.5% change. The function's
real share in this pipeline therefore spans roughly 0.5-31% depending on how
varied the per-deal outcomes are, and even the upper bound is post-processing
only.

`perf record` on `mock_card_ev_raw` (31,417 samples) attributed 2.76% to the
out-of-line `from_outcomes` symbol and 0.59% to `BTreeMap::or_insert`, but the
optimizer inlines most of the body into `CardEvSimulation::run`, so the sampled
symbol percentage understates the function and is not used as its share. The
direct timing and the stub A/B above are the measurements.

**Decision: keep it unoptimised.** `score_distribution` is a public
`BTreeMap<i32, usize>`, and `from_outcomes` accepts caller-constructed
outcomes, so it cannot assume `score` is a function of `declarer_tricks`; a
trick-indexed shortcut would either change the public field or add an
unstated precondition. The remaining work is one 1,000-element `Vec`, a sort
for the percentiles, and the map inserts. Even eliminating all of it saves at
most ~105 µs, about 0.0015% of a ~7 s native 1,000-deal payoff run and 0.0002%
of the 44.8 s lead pipeline. That gain does not justify changing a public
statistics contract.

**Unused `rayon` dependency removed.** `squeezer-sim` declared
`rayon = "1.12.0"` but had no code using it: the comments that mention rayon
describe the pool owned by `squeezer-solver`, which still depends on rayon. The
dependency is gone from the manifest and from `Cargo.lock`'s edge for
`squeezer-sim`; `AGENTS.md` and ADR-0008's dependency table were updated so they
no longer list it for this crate. Builds and tests are unchanged.

**Doctests.** The contract-payoff, lead and formatter examples remain
`no_run`, each now with the reason in its module docs: they use the native DDS
backend and solve thousands of deals, which the doctest suite does not pay for,
so they are compiled for API checking but not executed. The `simulation` module
docs gained an executable example that injects a small mock `ContextFactory`
and runs a card evaluation end to end; it passes with default features and with
`--no-default-features`, so the mock path is exercised without DDS.

Ticket-12 commands:

- `cargo test --workspace` - passed.
- `cargo test -p squeezer-sim --no-default-features --doc` - passed: 3
  doctests, including the executable mock example.
- `cargo bench -p squeezer-model --bench deal_constraints` - passed; the
  candidates-per-deal table above.
- `cargo bench -p squeezer-sim --bench simulation_paths` - passed; the
  `from_outcomes` and mock post-processing numbers above.
- `cargo build --workspace --all-targets --all-features` - passed.
- `cargo fmt --all --check` - passed.

## Rejected work

- A first version of the deal fast path was placed directly in the public `FastSampler`; it was
  discarded because skipping RNG draws there changes the observable state of a caller-supplied RNG.
  The final version is restricted to `StandardDealer` and keeps that public contract intact.
- A separate `Cards::high_card_points` mask refactor was screened but discarded: the constraint check
  measured 1.2839 ns versus 1.2749 ns on `HEAD` (a slight regression), and the end-to-end constrained
  dealer check was unchanged within noise. It is not part of the final diff.
- The SIMD preprocessing/trick-winner benchmark paths were not rewritten. They are small,
  benchmark-only paths in the current analysis flow, and the fixed winner loop addresses the measured
  hot path without architecture-specific unsafe code.
- A native DDS `-flto` experiment was discarded before acceptance: adding `-flto` to the C++ archive
  made `rust-lld` leave the `dds_c_*` symbols unresolved, so it could not produce a runnable
  benchmark. The original build flags were restored; no build-system change is in the final diff.
- The next Card EV hotspot, `CardStatistics::from_outcomes`, was deliberately
  left alone. §7 measures it directly: 8.05 µs per 1,000-outcome card and 13
  calls per 1,000-deal evaluation, up to ~31% of the mock card-EV
  post-processing but only 0.5% on the fixed-deal fixture, and at most about
  0.0015% of a native 1,000-deal run. It builds the public `score_distribution`
  map and sorts each card's values; fixing it well would mean either changing
  that public `BTreeMap` field or adding a trick-count-indexed fast path that
  assumes `score` is a function of `declarer_tricks` - an assumption the public
  `from_outcomes` cannot make for caller-constructed outcomes. Poor
  gain-to-risk ratio, so it stays as it is.

## Validation

> Recorded before the crate split, against the then-monolithic root package.
> The workspace is virtual now, so the root-level commands below are history;
> use the per-crate commands in `AGENTS.md` for current runs.

- `cargo test` — passed: 279 unit tests, integration tests, and 57 doctests.
- `cargo test --no-default-features` — passed: 195 unit tests and 54 doctests.
- `cargo test --features serde` — passed: 284 unit tests, integration tests, and 57 doctests.
- `cargo test --manifest-path dds/Cargo.toml` — passed: DDS unit, integration, and doctests.
- `cargo check --no-default-features --features dds,lin` — passed.
- `cargo fmt --all -- --check` and `git diff --check` — passed.
- `cargo clippy --all-targets` — passed with the pre-existing decimal-bitwise warning in
  `performance_analysis` test code; `cargo clippy --no-default-features --all-targets` passed.

No existing public function signatures or types were changed. (Ticket 12 later
added the provided `Dealer::deal_with_attempts` method, which is non-breaking:
existing implementors inherit the default.)
