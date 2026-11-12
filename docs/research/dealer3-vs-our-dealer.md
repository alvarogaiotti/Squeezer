# Dealer3 versus our dealer: language surface, published speed, copyable ideas, architecture

This is a research note, not a design. Every claim is cited by file and line;
where the tree is silent the note says so. No benchmark was run and nothing was
built for it, so every performance figure below is read out of somebody's
committed result file or prose report, and is labelled as such.

`D3/` is shorthand for `vendor/Dealer3/`; every path resolves by substitution.

Vocabulary follows `CONTEXT.md`: **Deal** (un-played four hands), **Hand**,
**Seat**, **Known cards** (predealt plus already-played cards fixed to a Seat),
**Auction**, **Call**, **Contract**, **Position**, **Solver context**, **TT**,
**DD-optimal**, **card EV** (`CONTEXT.md:11-59,61-84`).

## The premise, restated

The comparison that matters is **our dealer against their dealer**. Their
dealer against the C dealer is a different question, and the vendored numbers
for it have already moved twice: the 2026-01-07 pre-test reports Rust at 2.3x
slower than the original C dealer on the same RNG (`D3/performance-pretest-2026-01-07.md:11-16`),
then 1.14x slower after the hash-map work (`:255-260`), and the current
benchmark reports single-threaded dealer3 at **0.9x** the natively built C
dealer, geometric mean, i.e. about 10% slower (`D3/bench/results/CURRENT.md:34`,
`D3/bench/README.md:145-149`). None of those numbers involve us.

**Verdict on the premise: unanswered, and cheaply answerable.** There is no
published comparison of our dealer against theirs, no shared corpus, and no
shared unit of work. What their corpus does establish is that a comparison is
constructible (fixed-work mode, a synthetic generation-only baseline), and what
our own numbers establish is that we currently measure a different quantity
(produced deals, not candidates evaluated). The short answer to "are they
faster" is: *not established either way* — and the parts that can be inferred
from published figures point in opposite directions (their generation loop is
parallel, ours is not; our per-candidate check is cheaper than their
interpreter).

## 1. Expressiveness delta

### 1.1 What their language is

`.dlr` is a small imperative language, not a constraint algebra. It has 26
functions under 51 spellings (`D3/docs/FILTER_LANGUAGE_STATUS.md:38`), an
operator table with eight precedence levels, the last of which is assignment
(`:130-151`), statement forms for `condition`, `produce`, `generate`, `action`,
`average`, `frequency`, `pointcount`/`altcount`, `dealer`, `vulnerable`,
`predeal`, `seed`, `csvrpt` and more (`:157-178`), and per-deal variable
assignment (`:177,222-248`). The AST is
a general expression tree (`D3/dealer-parser/src/ast.rs:11-74,198-236`), and the
function enum runs from `Hcp` through `Top5` to `Tricks`/`Par`/`Rnd`
(`ast.rs:346-417`).
Script parameters `$0`-`$9` are textual substitutions performed before parsing
(`FILTER_LANGUAGE_STATUS.md:250-263`).

Shape matching has two syntaxes, both compiled to a 560-bit mask at parse time
(`D3/dealer-core/src/shape.rs:1-41`):

- `shape(north, any 4333 + any 4432 + any 5332)`: four ordered suit lengths,
  `x` for any, `any` for all permutations, `+`/`-` for union/difference
  (`FILTER_LANGUAGE_STATUS.md:73`).
- `shape{north, 4M(3+3+2+)}`: François Dellacherie's language with `5+`, `2-`,
  `[3-5]`, `(431)` for the remaining suits in any order, `M`/`m` for either
  major/minor, and `:` for conditions on the four lengths, expanded to the
  `shape(...)` form before the script is parsed (`D3/dealer-parser/src/fdshape.rs:1-20`).

The evaluator walks the AST per deal with a variable cache keyed by `&str` in an
`FxHashMap`, cleared at a borrow-checked deal boundary
(`D3/dealer-eval/src/lib.rs:330-403`), and classifies at parse time which
variables may reach `rnd()` and so must be re-evaluated at every mention
(`:687-706`). `tricks()`, `dds()` and `par()` read a per-deal solved-cell memo
before searching (`:1492`, `D3/dealer-run/src/dd_demand.rs:1-29`).

### 1.2 Constraint kinds, ours against theirs

"Ours" means the declarative surface: `HandConstraint` /
`HandConstraintBuilder` / `HcpRange` (`src/hand_constraint.rs:130-319`,
`src/hand.rs:259-341`) and the shape-pattern pipeline behind it
(`src/shapeparser/parser.rs:14-29`, `src/shape.rs:354-369`). Where the only way
to say something is `DealerBuilder::with_function`, the row says so: that escape
hatch is arbitrary Rust run once per candidate (`src/record/deal.rs:1200-1208`).

| Constraint kind | Dealer3 (`.dlr`) | Ours (declarative) | Evidence |
|---|---|---|---|
| Whole-hand HCP | `hcp(seat)`, scale replaceable by `pointcount` | `HcpRange`, fixed 4-3-2-1, clamped 0-37 | D3 `FILTER_LANGUAGE_STATUS.md:44-45`; `src/hand.rs:285-341` |
| HCP restricted to one suit | `hcp(seat, suit)` | no (closure only) | D3 `vocabulary.rs:136-148` |
| Controls (A=2, K=1) | `controls(seat[, suit])` | no (closure only) | D3 `FILTER_LANGUAGE_STATUS.md:46` |
| Losing trick count | `losers(seat[, suit])`, with the short-suit rules spelled out | no (closure only) | D3 `FILTER_LANGUAGE_STATUS.md:49` |
| Honour-count families | `aces/kings/queens/jacks/tens`, `top2`…`top5`, `c13`, `pt0`…`pt9` | no (closure only) | D3 `FILTER_LANGUAGE_STATUS.md:79-105` |
| Suit quality, whole-hand CCCC | `quality(seat, suit)`, `cccc(seat)` | no | D3 `FILTER_LANGUAGE_STATUS.md:51-54` |
| Shape union with exclusion | one call: `any 4432 + any 4333 - 5xxx`; braces shorthand | union of patterns at construction, `add_shape`/`remove_shape`; `x` joker, `(…)` for a permuted suit group | D3 `:73-74`; `src/hand_constraint.rs:168-185,401-449`; `src/shapeparser/parser.rs:14-29,62-72`; `src/shapeparser/interpreter.rs:432-457` |
| "Either major" / length ranges in one token | `4M`, `[3-5]`, `5+`, `:` conditions | no; must enumerate (`4xxx`, `x4xx`, …) | D3 `fdshape.rs:1-20` |
| Longest suit | expressible via lengths (`spades(n) >= hearts(n)` …) | first-class shortcut: `HandConstraint::longest` over const pattern tables | D3 `FILTER_LANGUAGE_STATUS.md:60-63`; `src/hand_constraint.rs:50-63,205-216` |
| A specific card in a seat | `hascard(seat, card)` as a predicate | only as fixing (predeal/known), not as a predicate | D3 `:75`; `src/record/deal.rs:1112-1150` |
| Arithmetic / comparisons across seats | full integer algebra, ternary, bool-to-int coercion (`hcp(north) + shape(north,5xxx)`) | no (closure only) | D3 `:130-151`; `bench/corpus/GIB_1C-P-Resp.dlr:46` |
| Per-deal randomness (`rnd(n)`) | yes, own stream seeded from the deal, fresh at every mention | no | D3 `:122-123,237-243` |
| DD tricks as a *filter* | `tricks(seat, strain)` / `dds()` in the condition | no; DD runs after generation | D3 `:111-115`; `src/simulation/solver.rs:71-87` |
| Scoring as a *filter* | `score(...)`, `imps(...)`, `par(side)` | no; `Contract::score` is analysis-side | D3 `:117-121` |
| Generation conditioned on DD result | yes, by putting `tricks()` in the condition | not stated: nothing in the tree conditions generation on DD | D3 `:111-112`; `src/simulation/solver.rs:71-87` |
| Generation conditioned on an Auction or Contract | no | no | D3: no grammar construct for either (`D3/dealer-parser/src/ast.rs:11-74`); the only hit for "auction" in the tree is prose (`vocabulary.rs:1125`). Ours: `Auction` exists as a record type (`src/record/auction.rs:1-15`) but is not a generation input; see `docs/research/auction-to-deal-constraints.md:31` |
| Partially-known deal (known + played cards) | no equivalent: whole deals via `--input-deals`, cards via `predeal` | yes: predeal ∪ played ∪ directly-known, validated | D3 reads whole deals only (`D3/dealer-pbn/src/oneline.rs:1-5`); ours `src/record/deal.rs:1087-1150,1354-1399` |
| Named expression variables | yes, per deal, cached | no (Rust values) | D3 `:177,222-248` |
| Script parameters | `$0`-`$9`, textual, before parsing | no (constructor arguments) | D3 `:250-288` |
| Hand-type mix control (levelling) | yes: measure, compute keeps, rewrite the script | no | D3 `docs/leveling-strategy.md:1-80`, `dealer-level/src/lib.rs:1-35` |
| Output and statistics in the language | `action`, `average`, `frequency`, `csvrpt`, `print*`, `trix` | statistics live in the simulation layer (`src/simulation/core.rs:4-11`) | D3 `:157-191` |

### 1.3 Concrete things their language computes that ours cannot express at all

In the declarative tier, all of the following are absent: suit-restricted HCP,
controls, losing-trick count, honour counts and top-N, C13, suit quality and
CCCC, `hascard` as a predicate, adjustable point scales, cross-seat arithmetic,
ternaries, and `rnd()` (`FILTER_LANGUAGE_STATUS.md:44-123`). Ours has exactly
two per-Hand dimensions — shape set and HCP window — and one derived union bound
(`src/hand_constraint.rs:130-146`).

Three of those are not merely syntax we lack, because a Rust closure does not
recover them either:

- **`rnd()` with a reproducible per-deal stream.** Their stream is seeded from
  the deal, so results do not depend on thread count
  (`FILTER_LANGUAGE_STATUS.md:122-123,237-243`). Our closure form has no
  per-deal seed and no stream contract.
- **DD / par / score as a generation predicate.** A closure could construct a
  solver context per call, but nothing in the tree does, the closure runs before
  the cheap per-seat filters (see §3), and no per-deal memo is reachable through
  `Fn(&Deal) -> bool` (`src/record/deal.rs:1633`).
- **Levelling.** This is not a predicate at all: it measures the hand-type mix
  over a sample, computes per-type keep probabilities, and writes them back into
  the script as `roll`/`levelTheDeal` (`D3/dealer-level/src/lib.rs:1-35`,
  `docs/leveling-strategy.md:1-80`). Nothing in a per-candidate accept/reject
  hook can express a sample-level reweighting.

### 1.4 Concrete things ours does that theirs cannot

- **Known cards richer than predeal.** `with_played` fixes every card in a
  `PlayRecord` to the seat that played it and remembers the played subset
  (`src/record/deal.rs:1087-1123`); `with_known` adds out-of-band information
  (`:1125-1150`); `resolve_known` rejects a card fixed to two seats and more
  than 13 known cards per seat with typed errors (`:1354-1399`). Dealer3 has
  whole deals (`--input-deals`) and `predeal`, but no played-card model: its
  deal readers produce deal-level data only (`D3/dealer-pbn/src/oneline.rs:1-5`,
  `D3/dealer-pbn/src/deal.rs:1-12`) and its grammar has no play or auction
  construct.
- **A conditioning seed validated before sampling.** Ours fails at `build()`
  with `CardAttributedToTwoSeats` / `TooManyKnownForSeat`
  (`src/dealproduction.rs:20-40`, `src/record/deal.rs:1331-1352`); theirs
  validates each `predeal` call for duplicates and the 13-card cap, but has no
  cross-seat conditioning concept to validate (`D3/dealer-core/src/fast_deal.rs:41-70`).
- **DD-shaped analysis, not just DD filters.** Card EV over a distribution of
  Deals consistent with Known cards (`src/simulation/card_ev.rs:4-32`), opening
  leads derived from the same primitive (`src/simulation/lead.rs:6-12`), and a
  contract payoff matrix with denominations and confidence intervals
  (`src/simulation/core.rs:4-11`). Dealer3 reports `average`/`frequency` over
  produced deals; its DD surface is per-deal tricks
  (`FILTER_LANGUAGE_STATUS.md:111-121`).
- **A pure-Rust core with the solver behind a feature** (ADR-0004), so
  `cargo check --no-default-features` builds the domain model with no native
  dependency (`docs/adr/0004-pure-core.md:3-8`). Dealer3 also keeps the solver
  out of `dealer-core`, but the equivalent consumer story is an eight-member
  workspace (`D3/Cargo.toml:1-12`).
- **In-process seams.** Any `Dealer` can be substituted, so the simulations are
  generic over it (`src/simulation/card_ev.rs:418`), and the solver is an
  object-safe factory (`docs/adr/0005-solver-seam.md:1-9`). Dealer3's seams are
  process-level: one crate per concern, one binary per front end.

## 2. Are they faster than us, and why

### 2.1 What their published numbers measure

Their benchmark is deliberately **deals evaluated per second at a fixed deal
count**, not "time to produce N matches": `-g N` makes all programs evaluate the
same number of candidates, `-p` is pinned high so it never binds, and each
sample is checked to confirm the run really generated N deals
(`D3/bench/README.md:159-168`). The `action` block is replaced by a cheap
`average`, so the number is filter throughput rather than output formatting
(`:170-172`). A synthetic corpus entry, `_shuffle_baseline`, has a near-free
condition (`hcp(north) >= 24`, `D3/bench/corpus/_shuffle_baseline.dlr:1-7`), and
its cost per deal is subtracted in nanoseconds to separate generation from
evaluation (`:203-226`).

The corpus is ten PBS scripts plus two synthetic entries, `_shuffle_baseline`
and `_solver_agreement` (`D3/bench/corpus/`); the PBS scripts carry workloads of
650k-3.7M deals each (`D3/bench/corpus.json`, `scripts[*].deals`),
and the heaviest are not small programs: `GIB_1C-P-Resp.dlr` has 202 variable
assignments and conditions like
`hcp(north) + shape(north,5xxx+x5xx)` and
`spades(north)<3 and hcp(north,spades)>0 ? 1 : 0`
(`D3/bench/corpus/GIB_1C-P-Resp.dlr:29-59`).

Their committed result, on an Apple M4 Pro, best of three
(`D3/bench/results/CURRENT.md:3,17-37`):

| quantity | dealer3 `-R 1` | dealer-c | DealerV2_4 |
|---|---|---|---|
| corpus geometric mean | 0.9x dealer-c, 1.3x V2_4 | 1.00 | 0.77x dealer3 |
| per-deal generation | 188 ns | 176 ns | 269 ns |
| per-deal evaluation | 874 ns | 781 ns | 943 ns |
| per-deal total | 1062 ns | 957 ns | 1212 ns |
| threaded (12) | 4.29x over `-R 1` | single-threaded | single-threaded |

"Generation" in that table is their subtractive baseline, so it still includes
the one `hcp()` and comparison of `_shuffle_baseline`; "evaluation" is the
residual (`CURRENT.md:43-48`).

Per-script, single-threaded, they range from 1.2x faster than C on the cheap
scripts (`Scrambling_2NT`, `GIB_1N_Basic`) to 0.5x on the variable-heavy ones
(`Drury`, `GIB_1C-P-Resp`) (`CURRENT.md:20-29`).

### 2.2 The caveats their own documents state

- "A characterization, not a proof" (`bench/README.md:3-5`).
- They stopped reproducing dealer.exe's deal sequence in 0.5.0, so the three
  programs do not deal the same boards; producing N matches is different work
  for each, which is exactly why fixed-work mode exists (`:161-168`).
- Threaded rows vary "by tens of percent" where single-threaded rows vary by
  one or two, and the threaded figure is "a floor rather than a ceiling"
  because the measurement machine was not idle (`:142-157`).
- The sweep is only meaningful above `--min-sweep-seconds`; the harness once
  reported a throughput *regression* past six threads that was pure startup
  noise (`:237-245`).
- `dealer.exe` runs x86-emulated on ARM64, so the "vs exe" column is
  operational, not algorithmic (`:55-81,191-196`).
- The pre-test's own note: Penguin's faster match rate is an RNG difference, and
  only the Rust-vs-C comparison holds the RNG fixed
  (`performance-pretest-2026-01-07.md:15-16,67-78`).

### 2.3 What our numbers measure — and why they cannot be compared directly

Our only committed dealer figures are Criterion medians from
`benches/deal_constraints.rs`, and they time **one produced deal**, i.e. the
whole rejection loop until a deal is accepted
(`docs/performance-optimization.md:127-137`):

| our case | median per *produced* deal |
|---|---|
| unconstrained | 143.53 ns |
| balanced shape, North | 328.10 ns |
| HCP 12-17, North | 488.04 ns |
| balanced 12-14, North | 1.5542 µs |
| cross-hand fit closure | 303.70 ns |

Their 1062 ns is per **candidate evaluated**. The units differ by the rejection
factor, and nothing in our tree records that factor: `attempt_budget` is only
reported on failure (`src/record/deal.rs:41-48,1633-1641`), and the benches call
`dealer.deal()` and discard how many candidates it took
(`benches/deal_constraints.rs:16-25`).

### 2.4 If the arithmetic is forced

This is a hypothesis from published numbers, not a measurement.

- Their generation-only baseline is 188 ns/deal including one `hcp()` and a
  compare (`CURRENT.md:39-48`); our unconstrained deal is 143.5 ns for the same
  four hands with no check. That makes the two generation loops roughly
  comparable single-threaded, with ours perhaps 20-25% cheaper.
- Our own profile of the constrained path attributes 64.97% of samples to
  `Cards::pick_rng::<FastSampler>`, 21.70% to the RNG, and 12.09% to the
  benchmark/dealer closure (`docs/performance-optimization.md:109-114`). The
  fused `summary()` + range + bit-table check (`src/hand.rs:36-45`,
  `src/hand_constraint.rs:298-319`, `src/shape.rs:458-485`) is therefore a
  minority of a ~143 ns candidate.
- The 1.5542 µs "balanced 12-14" case is consistent with ~8-9 candidates at
  ~150-190 ns each, since balanced is roughly half of all hands and 12-14 HCP
  roughly a quarter, and the two correlate.
- Their condition evaluation is 874 ns per candidate, but that is a corpus mean
  over 202-variable scripts with per-suit HCP and several `shape()` calls
  (`GIB_1C-P-Resp.dlr:29-59`), not a shape+HCP pair. On the shared subset —
  whole-hand HCP and ordered shape — their cost per candidate is a `ShapeMask`
  bit test plus a 13-card `hcp()` scan and an AST walk
  (`D3/dealer-core/src/hand.rs:124-126,151-154`), against our two bitboard
  reductions and one table lookup (`src/card.rs:623-637`).

Read that way, **on a single core and on a condition both can state, ours looks
cheaper per candidate**; **on wall-clock generation throughput theirs looks
faster, by the thread count**, because their deals are independent functions of
a seed.

### 2.5 The algorithmic differences that would decide a race on equal terms

1. **What is drawn per candidate.** They start from a sorted 52-byte deck and
   Fisher-Yates it, taking either the no-predeal fast path or a two-phase
   predeal path (`D3/dealer-core/src/fast_deal.rs:83-125,127-145`). We pick
   13-of-52 three times from a bitboard and take the remainder directly, with
   the final seat free (`src/card.rs:261-289`, `src/record/deal.rs:1606-1629`).
   Ours does strictly less work per candidate; their own history corroborates
   how much generation constants matter (removing a per-deal hand sort took
   their shuffle baseline from 534 to 188 ns, `CURRENT.md:41-45,70`).
2. **What is checked per candidate.** Theirs is an AST interpretation with a
   per-deal variable cache and no per-hand precomputation — the `HandStats`
   experiment was reverted because eager stats cost more than they saved
   (`performance-pretest-2026-01-07.md:100-140,183-185`). Ours is one fused
   `(shape, hcp)` summary per Hand and one union-bounded lookup per branch
   (`src/hand_constraint.rs:298-319`). Ours wins on expressible-today
   conditions; theirs wins the moment the condition needs a function we do not
   have.
3. **Acceptance rate per candidate.** Same condition, different shuffle, so
   different candidates and different counts. Their docs are explicit that
   comparing time-to-N-matches is meaningless across programs for this reason
   (`bench/README.md:161-168`). A race must count candidates, not matches.
4. **Threading.** Each of their deals is a pure function of one `u64` seed,
   seeded through SplitMix64 into xoshiro256++ (`D3/dealer-core/src/rng.rs:1-10,48-60`),
   so batches parallelise freely and output is thread-count independent
   (`:7-10`); measured 4.29x on 12 threads (`CURRENT.md:37,76-90`). Our
   generation is deliberately sequential, before any parallel work, so the RNG
   order does not depend on the worker count (`src/simulation/solver.rs:26-30`).
   For a generation-only workload on 12 cores that is a ~5x wall-clock
   difference, and it is architectural rather than micro-optimisation.
5. **DD inside the filter.** They can put `tricks()` in the condition, backed by
   a per-deal solved-cell memo shared across threads
   (`D3/dealer-run/src/dd_demand.rs:1-29`), and a full table costs about 23 ms
   per deal (`D3/dealer-run/src/run.rs:1290-1298`). We never condition
   generation on DD; we solve after generation on a pool, deal-major with
   same-strain contracts adjacent (`docs/adr/0005-solver-seam.md:1-9`). These
   are different algorithms, and no throughput ratio between them means
   anything.

### 2.6 Verdict

**No claim survives as stated.** Their 2.3x figure was theirs against C, it is
stale, and it has already been revised to ~10% slower
(`performance-pretest-2026-01-07.md:255-260`, `bench/README.md:145-149`). Against
us there is no measurement at all. The defensible inferences are narrow: our
per-candidate constant factor is probably smaller on single-threaded
shape+HCP work; their wall-clock generation throughput is probably larger by
roughly the core count; and neither statement has been tested on the same
corpus.

## 3. Copyable performance ideas

Ranked by expected gain per unit of risk. "Already ours" means the idea is
present in our code under a different shape and there is nothing to copy.

| # | Idea | Where they do it | Do we? | Gain | Risk |
|---|---|---|---|---|---|
| 1 | Reject at the earliest check that can decide: per-seat constraints before the user predicate, and a seat's constraint as soon as its cards exist | No automatic equivalent — their order is the script author's, with short-circuit `&&` (`D3/docs/FILTER_LANGUAGE_STATUS.md:146-147`) | **No.** `accept_function` runs before `constraints_respected` (`src/record/deal.rs:1633`), contradicting its own docs ("called after hand constraints are checked", `:1176`; algorithm steps 3 then 4, `:1487-1488`) while satisfying a competing sentence ("called for every candidate deal", `:1182-1185`). `check_cached` is *not* part of this: the four seats are four different `Hand`s, so `check` computes four summaries either way, and `check_cached` has no caller but `check` (`src/hand_constraint.rs:305,308-319`) | Measured in §3.1: 0.40-0.80x on first-seat constraints, 0.99-1.00x on late-seat ones | Low, but it changes the published closure-call contract, so the docs and a test are part of the change |
| 2 | Parallel sampling from stateless per-deal seeds | `D3/dealer-core/src/rng.rs:1-10`, `fast_deal.rs:83-125`, batch dispatch `D3/dealer-run/src/run.rs:486-528` | **No.** Generation is sequential by design (`src/simulation/solver.rs:26-30`); the RNG is the thread RNG and the sampler is stateful per deck | Up to the core count on generation-bound work; measured 4.29x at 12 threads for them (`CURRENT.md:76-90`) | High: changes the RNG contract, the sampler's distribution, and the documented reproducibility invariant; only pays when DD is not dominant |
| 3 | Shape set as a compile-once bit mask | `D3/dealer-core/src/shape.rs:1-41`; braces language expanded to it (`fdshape.rs:1-20`) | **Yes.** `Shapes` is a bit table with a collision-free hash and min/max precheck (`src/shape.rs:354-369,408-440,458-485`) | 0 | — |
| 4 | Inline hand storage, no per-deal allocation, no per-deal sort | `[Card; 13]` inline (`D3/dealer-core/src/hand.rs:11-17`); sort removed for 2.84x on the shuffle baseline (`CURRENT.md:56-74`) | **Yes.** `Hand` is a `u64` bitboard; nothing is sorted or allocated per deal (`src/hand.rs:12-14`, `src/record/deal.rs:1601-1631`) | 0 | — |
| 5 | Final-seat fast path (no sampling decision left) | no explicit equivalent: they always shuffle the deck | **Yes.** `fill == deck.len()` takes the remainder (`src/record/deal.rs:1615-1620`); measured 18.5% on the unconstrained case (`docs/performance-optimization.md:108-129`) | 0 | — |
| 6 | Cache per-script decisions once, not per deal | variable-volatility classification at parse (`D3/dealer-eval/src/lib.rs:687-706`), with a 9.2 s → 0.46 s anecdote for the per-deal version (`:690-696`) | **Not applicable** — we have no expression layer. The generalisable part (hoist anything invariant out of the loop) we already do with `summary()` | 0 today | — |
| 7 | Cheap borrowed hash keys in a per-deal cache | `FxHashMap<&str, i32>` (`D3/dealer-eval/src/lib.rs:341-403`) | **Not applicable** (no cache, no keys) | 0 | — |
| 8 | DD demand analysis: read what the consumer will ask, pre-solve on the pool | `D3/dealer-run/src/dd_demand.rs:1-29` (12.1 s vs 1.9 s on one measurement) | **Partly.** Our simulations already batch every solve they need on a pool, deal-major (`src/simulation/solver.rs:1-30`) | 0 for generation | — |
| 9 | Several deals per shuffle (`-2`/`-3` seat swaps) | `D3/dealer-core/src/swap.rs:1-13` | **No, deliberately.** It produces correlated samples and a run reports six times as many deals as it has shuffles (`:8-12`) | Negative for our statistics; only useful for lead/defence layouts | High: would silently invalidate card-EV and leverage claims |

Two ideas are worth separating from the ranking. Their `-g N` fixed-work mode
and `_shuffle_baseline` are not optimisations but **measurement enablers** that
make idea 2 evaluable; without them a generation race cannot be scored. And the
single largest constant-factor difference is one we cannot copy without
changing meaning: they evaluate an expression tree, we evaluate a compiled
lookup, and the gap is the price of the language, not inefficiency.

### 3.1 Measured: current vs constraints-first vs seat-early

A throwaway harness (release build, `--no-default-features`, pinned to one
core, 200 ms warm-up, ten 300 ms samples, three process runs, median of the
three run medians; spread across runs stayed under 0.3%) compared three orders:

- **current** — closure, then the per-seat constraints (`:1633`);
- **A** — constraints, then the closure;
- **B** — check each constrained seat as it is filled, so a doomed candidate
  never deals the remaining seats; the closure then runs only on survivors.

| scenario | current ns/deal | A | B | B/current |
|---|---:|---:|---:|---:|
| unconstrained | 168.7 | 168.6 | 170.0 | 1.01 |
| NT 15-17, North | 2987.9 | 2975.4 | 1204.7 | **0.40** |
| NT 15-17, South | 2992.2 | 2978.1 | 2973.7 | 0.99 |
| NT 15-17, West | 3001.7 | 2981.7 | 2995.2 | 1.00 |
| weak two + closure, North | 16415.5 | 17170.1 | 7117.1 | **0.43** |
| Polish Club, South | 889.0 | 886.6 | 889.1 | 1.00 |
| Lavazza 3NT + Zar closure, North | 21372.9 | 16614.4 | 6656.7 | **0.31** |
| predeal + HCP, North | 268.0 | 267.2 | 213.2 | **0.80** |
| cross-hand closure only | 867.3 | 867.0 | 871.3 | 1.00 |
| two-seat HCP, N+S | 1394.3 | 1386.3 | 852.2 | **0.61** |

Geometric mean: A/current **0.977**, B/current **0.696**, and **0.484** over the
five scenarios where B can help at all. Three findings matter:

1. **Constraints-first (A) is not universally a win.** It only pays when the
   closure costs more than the constraint check it defers: Lavazza improves
   22%, but the weak-two closure is *cheaper* than the shape+HCP check, so A
   makes that case 4.6% slower. B has no such failure mode because it skips the
   dealing too.
2. **Seat position decides whether B pays.** A constraint on the first seat
   checked gains 60-69%; the same constraint on South or West gains nothing,
   because the seat is filled last (directly, via `take_all`). Reordering to
   fill constrained seats first is the open follow-on.
3. **B is neutral where it cannot help**, including the unconstrained case
   (+0.8%, one `Option` test per seat).

The harness also recorded the rejection factor the committed matrix lacks:
candidates per produced deal were 20.5 for the NT cases, 114 for weak two, 112
for Lavazza, 8.9 for two-seat, 5.7 for Polish Club and the cross-hand closure,
1.6 for predeal + HCP, and 1.0 unconstrained. A 100,000-deal smoke test under B
(N and West constraints, plus a predealt seat) produced no invalid deal.

The change has since landed and the permanent matrix reproduces the result:
criterion medians moved the 1NT constraint from 3,248.3 to 1,221.2 ns on North
(0.38x), 3,280.9 to 3,120.7 on South (0.95x) and 3,283.0 to 3,186.1 on West
(0.97x); shape+HCP 12-14 from 1,679.6 to 702.6 ns (0.42x); geometric mean 0.703
over the twelve cases and 0.657 excluding the two the change cannot help.
`docs/performance-optimization.md` §6 carries the full table.

## 4. Architecture, both directions

### 4.1 Their split

Seven workspace libraries plus the `dealer` binary — eight members in all;
`wasm/` is built outside the workspace (`D3/Cargo.toml:1-16`):

- `dealer-parser`: preprocessor, pest grammar, AST, vocabulary, Dellacherie
  expansion, script parameters (~5.7k lines).
- `dealer-core`: cards, hands, deals, the 560-bit shape mask, xoshiro RNG,
  predeal-aware Fisher-Yates, swap modes.
- `dealer-eval`: the expression interpreter, variable cache and volatility,
  point counts, scoring, a thin DD accessor.
- `dealer-dds`: solved-cell memo (`DealTricks`) shared across threads.
- `dealer-level`: levelling arithmetic and text rewriting, no dealing.
- `dealer-pbn`: deal/oneline/PBN formatting and parsing.
- `dealer-run`: the pass loop, batching, threading, streaming, DD warming.
- `dealer` (binary) + `wasm/`: two front ends over the same core.

### 4.2 Ours

One crate with a feature-gated backend (ADR-0004), and inside it:

- `src/record/deal.rs`: `Dealer` trait, `DealerBuilder`, `StandardDealer`, the
  rejection loop, conditioning validation (~2.2k lines with tests).
- `src/hand_constraint.rs`: the whole "which hands may the dealer accept"
  surface, behind seven public methods.
- `src/shapeparser/` + `src/shape.rs`: pattern scanner/parser/interpreter and
  the compiled bit table.
- `src/simulation/`: card EV, leads, contract payoff, shared pool solver.
- `src/solver/`: the ADR-0005 seam.

**Since this note was written**, ticket 08 of the crate split gave deal
generation its own module, `src/dealer.rs` (`Dealer`, `DealerBuilder`,
`StandardDealer`, `DealerError`, `AcceptFunction`), and deleted the
errors-only `src/dealproduction.rs`; `src/record/deal.rs` now holds only
`Deal`, `DealError`, `Seat` and `Vulnerability`. The paths cited below describe
the tree at the time of writing; see `docs/adr/0008-crate-split.md` for the
target layout.

### 4.3 Where their design is deeper

Depth here means behaviour behind a small interface.

- **`dealer-eval` is the clearest case.** Its interface is essentially
  `eval(&Expr, &EvalContext) -> Result<i32>` (`D3/dealer-run/src/run.rs:1274-1285`);
  behind that sit variables, volatility classification, a deal-scoped cache
  whose lifetime the borrow checker enforces, custom count tables, DD memo
  reads, and a full error taxonomy (`D3/dealer-eval/src/lib.rs:330-403,687-706,1492`;
  `counts.rs:1-37`). Adding a function is one match arm, and every consumer —
  CLI, wasm, report, validator — gets it.
- **`dealer-dds::DealTricks` is leverage.** A solved cell bought by a worker's
  condition is reused by the main thread's action, the `average`, and a `par()`
  report, with the demand analysis deciding which cells to warm
  (`D3/dealer-run/src/dd_demand.rs:1-29`). One purchase, many consumers.
- **`dealer-core::shape` is depth at the smallest scale**: `shape_to_index` +
  `ShapeMask::contains` (nine `u64`s) hides all 560 distributions
  (`D3/dealer-core/src/shape.rs:18-41`), and `fdshape` hides a whole
  alternative syntax behind it (`fdshape.rs:1-20`).
- **`dealer-level`** exists because levelling is a real workflow with its own
  arithmetic, kept out of the run loop and portable to wasm
  (`D3/dealer-level/src/lib.rs:1-35`).

### 4.4 Where ours is deeper

- **`HandConstraint`.** One interface, one construction path, one error type,
  one hot method; behind it the scanner/parser/interpreter pipeline is private,
  HCP is fused into the same branch, the union bound is derived rather than
  exposed, and "all shapes" is a materialised table instead of an enum variant
  with surprising `add` semantics (`src/hand_constraint.rs:4-40,130-146,284-319`).
  Their equivalent (`shape(...) and hcp(...)` as two AST nodes) is more
  expressive but shallower per check: two evaluations, no fusion, no derived
  bound.
- **The `Dealer` seam.** `Dealer` is a trait, so every simulation is generic
  over the dealer and a caller can substitute one (`src/simulation/card_ev.rs:418`;
  `src/record/deal.rs:1442-1478`). Dealer3's run loop knows
  `FastDealGenerator`/`Source` concretely (`D3/dealer-run/src/run.rs:932-937`);
  there is no dealer interface to implement.
- **The solver seam.** Object-safe `SolverContext`, a `ContextFactory` for
  boxed backends, and a sized pool because worker count bounds TT memory
  (`docs/adr/0005-solver-seam.md:1-9`). Theirs is free functions over a
  solved-cell memo; ours is a seam a mock can satisfy.
- **Conditioning.** `DealerBuilder` validates a Known-cards seed structurally
  and reports typed failures (`src/record/deal.rs:1331-1399`);
  `src/dealproduction.rs` exists to keep those failures typed and `source()`-able
  (`:6-13,93-105`). Dealer3 has no analogue because it has no seed.

### 4.5 Seams, leverage, locality

- **Seam placement differs by kind, not by quality.** Theirs are process seams
  (binary, wasm, run), which buy one core behind four front ends; ours are
  in-process trait seams, which buy substitution, mocking and backend choice.
  Ours has more seams inside a process; theirs has more front ends per core.
- **Leverage follows the seam.** Their parser+eval seam means one grammar serves
  CLI, browser, editor highlighting and reference docs generated from
  `vocabulary.rs` (`FILTER_LANGUAGE_STATUS.md:6-24`). Our `HandConstraint` seam
  means one check serves the dealer, the simulation samplers and the legacy
  `HandDescriptor` bridge (`src/hand_constraint.rs:322-360`).
- **Locality is where we lose.** Reading our tree, "deal production" is split
  across `src/dealproduction.rs` (errors only), `src/record/deal.rs` (the
  dealer), `src/hand_constraint.rs` and `src/shapeparser/` — four places with no
  single entry point. Their `parser → eval → run` is one pipeline whose
  interface is the script text. That is a navigational cost, not a depth
  deficiency, and it is already recorded in `SESSION.md`.

### 4.6 What we do better, and should not "fix"

- **Do not split into eight crates to get solver isolation.** A feature flag
  already gives it, and `--no-default-features` builds the domain model with no
  native dependency (`docs/adr/0004-pure-core.md:3-8`). Their split is the price
  of also shipping a CLI, a wasm build and a browser reference from one tree.
  This rule concerns solver isolation only; the six-crate split for dependency
  hygiene is recorded in `docs/adr/0008-crate-split.md`.
- **Do not trade the bitboard for `[Card; 13]`.** Our HCP is four masks and four
  popcounts (`src/card.rs:623-637`); theirs is a 13-card iterator with a match
  per card (`D3/dealer-core/src/hand.rs:124-126`), and their shape index costs
  four more 13-card scans (`:128-154`). Card arrays buy ergonomics we already
  have through `Cards`.
- **Do not turn `HandConstraint` into an expression tree.** The fused check is
  the reason our per-candidate cost is a lookup rather than an interpretation.
  If we ever want their expressiveness, the seam to add is a second, clearly
  separated predicate tier — not a rewrite of the fast tier.
- **Do not parallelise generation casually.** Sequential generation before
  parallel solving is a documented invariant that makes our RNG order
  thread-count independent (`src/simulation/solver.rs:26-30`). Their model gets
  the same property a different way (per-deal seeds); swapping ours without
  deciding the reproducibility contract would "fix" a good decision into a bad
  one.
- **Do not copy swap modes.** Correlation between produced deals is exactly what
  our card-EV and lead statistics assume away (`src/simulation/card_ev.rs:4-32`).

## What this note does not settle

- **Whether we are faster or slower.** No shared corpus, no shared unit (their
  per-candidate deals/sec against our per-produced-deal nanoseconds), and no
  measurement of ours against any script of theirs. Everything in §2.4 is
  arithmetic on published numbers under stated assumptions.
- **What our per-candidate cost actually is.** The §3.1 prototype measured the
  rejection factor for ten scenarios, but nothing in the crate records it:
  `attempt_budget` is only visible on failure, and the bench matrix times
  `deal()`.
- **Whether `FastSampler` is uniform.** Their comparisons assume a uniform draw
  from a seed; ours uses a 64-bit-mask sampler in the dealer hot path
  (`src/record/deal.rs:1621`, `src/card.rs:261-289`) while a uniform sampler
  exists beside it (`:292-310`) and the only uniformity check is a disabled test
  (`tests/uniformity_check.rs:5-6`). Same condition, different candidate
  population, different acceptance rate — a race result would be partly a
  statement about samplers.
- **How their language's cost scales with script size.** The corpus mean (874 ns
  per candidate) is over scripts with up to 205 variables
  (`D3/dealer-eval/src/lib.rs:389-392`); nothing in their
  results separates per-clause cost from per-deal cache cost, and their
  `HandStats` experiment shows the obvious micro-optimisations were not where
  the time was (`performance-pretest-2026-01-07.md:100-185`).
- **Whether the expressiveness gap matters for our users.** Their language
  exists to serve a browser practice-scenario site
  (`D3/docs/FILTER_LANGUAGE_STATUS.md:23-24`); ours is a library whose
  consumers hand-write predicates in Rust (`src/record/deal.rs:1200-1208`).
  Nothing in either tree says which audience we are optimising for.
- **Their threaded numbers on our hardware.** Their 4.29x is an M4 Pro; our
  solver story is rayon with a TT-memory bound (`docs/adr/0005-solver-seam.md:1-9`),
  and the two have never been run side by side.

## Recommended next step

**`/prototype` for a same-corpus race.** It is the only step that answers the
question actually asked, and its hard half already exists on their side: a
committed corpus, a documented fixed-work mode, and a generation-only baseline.
A first race does not need the whole corpus: translate the two or three scripts
that our declarative tier can state (shape plus HCP) and one that needs a
closure, then measure candidates per second against their published per-script
figures. The limit is §1 — we have no `.dlr` evaluator, so anything outside the
shared subset would be measuring a different condition, and a full-corpus race
has to wait for a predicate tier that does not exist. Even the subset decides
the three open items: our per-candidate cost, our throughput on conditions we
did not write, and whether the threading gap is as large as §2.5 suggests.
`/grill-with-docs` would be premature — there is no decision to stress-test
until the number exists, and the question it would grill (do we want a
`.dlr`-like language?) is a design, which this note deliberately does not do.
Dropping it leaves our dealer's throughput unmeasured, and the repo's only
cross-implementation statement a stale, non-comparable pre-test.
