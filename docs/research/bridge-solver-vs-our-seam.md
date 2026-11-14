# bridge-solver as a solver backend: adapter fit, published numbers, and risk

This is a research note, not a design. Every claim below is taken from the tree
and cited by file and line; where the tree is silent the note says so. No
benchmark was run and nothing in this repo was built. The one build that was
run compiled copies under `tmp/probe/`, to test offline resolution; its commands
and output are quoted in "Verdict on the premise".

`docs/research/` did not exist before this note. It is now the home for repo
research notes of this kind (as opposed to ADRs, which stay in `docs/adr/`).

Vocabulary follows `CONTEXT.md`: **Deal** (un-played four hands), **Board**
(Deal + auction/contract/play), **Position** (partially played state), **Solver
context** (stateful handle owning its TT), **TT** (transposition table),
**DD-optimal**.

## Verdict on the premise

> "It is vendorable with no native build step."

**Three claims, three answers: no native build step — true; sources present in
the tree — true, for the exact pinned revisions; builds with `--offline` from
the tree as-is — still false, but only two path patches and two registry
re-pins short, not "not vendorable".**

- `vendor/bridge-solver/build.rs` takes an early return unless `CARGO_FEATURE_DDS_REFERENCE` is set
  (`vendor/bridge-solver/build.rs:16-18`). Only `dds-reference` looks for an
  arm64 `libdds.a`, links it `static=dds` and adds `libc++`
  (`vendor/bridge-solver/build.rs:20-40`); that feature is off by default and is
  not reachable from `bench` (`vendor/bridge-solver/Cargo.toml:23-29`).
- `default = []` (`vendor/bridge-solver/Cargo.toml:12`), but the crate still
  declares two mandatory **git** dependencies, `bridge-types` and
  `bridge-encodings` (`vendor/bridge-solver/Cargo.toml:68-69`), pinned in the
  lockfile to GitHub commits (`vendor/bridge-solver/Cargo.lock:65-89`).
- Both dependencies are now in this repo as untracked nested checkouts,
  `vendor/bridge-types` and `vendor/bridge-encodings` (`git status --porcelain
  vendor` lists both as `??`; `.gitmodules` still names only `vendor/dds`,
  `.gitmodules:1-3`). Every locked revision exists in them
  (`git cat-file -t 57f32241d491` in `bridge-types` and `446bff59650f` in
  `bridge-encodings` both report a commit). The checkouts are 9 and 20 commits
  ahead of those pins, additively, and both still declare `version = "0.1.0"`
  (`vendor/bridge-types/Cargo.toml:3`, `vendor/bridge-encodings/Cargo.toml:3`),
  so a path redirect is version-compatible.
- Nothing in the tree redirects them: there is no `[patch]` and no
  `.cargo/config.toml` in any ancestor (the checkout's `.gitignore` excludes
  `.cargo/`, `vendor/bridge-solver/.gitignore:4`). On a copy of the manifest and
  lock, `cargo metadata --offline` therefore exits 101 at
  `bridge-encodings`' git source. Two `[patch."<source-url>"]` entries pointing
  at the two checkouts take it past both git sources; the key must be the git
  source URL, since `[patch.crates-io]` and a mistyped URL are silent no-ops,
  and a virtual workspace holding all three crates does not help. That only
  works offline because `Cargo.lock` already pins the revisions: with the lock
  removed, Cargo must contact the remotes to choose one even when a patch
  exists.
- After the redirects the only remaining offline failure is registry-side. The
  committed lock pins `quote v1.0.46` and `syn v3.0.3`, neither of which is in
  `~/.cargo/registry/cache`; 39 of the other 41 locked registry crates are,
  including `thiserror`, `clap`, `serde`, `sha2`, `serde_json` and `libc`.
  Re-pinning those two to the cached `quote 1.0.47` / `syn 3.0.6` gives
  `cargo metadata --offline` exit 0 and `cargo build --offline` exit 0, with no
  native toolchain involved.
- `bridge-encodings` is dead weight for a library build — its `use` sites are
  the two `cli`/`bench` binaries, two `play-analysis` examples and the separate
  `wasm` package (`vendor/bridge-solver/src/bin/bridge-solver/main.rs:31`,
  `vendor/bridge-solver/src/bin/solver-bench/main.rs:36`) — but
  `vendor/bridge-solver/Cargo.toml:69` is unconditional, so a default build
  still compiles it, and still needs it redirected.

## 1. Provenance

| Item | Value | Evidence |
|---|---|---|
| Crate version | 2.0.0 | `vendor/bridge-solver/Cargo.toml:3` |
| Checkout HEAD | `19fc199`, described `v2.0.0-1-g19fc199` (one commit past the `v2.0.0` tag `3da1d75`, message "Update absolute paths for move to /Volumes/Express2T") | git metadata in `vendor/bridge-solver` |
| Upstream port | `macroxue/bridge-solver` commit `75b4619` | `vendor/bridge-solver/README.md:125-132` |
| Lock-step target | the reference's 2026-01-31 state, node-for-node on 200 deals; invariant 296,689,028 nodes | `vendor/bridge-solver/CLAUDE.md:70-90` |
| Upstream drift | 49 commits ahead by 2026-08-31, 27 touching `solver.cc` | `vendor/bridge-solver/bench/results/release-profile.md:48-52` |
| Licence | MIT OR Apache-2.0; original C++ by Hanhong Xue, Rust by Rick Wilson | `vendor/bridge-solver/Cargo.toml:5`, `vendor/bridge-solver/LICENSE-MIT:1-3` |
| Upstream licence | same MIT/Apache pairing (adopted upstream in `dc2d4df`) | `vendor/bridge-solver/README.md:134-137` |

The Rust port is not a thin binding: it reimplements the search in Rust and adds
par scoring, play analysis, PBN handling, wasm and a web app
(`vendor/bridge-solver/README.md:125-131`). `vendor/bridge-solver/src/cache.rs`
is a stub — one line, `//\! Transposition table and caching`
(`vendor/bridge-solver/src/cache.rs:1`) — even though
`vendor/bridge-solver/src/lib.rs:30` declares `mod cache;`; the real caches
live in `vendor/bridge-solver/src/search.rs:70` and
`vendor/bridge-solver/src/pattern.rs:354`.

## 2. Adapter fit against the seam

The seam is `Solver`/`SolverContext`/`ContextFactory` plus three cold traits
(`src/solver/mod.rs:156-294`). The reference adapter is `src/solver/dds.rs`.

### Method-by-method

| Seam method | bridge-solver API | Conversion needed | Verdict |
|---|---|---|---|
| `solve(position, mode, solutions)` | `Solver::new(Hands, trump, leader)` (`vendor/bridge-solver/src/bridge_solver.rs:596-604`) + `solve_with_caches(&mut CutoffCache, &mut PatternCache) -> u8` (`vendor/bridge-solver/src/bridge_solver.rs:648-654`) | `Position` → `Hands` + trump + leader; result is **NS tricks only**, no card identity | **Fights the interface**: cannot return `CardSolve` at all without enumerating and re-solving |
| `solve_tricks(position)` | same call; `u8` is NS tricks | `Hands` from the remaining cards; `trump` maps 1:1 (`vendor/bridge-solver/src/types.rs:3-10` vs `src/contract.rs:112-118`); leader is the trick leader only for a fresh trick — `new_mid_trick` derives it from the `PartialTrick` (`vendor/bridge-solver/src/bridge_solver.rs:618-638`); convert NS → side-to-play as `ns` if the side is NS else `remaining - ns` (the rule the crate uses at `vendor/bridge-solver/src/analyse_play.rs:31-33,194-195`) | **Fits behind a shim** (~30 lines) |
| `solve_all_cards(position)` | no card-returning API; per-card evaluation exists only internally (`ns_deal_total_after`, `vendor/bridge-solver/src/analyse_play.rs:755-769`) or via `node_alternatives` (`vendor/bridge-solver/src/analyse_play.rs:266-371`) or `PartialTrick` + `solve_mid_trick` (`vendor/bridge-solver/src/bridge_solver.rs:618-638,691-698`) | `Position` → `PlayInput` with a fabricated `plays` prefix, table cards re-inserted, then one solve per legal card; no class is returned | **Fights the interface**: N cold solves with N cache pairs, no TT reuse, and `node_alternatives` miscomputes `total` for a mid-trick input (see below) |
| `clear_tt()` | `CutoffCache::reset` (`vendor/bridge-solver/src/search.rs:143-153`) and `PatternCache::reset` (`vendor/bridge-solver/src/pattern.rs:396-424`) | none; two calls | **Fits as-is** behind a two-line shim |
| `reset()` | no per-solve state other than the caches (`Solver` is immutable: `vendor/bridge-solver/src/bridge_solver.rs:507-512`) | none | **Fits as a no-op**; the seam's distinction between "clear TT" and "reset per-solve state" has no counterpart |
| `dd_table(deal)` | `solve_dd_table(&bridge_types::Deal) -> DdTable` (`vendor/bridge-solver/src/par.rs:61-63`) | our `Deal` → `bridge_types::Deal`; `DdTable` → our table type | **Fits behind a shim**, but see the conversion caveat below |
| `par(table, vulnerability)` | `par(&DdTable, vul_ns: bool, vul_ew: bool) -> ParResult` (`vendor/bridge-solver/src/par.rs:543-547`) | `Vulnerability::{NS,EW,All}` → two bools (`src/record/deal.rs:699-709`); `ParResult` → our par type | **Fits behind a shim** |
| `analyse_play(board)` | `running_trace(&PlayInput, &[String], &HashMap<String,u8>) -> Tier1Output` (`vendor/bridge-solver/src/analyse_play.rs:186-190`) | full un-played `Hands` + trump + declarer + leader + plays; returns per-card **costs**, not per-play DD totals | **Fits behind a shim**: ~40 lines for the inversion, plus `Deal` → `Hands` card by card and plays → indices, and a caller-owned key cache |

Card/seat encoding conversions required throughout:

| Concept | Ours | Theirs | Conversion |
|---|---|---|---|
| Suit | `Spades=0, Hearts=1, Diamonds=2, Clubs=3` (`src/shape.rs:902-911`) | `SPADE=0, HEART=1, DIAMOND=2, CLUB=3` (`vendor/bridge-solver/src/types.rs:3-8`) | identity |
| Strain | `NoTrumps=4` (`src/contract.rs:112-118`) | `NOTRUMP=4` (`vendor/bridge-solver/src/types.rs:10`) | identity |
| Rank | `2..=14` (`src/solver/dds.rs:217-220`) | `TWO=0 .. ACE=12` (`vendor/bridge-solver/src/types.rs:12-27`) | `rank - 2` (exactly what `vendor/bridge-solver/src/convert.rs:60-62` does) |
| Seat | `North=0, East=1, South=2, West=3` (`src/record/deal.rs:390-399`) | `WEST=0, NORTH=1, EAST=2, SOUTH=3` (`vendor/bridge-solver/src/types.rs:29-34`) | permutation, not identity |
| Card | `Card { suit, rank }` | `usize` 0..51, suit-major, rank offset inverted (`vendor/bridge-solver/src/cards.rs:15-29`) | via `card_of(suit, rank)` (`vendor/bridge-solver/src/convert.rs:39-43`) |

One further encoding boundary sits behind `dd_table`, in `bridge-types`, and it is
the one most likely to be got wrong: that crate orders suits the **opposite way**
from both us and `bridge-solver` — `Clubs = 0 … Spades = 3`
(`vendor/bridge-types/src/card.rs:7-12`) against our descending
`Spades = 0 … Clubs = 3` (`src/shape.rs:899-911`). A cast across that boundary
swaps spades with clubs and hearts with diamonds while leaving `NT = 4` correct
on both sides, and it also inverts the derived `Ord`. Seats do agree
(`North = 0 … West = 3`, `vendor/bridge-types/src/direction.rs:8-13` vs
`src/record/deal.rs:390-399`), so the damage is confined to suits — which is
exactly the kind that survives review.

### Which of these eight our code actually calls

The table above grades all eight methods, but only five have any call site at
all, and only three run in production:

| Seam method | Production call site | Load-bearing? |
|---|---|---|
| `solve_tricks` | `src/simulation/solver.rs:126`, inside the per-deal pool job that drives every contract-payoff solve (`src/simulation/payoff.rs:576,595`) | yes, hot |
| `solve_all_cards` | `src/simulation/solver.rs:152`, one call per deal, reached by card EV (`src/simulation/card_ev.rs:436,454`) and indirectly by lead simulation (`src/simulation/lead.rs:621,630`) | yes, hot |
| `analyse_play` | `src/performance_analysis.rs:760`, the only production call, through the only impl (`src/solver/dds.rs:120-178`) | yes, cold |
| `solve` | none: only the two trait defaults invoke it (`src/solver/mod.rs:180,192`) | via the defaults only |
| `clear_tt`, `reset` | none: the only calls are the assertion-free smoke test at `src/solver/dds.rs:390-394` | no |
| `dd_table`, `par` | none, and no implementor: not in `src/`, and `dds/src/par.rs:11-61` declares its own `ParCalculator` with no impl | no |

Two consequences the fit table does not show. First, of the knobs, only
`SolveMode::AutoReuse` and two of the three `Solutions` levels are ever used:
`Solutions::AllDdOptimal` and `SolveMode::ForceReuseAlwaysSearch` appear only at
their declarations and in the DDS mapping (`src/solver/mod.rs:62,77`,
`src/solver/dds.rs:185,198`). Second, the hot path depends on implicit reuse, not
on `clear_tt`: `AutoReuse` is passed on every solve and the payoff engine sorts
contracts by strain so same-strain solves stay adjacent on one context
(`src/simulation/payoff.rs:590-599`), which is the deal-major requirement of
ADR-0005 asserted by `contract_payoff_is_deal_major_with_same_strain_adjacent`
(`src/simulation/payoff.rs:1107-1165`).

What a replacement backend must satisfy is pinned by tests, not by the trait
docs: full physical equivalence classes (`src/solver/dds.rs:292-326`), the
declarer complement applied by callers (`src/simulation/card_ev.rs:682-714`),
`n+1` declarer-relative play-analysis totals (`src/solver/dds.rs:397-406`, and
`analyze_hand` panics on a short vector at
`src/performance_analysis.rs:808-811,826-830`), and context reuse across a deal
with same-strain adjacency (`src/simulation/payoff.rs:1107-1165`).

### Threading, `Send`, and object safety

- Their `Solver` is `{ Hands, trump, initial_leader, num_tricks }` and derives
  nothing; `Hands` is a `[Cards; 4]` of `u64` bitboards
  (`vendor/bridge-solver/src/hands.rs:10-13`, `vendor/bridge-solver/src/cards.rs:60-62`), so `Solver` is `Send + Sync`.
- The caches are the stateful part. `CutoffCache` is a `Box<[CutoffEntry]>`
  (`vendor/bridge-solver/src/search.rs:70-79`), `PatternCache` a `Box<[ShapeEntry]>`
  (`vendor/bridge-solver/src/pattern.rs:354-362`), and `ShapeEntry` contains a `Pattern` containing
  a `PatternVec` (`vendor/bridge-solver/src/pattern.rs:121-125,305-308`). `PatternVec` is
  `unsafe impl Send` but deliberately **not `Sync`**, because its free lists are
  thread-local (`vendor/bridge-solver/src/pattern_vec.rs:45-50,343-346`). A context wrapping the two
  caches is therefore `Send` but not `Sync`.
- The seam requires only `SolverContext: Send` (`src/solver/mod.rs:156`), so
  that is satisfied. `ContextFactory: Send + Sync` (`src/solver/mod.rs:207`) is
  satisfied by a factory that owns no caches. No object-safety problem: the
  trait methods are non-generic (`src/solver/mod.rs:167-199,207-210`), and the
  bridge-solver entry points the adapter would call are non-generic free
  functions (`vendor/bridge-solver/src/par.rs:61-97`) or methods on a concrete type
  (`vendor/bridge-solver/src/bridge_solver.rs:641-698`).
- Wrinkle: `PatternVec`'s `Drop` returns blocks to the *dropping thread's* pool
  (`vendor/bridge-solver/src/pattern_vec.rs:292-300`), so moving a context between workers leaks
  retention onto whichever thread drops it; the pool never shrinks on its own
  (`vendor/bridge-solver/src/pattern_vec.rs:52-60`). `drain_pool` is the escape hatch
  (`vendor/bridge-solver/src/lib.rs:57`).
- Wrinkle: the debug knobs are process-global atomics, not context state —
  `XRAY_*`, `NO_PRUNING`, `NO_TT`, `NO_RANK_SKIP`, `SHOW_PERF`
  (`vendor/bridge-solver/src/bridge_solver.rs:525-535`) with setters (`vendor/bridge-solver/src/bridge_solver.rs:543-592`)
  — while the node counter is a `thread_local` (`vendor/bridge-solver/src/bridge_solver.rs:516-523`).
  DDS's seam keeps all such state in the context.

### Per-context state the seam has no slot for

The seam's context is one TT (`src/solver/mod.rs:151-156`). This backend needs:

1. a `CutoffCache` (move-ordering cache) and a `PatternCache` (bounds tree),
   both caller-owned and passed per solve (`vendor/bridge-solver/src/bridge_solver.rs:648-654`);
2. the `TableSolver` wrapper that owns one pair and resets it per strain
   (`vendor/bridge-solver/src/par.rs:124-127,233-269`), or a `thread_local` shared one for the free
   table functions (`vendor/bridge-solver/src/par.rs:272-293`);
3. for play analysis, a prefix-key list and a `HashMap` position cache supplied
   by the caller (`vendor/bridge-solver/src/analyse_play.rs:186-190,880-915`), plus a fresh cache
   pair per position solve because the pattern cache has no partial-trick
   component (`vendor/bridge-solver/src/analyse_play.rs:721-751`).

`CutoffCache::new(16)` alone is "a megabyte" (`vendor/bridge-solver/src/par.rs:104-110`), so a
context-per-job design like ours (`src/solver/pool.rs:24-38`, where a context is
created per rayon *job*, not per worker) would pay that construction per job.
The MTD(f) seed is a per-call argument, not stored state
(`vendor/bridge-solver/src/bridge_solver.rs:665-677,756-800`), so "mtdf state" is not an extra slot.

### TT, reuse, and `SolveMode`

- Reuse is automatic and content-keyed: the caches are hash tables probed by
  shape (`vendor/bridge-solver/src/search.rs:175-196`, `vendor/bridge-solver/src/pattern.rs:328-341`). There is no mode
  argument on any solve entry point (`vendor/bridge-solver/src/bridge_solver.rs:641-698`).
- `reset` empties entries while keeping capacity, and the docs explain why size
  cannot change answers (`vendor/bridge-solver/src/search.rs:121-153`, `vendor/bridge-solver/src/pattern.rs:371-395`).
- Reuse is **not** safe merely because positions are "similar":
  `vendor/bridge-solver/src/par.rs:240-245` resets both caches per strain, and
  `vendor/bridge-solver/src/analyse_play.rs:721-729` warns that `PatternCache`
  is keyed only on `(shape, seat_to_play)` with no trump or partial-trick
  component, so sharing it across those corrupts results.

Consequence: `SolveMode::AutoReuse` and `SolveMode::ForceReuseAlwaysSearch`
(`src/solver/mod.rs:48-63`) have no counterpart, and
`ForceReuseAlwaysSearch` is worse than meaningless here — the seam's contract
("the caller guarantees the positions are similar enough", `src/solver/mod.rs:58-62`)
is weaker than what this backend requires (reset per trump; never share a
pattern cache across mid-trick states). An adapter would have to enforce the
reset discipline itself, which is precisely what ADR-0005 relies on DDS doing
(`docs/adr/0005-solver-seam.md:17-21`).

### Termination and errors

There is no error type on the solve path: `solve_with_caches` returns `u8`
(`vendor/bridge-solver/src/bridge_solver.rs:648-654`), and the MTD(f) driver returns
`(u8, u64)` (`vendor/bridge-solver/src/bridge_solver.rs:756-800`). Invalid input is rejected by
types/parsers (`Hands::from_pbn -> Option`, `vendor/bridge-solver/src/hands.rs:76-106`;
`new_mid_trick -> Option`, `vendor/bridge-solver/src/bridge_solver.rs:618-621`), and internal
invariant violations `assert!`/`expect` (`vendor/bridge-solver/src/pattern_vec.rs:242`,
`vendor/bridge-solver/src/bridge_solver.rs:487`). Play analysis has `PlayError`
(`vendor/bridge-solver/src/analyse_play.rs:135-152`).

| `SolverError` (`src/solver/mod.rs:124-134`) | Counterpart | Note |
|---|---|---|
| `ContextCreation` | **none** | Constructors are infallible (`vendor/bridge-solver/src/bridge_solver.rs:596-604`, `vendor/bridge-solver/src/search.rs:110-119`); failure is OOM/abort |
| `Backend(String)` | **none** | No status enum; a broken invariant panics rather than returning |
| `Position(String)` | **partial** | `Hands::from_pbn -> Option` (`vendor/bridge-solver/src/hands.rs:76`), `new_mid_trick -> Option` (`vendor/bridge-solver/src/bridge_solver.rs:618-621`), `PlayError::{NotHeld,Revoke,NodeOutOfRange}` (`vendor/bridge-solver/src/analyse_play.rs:136-152`) |
| `NoCards` | **none** | A solve always returns a number; "no legal cards" is not representable |

### `Solutions` and equivalence classes

- `Solutions::OneDdOptimal` expects *one card* (`src/solver/mod.rs:72-75`).
  Their solve returns one *trick count* (`vendor/bridge-solver/src/bridge_solver.rs:641-654`), so
  even this level needs a card-enumeration shim to name a card.
- `Solutions::AllDdOptimal`/`AllLegal` expect per-card tricks and equivalence
  classes (`src/solver/mod.rs:76-80,88-96`). Their nearest public analogue is
  `node_alternatives`, which returns `Vec<Alternative { card, tricks, cost }>`
  for every legal card (`vendor/bridge-solver/src/analyse_play.rs:69-90,266-371`). It collapses
  touching ranks **only when leading** (`vendor/bridge-solver/src/analyse_play.rs:305-329`);
  when following, every legal card is its own class and gets its own solve
  (`vendor/bridge-solver/src/analyse_play.rs:311-316`). Either way the output
  lists each concrete card, with **no exposed class**.
- Equivalence itself is a private helper (`is_equivalent`,
  `vendor/bridge-solver/src/analyse_play.rs:788-813`), also private inside the engine's search
  (`vendor/bridge-solver/src/search.rs:1422`), and not exported (`vendor/bridge-solver/src/lib.rs:41-61`). It
  reasons only about cards still in hand, so it is valid only on an empty
  trick; even exported, it would not reproduce DDS's `equals` for a follow.
  Our `CardSolve::equivalents` (`src/solver/mod.rs:89-96`) is expanded from DDS's
  `equals` bitmask (`src/solver/dds.rs:221-242`); there is nothing to expand here.
- No batch/`CalcAllTables` analogue: tables are one deal at a time
  (`vendor/bridge-solver/src/par.rs:61-97`); parallelism is the caller's job.

### `analyse_play` shape

- Our trait: `&Board -> Result<Vec<u8>, Error>`, one DD trick count per play
  plus the starting Position, length `n+1`, declarer-relative
  (`src/solver/mod.rs:273-294`).
- Theirs: free function, feature-gated (`vendor/bridge-solver/src/lib.rs:27-28`), taking
  `PlayInput { hands, trump, declarer, leader, plays }`
  (`vendor/bridge-solver/src/analyse_play.rs:122-132`) and returning `Tier1Output { contract_tricks,
  trace: Vec<TraceEntry>, new_entries }` (`vendor/bridge-solver/src/analyse_play.rs:94-101`), where
  each `TraceEntry` is `{ index, seat, card, cost }` (`vendor/bridge-solver/src/analyse_play.rs:56-66`).
  It also returns `Vec<(String,u8)>` cache entries, which we have no slot for.
- Preconditions: a partial *play* is legal (`vendor/bridge-solver/src/analyse_play.rs:130-131`),
  but `hands` must be the full un-played Deal — the replay always starts from
  an empty `PartialTrick` (`vendor/bridge-solver/src/analyse_play.rs:197-199`). `keys`
  must have exactly `plays.len() + 1` entries: `keys[0]` and `keys[k + 1]` are
  indexed with no length check, so a short slice panics
  (`vendor/bridge-solver/src/analyse_play.rs:204,234,239`).
- Length is compatible — `trace.len() == plays.len()`
  (`vendor/bridge-solver/src/analyse_play.rs:214-256`), and `contract_tricks` is
  `V_0`, the value of the opening-lead position, which is our first vector entry.
  Reconstruction is exact because cost is defined one-sided: declaring side
  `cost = max(0, V_k - V_{k+1})`, defenders `cost = max(0, V_{k+1} - V_k)`
  (`vendor/bridge-solver/src/analyse_play.rs:15-18,243-248`), so
  `V_{k+1} = V_k - cost` or `+ cost` according to the seat. Nothing is lost:
  `V_node` is the best value over the node's legal cards
  (`vendor/bridge-solver/src/analyse_play.rs:338-343`), so the clamped
  `saturating_sub` can never actually clamp; a forced node carries `V` over
  (`vendor/bridge-solver/src/analyse_play.rs:232-234`); and the final trick is
  either carried over or solved mid-trick
  (`vendor/bridge-solver/src/analyse_play.rs:737-750`). The crate states the
  cost/value identity in the `optimal_line` section
  (`vendor/bridge-solver/src/analyse_play.rs:438-444`); it never spells out the
  reconstruction recipe.
- The output shape is closer to ours than it first looks: their `cost` is
  literally our `dd_tricks_lost`. `evaluate_play` computes
  `|prev - curr|` (`src/performance_analysis.rs:690-711`), which equals the
  one-sided cost for either side, and our `dd_tricks` is the running total from
  the opening value. So the trick vector and the per-play deviation can both be
  derived without re-typing the analysis layer.
- The richer API is `node_alternatives` (per-card `tricks` and `cost`) and
  `optimal_line` (a DD-perfect continuation, `vendor/bridge-solver/src/analyse_play.rs:389-542`);
  neither has a counterpart in our seam or in `performance_analysis`.

### Where `node_alternatives` cannot stand in for `solve_all_cards`

The fit table's "one search becomes N" understates the obstacle, and the
`Alternative` shape hides a defect.

- It needs a non-empty `plays` prefix: `node >= input.plays.len()` is
  `NodeOutOfRange` (`vendor/bridge-solver/src/analyse_play.rs:267-272`), so a
  `Position` at the opening lead has no node to ask about and one has to be
  fabricated.
- It always replays from an empty trick (`vendor/bridge-solver/src/analyse_play.rs:281-283`),
  so a `Position` that is mid-trick — the case our `Position` explicitly
  represents (`src/record/position.rs:178-183`) — has to have the table cards
  re-inserted by the caller.
- `total` is `input.hands.num_tricks()`, which `Hands` defines as
  `hands[WEST].size()` (`vendor/bridge-solver/src/analyse_play.rs:276`,
  `vendor/bridge-solver/src/hands.rs:58-60`) — not the largest hand. On a
  mid-trick or partially played input where West has already played, every
  `tricks` value is computed against a short `total`, and `total - ns` for an
  EW declarer can underflow. `running_trace` and `optimal_line` share the same
  expression (`vendor/bridge-solver/src/analyse_play.rs:193,399`); it is
  harmless only because they are documented to receive a full un-played Deal.
- Cost: one cold solve per legal card (per class when leading), each allocating
  its own ~1 MB `CutoffCache` pair with no reuse across cards
  (`vendor/bridge-solver/src/analyse_play.rs:331-336,741-742`;
  `vendor/bridge-solver/src/par.rs:104-110`), against DDS's single
  `solve_board` with `solutions = 3` on an existing context
  (`src/solver/dds.rs:182-188`).

### `dd_table` and `par`

Both exist, and **par is derived from a DD table**, not solved independently:
`par(&DdTable, vul_ns, vul_ew)` runs a bounded search over the 35 contracts
using duplicate scoring (`vendor/bridge-solver/src/par.rs:1-6,543`). The table itself is 20
cells (`solve_dd_table`, `vendor/bridge-solver/src/par.rs:61-63`) and is infallible.

`bridge_types::Deal` is not an obstacle, and the earlier PBN-round-trip caveat is
now obsolete: the four fields are public (`vendor/bridge-types/src/deal.rs:10-15`)
and `Hand::from_cards` is total (`vendor/bridge-types/src/hand.rs:18-21`), so our
`Deal` converts in memory. Nothing validates the result — the crate has no error
type at all (`rg "pub enum .*Error"` over `vendor/bridge-types/src` returns
nothing) and its opt-in checks are total-count and duplicate-only
(`is_complete` is 52 cards, `is_valid` is no duplicates,
`vendor/bridge-types/src/deal.rs:109-122`), not 13 cards per seat.

The table boundary is the cleanest part of the adapter: `solve_dd_table` returns
`bridge_types::DdTable` directly (`vendor/bridge-solver/src/par.rs:60-63`), whose
layout is private and read only through `tricks(declarer, strain)` / `set`
(`vendor/bridge-types/src/dd_table.rs:52-55,127,135`); `par` reads it the same way
(`vendor/bridge-solver/src/par.rs:52-58,543`). Neither needs reindexing. Its axes
are declarer-major N,E,S,W over C,D,H,S,NT
(`vendor/bridge-types/src/dd_table.rs:7-23`) — a third order, different on both
axes from the corpus string (N,S,E,W × NT,S,H,D,C,
`vendor/bridge-solver/bench/corpus.json:4`) and different on the outer axis from
DDS's `[strain][hand]` table. The crate's own doc names the three row orders that
were in circulation before it, which is the reason the layout is private and the
accessors take a seat and a strain (`vendor/bridge-types/src/dd_table.rs:27-45`).

Note also that our seam declares `DdTableCalculator` and `ParCalculator`
(`src/solver/mod.rs:234-267`) but **no backend implements either** — the only
`PlayAnalyser` impl is DDS's (`src/solver/dds.rs:120-178`). The `dds` crate's
own `DdTableCalculator` is a different trait with generic methods
(`dds/src/tables.rs:31-52,55`).

## 3. Published performance evidence

Their harness is `vendor/bridge-solver/bench/` plus `solver-bench`, gated behind `bench`
(`vendor/bridge-solver/Cargo.toml:20-23,49-52`); `dds-reference` adds in-process
DDS 2.9 (`vendor/bridge-solver/Cargo.toml:25-29`,
`vendor/bridge-solver/src/bin/solver-bench/dds.rs:1-9`).

Corpora, exactly:

| Name | Size | Where |
|---|---|---|
| `corpus-v1` | 10 real boards from `vendor/bridge-solver/fixtures/bench-boards.lin` | `vendor/bridge-solver/bench/corpus.json:1-11`; `vendor/bridge-solver/bench/README.md:159-170` |
| lock-step corpus | 200 PBN deals, no auction/play | `vendor/bridge-solver/fixtures/divergence/lockstep-200.pbn`; `vendor/bridge-solver/CLAUDE.md:70-90` |
| random corpora | 2,000 / 500 / 300 / 240 deals, generated from seeds by `vendor/bridge-solver/bench/comparison/gen-corpus.py` | `vendor/bridge-solver/bench/comparison/METHODOLOGY.md:66-83`; `vendor/bridge-solver/bench/results/release-profile.md:1030-1033` |
| club session | 27 boards | `vendor/bridge-solver/bench/comparison/RESULTS.md:93-95` |

Machine for every number: Apple M4 Pro, 12 logical (8P+4E)
(`vendor/bridge-solver/bench/results/release-profile.md:3-4`; `vendor/bridge-solver/bench/comparison/RESULTS.md:1-3`). Build:
`cargo build --release` with `lto = "fat"`, `codegen-units = 1`
(`vendor/bridge-solver/Cargo.toml:80-82`), no `target-cpu` override — the doc records
`-C target-cpu=native` as neutral on this target
(`vendor/bridge-solver/bench/results/release-profile.md:1157-1159`). DDS, when linked, is **2.9**, built with
`Makefile_Mac_clang_static` (`-O3 -flto`), STL threading
(`vendor/bridge-solver/bench/comparison/METHODOLOGY.md:9-13,130-134`), on arm64 (`vendor/bridge-solver/bench/comparison/RESULTS.md:182-190`).

| Claim | Number | Corpus | Threads | `dds-reference` linked? | Source |
|---|---|---|---|---|---|
| full 20-cell table, typical deal | ~14 ms across 12 cores; ~110 ms one thread | corpus-v1 | 12 / 1 | no | `vendor/bridge-solver/README.md:37-40`; `vendor/bridge-solver/docs/cli-guide.md:325-329` |
| ten-board single-core range | 15–263 ms | corpus-v1 | 1 | no | `vendor/bridge-solver/README.md:39-40`; `vendor/bridge-solver/docs/cli-guide.md:325-327` |
| 200-board annotation | 22.55 s → 2.75 s, 8.20x | 200 boards (unnamed) | 1,2,4,8,12 | no | `vendor/bridge-solver/docs/cli-guide.md:206-214`; `vendor/bridge-solver/README.md:42-44` |
| worst freak deal | 39.7 s, 1.6 GB peak | `deals/freak/deal.1` | 1 | no | `vendor/bridge-solver/README.md:47-50`; `vendor/bridge-solver/bench/results/release-profile.md:131-134` |
| vs C++ reference and DDS 2.9 | 23.09 s vs 21.64 s / 20.25 s → 1.067x ref, 1.14x DDS | lockstep-200 | 1 | **yes**, in-process | `vendor/bridge-solver/bench/results/release-profile.md:1047-1059`; `vendor/bridge-solver/README.md:52-54` |
| per-node gap vs C++ | 1.127x, identical 296,689,028 nodes | lockstep-200 | 1 | no (C++ only) | `vendor/bridge-solver/bench/results/release-profile.md:204-222` |
| vs DDS, curated corpus | per-core geomean 1.06x; corpus total 1.22x; per-board 0.73x–1.76x | corpus-v1 | 1 | **yes** | `vendor/bridge-solver/bench/results/release-profile.md:692-715` |
| vs DDS, random throughput | 1.12x at 1 thread → 1.10x at 12 | 240 generated | 1..12 | **yes** | `vendor/bridge-solver/bench/results/release-profile.md:726-741` |
| vs DDS, 500 random deals | 57.1 s vs 52.9–53.3 s → 1.08x slower | 500 seeded | 1 | **yes** | `vendor/bridge-solver/bench/results/release-profile.md:1030-1045` |
| one board, latency | p50 78.7 vs 67.1 ms (1.17x); p99 637.6 vs 703.2 ms (0.91x) | 2,000 random | 1 | **yes** | `vendor/bridge-solver/bench/comparison/RESULTS.md:26-38` |
| one event (27 boards) | 10.2/37.1/70.8/77.6 deals/s vs DDS 12.4/46.6/90.1/98.4 | 27 club boards | 1,4,8,12 | **yes** | `vendor/bridge-solver/bench/comparison/RESULTS.md:95-103` |
| one file (500 deals) | 8.8/31.8/60.6/71.0 deals/s vs DDS 9.3/34.7/61.3/67.3 | 500 random | 1,4,8,12 | **yes** | `vendor/bridge-solver/bench/comparison/RESULTS.md:137-142` |
| wasm | ~250 ms table; ~128 ms per-card vs ~90 ms native | not named | 1 | no | `vendor/bridge-solver/README.md:93-94`; `vendor/bridge-solver/docs/ROADMAP.md:24-29` |

### Numbers that are self-comparison, not comparison to DDS

These are vN against vN−1 on corpus-v1 (10 boards) or lockstep-200, on the same
M4 Pro, with no DDS linked. They say how the port improved, not how it compares.

| Change | Published effect | Source |
|---|---|---|
| `lto = "fat"` + `codegen-units = 1` | −7.7% wall and CPU | `vendor/bridge-solver/bench/results/release-profile.md:26-44` |
| bounds-cache correctness fix (#14) | faster than the code it replaced | `vendor/bridge-solver/bench/results/cache-fix.md:1-12` |
| MTD(f) seed carry-forward | self A/B, corpus-v1 | `vendor/bridge-solver/bench/results/mtdf-seed.md:1-12` |
| cache reuse instead of rebuild | self A/B, corpus-v1 | `vendor/bridge-solver/bench/results/cache-reuse.md:1-14` |
| node counter atomic removal | 12-thread speedup 1.27x → 8.07x | `vendor/bridge-solver/bench/results/BASELINE.md:58-74` |
| `PatternCache` probing | 4.5% fewer nodes, no time change | `vendor/bridge-solver/bench/results/release-profile.md:605-634` |
| PGO | −7.50% instructions, −1.69% geomean wall | `vendor/bridge-solver/bench/results/release-profile.md:1216-1221` |

`vendor/bridge-solver/bench/results/BASELINE.md` says outright "No external reference. DDS is not measured here"
(`vendor/bridge-solver/bench/results/BASELINE.md:78-81`). Every recorded `.json` carries a `-dirty`
revision (`vendor/bridge-solver/bench/results/release-profile.json:3`,
`vendor/bridge-solver/bench/results/cache-fix.json` etc.), although the checkout is currently clean.

## 4. Comparability with our harness

Ours measure a whole simulation, not a solver call:

| Bench | What is timed | Backend | Deals | Metric | Threads |
|---|---|---|---|---|---|
| `benches/bench_payoff.rs` | `ContractPayoffSimulation::run()` | real `DdsSolver` (`src/simulation/payoff.rs:567`) | 1,000, 4 contracts, one predealt seat (`benches/bench_payoff.rs:16-30`) | criterion wall-clock per run, sample 10 (`benches/bench_payoff.rs:38-42`) | `available_parallelism` (`src/solver/pool.rs:66-69`) |
| `benches/bench_leadsim.rs` | `LeadSimulation::run()` | real `DdsSolver` (`src/simulation/lead.rs:609`) | 1,000, 1 contract, predealt South + constraints (`benches/bench_leadsim.rs:8-24`) | criterion wall-clock, sample 10 (`benches/bench_leadsim.rs:33-37`) | `available_parallelism` |
| `benches/simulation_paths.rs` | card-EV / payoff / lead with a mock context | `FixedContext` (`benches/simulation_paths.rs:8-40`) | 1,000 | criterion wall-clock | `available_parallelism` |

The unit of parallel work is a Deal, and a context is created per rayon *job*,
not per worker; `workers × table capacity` bounds instantaneous memory
(`src/solver/pool.rs:24-38`). Our benches therefore include Deal generation and
post-processing, and they never fix the worker count. The DDS context is
`dds_c_create_solvercontext_default` (`src/solver/dds.rs:44`); its TT capacity is
**not stated anywhere in the tree**.

They cannot go in the same table. Concretely:

| Dimension | Theirs | Ours |
|---|---|---|
| Corpus | 10 curated boards or seeded random Deals (`vendor/bridge-solver/bench/README.md:159-170`) | one predealt seat + sampler; no shared corpus (`benches/bench_payoff.rs:23-28`) |
| Engine | Rust port | vendored DDS **3.1.0** x86-64 (`docs/adr/0002-dds3-backend.md:1-8`; `AGENTS.md:24`) |
| DDS baseline | **2.9**, arm64 | 3.1, x86-64 |
| Parallelism | explicit 1..12 sweep | fixed `available_parallelism`, context per job |
| Metric | per table / deals per second | whole simulation, criterion estimate |
| Cache sizing | `CutoffCache::new(16)` + `PatternCache::new(16)` (`vendor/bridge-solver/src/par.rs:99-102`) | DDS default context, size unstated |
| Release flags | fat LTO, `codegen-units = 1` (`vendor/bridge-solver/Cargo.toml:80-82`) | no `[profile.release]` in `Cargo.toml:1-98` |
| Measurement | best-of-N, interleaved rounds (`vendor/bridge-solver/bench/README.md:151-157`) | criterion median/estimate |

A fair head-to-head would need, at minimum: the same corpus (the same Deal
strings read by both), the same machine and CPU governor, the same DDS version
and build flags (ours 3.1 vs their 2.9 is not a like-for-like baseline), the
same worker count and context-per-job policy, the same TT cap, the same release
profile, and the same metric and unit — one 20-cell table or one Position solve,
not a whole simulation. It would also have to state which of our benches is the
comparison target, because `bench_payoff`/`bench_leadsim` time generation plus
post-processing while `simulation_paths` times only the pipeline with a mock
(`benches/simulation_paths.rs:84-151`). None of that exists today.

## 5. Capability delta

| Capability | bridge-solver | DDS 3.1 through our seam | Our code |
|---|---|---|---|
| Position solve, trick count | `Solver::solve_with_caches -> u8` (NS) `vendor/bridge-solver/src/bridge_solver.rs:648-654` | `dds_c_solve_board`, three `Solutions` levels `src/solver/dds.rs:80-105,182-188` | `solve`/`solve_tricks` `src/solver/mod.rs:167-183` |
| All legal cards with tricks | only by enumerating and re-solving; `node_alternatives` for one node `vendor/bridge-solver/src/analyse_play.rs:266-371` | `solutions=3` returns every legal card `src/solver/dds.rs:182-188,203-227` | `solve_all_cards`; used by card EV and lead simulation (`src/simulation/card_ev.rs:454`, `src/simulation/solver.rs:138-159`) |
| Equivalence classes | private `is_equivalent`; collapse on lead only `vendor/bridge-solver/src/analyse_play.rs:305-329,788-813` | `equals` bitmask expanded `src/solver/dds.rs:230-242` | `CardSolve::equivalents` `src/solver/mod.rs:89-96` |
| 20-cell DD table | `solve_dd_table` `vendor/bridge-solver/src/par.rs:61-63` | `CalcDDtable`/`CalcAllTables` `dds/src/tables.rs:12,65,88`; `dds_c_calc_dd_table` `dds/src/context.rs:64` | `DdTableCalculator` declared, unimplemented `src/solver/mod.rs:234-246` |
| Par | derived from a table `vendor/bridge-solver/src/par.rs:1-6,543` | `Par`/`DealerPar` `dds/src/context.rs:99-113` | `ParCalculator` declared, unimplemented `src/solver/mod.rs:249-267` |
| Play analysis | per-card costs + alternatives + `optimal_line` `vendor/bridge-solver/src/analyse_play.rs:186,266,389` | `AnalysePlayBin` → `n+1` declarer totals `src/solver/dds.rs:128-178`; `dds/src/context.rs:91` | `analyze_board`/`analyze_hand` `src/performance_analysis.rs:749-870` |
| Batch tables | none; one deal per call `vendor/bridge-solver/src/par.rs:61-97` | `CalcAllTables` batch `dds/src/tables.rs:88` | not used |
| WASM | yes `vendor/bridge-solver/wasm/Cargo.toml:1-20` | no | no |

Overlap with code we already own:

- `performance_analysis::analyze_board` feeds `analyse_play`'s `Vec<u8>` into
  `analyze_hand`, which derives per-play `TrickDelta`s and per-seat statistics
  (`src/performance_analysis.rs:749-766,781-870`). Their `running_trace` computes
  the same DD swing directly as a cost (`vendor/bridge-solver/src/analyse_play.rs:243-255`) but
  returns no trick vector and no player records; it would **replace the backend
  call**, not the analysis layer, and only if the layer were re-typed around
  costs. `node_alternatives` and `optimal_line` have **no counterpart** in our
  tree, so those genuinely extend capability.
- Lead logic: `LeadSimulation` aggregates `solve_all_cards` over sampled Deals
  into `CardLeadResult`/`LeadStatistics`
  (`src/simulation/solver.rs:138-159`, `src/simulation/lead.rs:128-176,241-293`).
  Their `node_alternatives` prices one node of one fully known Deal; the library
  exports no deal generator and no Monte Carlo loop (`vendor/bridge-solver/src/lib.rs:41-61`). So
  theirs would **complement** a single-Deal lead report and cannot replace the
  simulation. Note they use their own move-ordering heuristics for a lead
  (`order_leads`, `vendor/bridge-solver/src/bridge_solver.rs:136-330`), but that is search ordering,
  not lead statistics.
- Table/par: ours are declared but unimplemented (`src/solver/mod.rs:234-267`),
  so their table+par would fill a real gap — while DDS already exports the same
  two capabilities (`dds/src/tables.rs:55`, `dds/src/context.rs:99`), so it is
  not a capability DDS lacks.

## 6. Risk

**Provenance and upstream divergence.** The port claims correspondence to
`macroxue/bridge-solver` at `75b4619` (`vendor/bridge-solver/README.md:125-132`) and keeps a
node-count invariant of exactly 296,689,028 over 200 Deals
(`vendor/bridge-solver/CLAUDE.md:80-90`). Upstream has moved on — 49 commits by 2026-08-31, 27 in
`solver.cc`, of which four changed the search
(`vendor/bridge-solver/bench/results/release-profile.md:48-109`). The port deliberately reproduces at least one
upstream accident: the `Have(1)` implicit-conversion bug in lead ordering, kept
because matching the reference is the goal, with a note that restoring the
intent is a one-line change (`vendor/bridge-solver/src/bridge_solver.rs:228-247`). That is a stated
divergence from "good bridge" in favour of node-for-node parity.

**Maintenance signals.** Version 2.0.0 with a single commit after the tag;
active development through 2026-08-01 (`vendor/bridge-solver/docs/HANDOFF-phase6.md:1-6`) and
2026-07-31 (`vendor/bridge-solver/docs/ROADMAP.md:10-12`); not published to crates.io *because* of
the `bridge-types` git dependency (`vendor/bridge-solver/README.md:96-104`). The version-control
dependency is not rev-pinned in the manifest — only in `vendor/bridge-solver/Cargo.lock`
(`vendor/bridge-solver/Cargo.toml:68-69`, `vendor/bridge-solver/Cargo.lock:65-89`) — so any re-resolve can move it unless
the lockfile travels with the vendored copy. `vendor/bridge-solver/CLAUDE.md` also documents a
`[patch]`-based local workflow and warns that bare cargo silently rewrites the
lockfile (`vendor/bridge-solver/CLAUDE.md:9-21`), which is a hazard for anyone vendoring and building
this crate inside another workspace.

**Feature matrix.** What a default build pulls in:

| Feature | Enables | Effect on a default build | Source |
|---|---|---|---|
| `default = []` | — | library only (search, table, par) | `vendor/bridge-solver/Cargo.toml:12` |
| `cli` | `clap` | binaries `solver-diag`, `bridge-solver` | `vendor/bridge-solver/Cargo.toml:13,39-47` |
| `play-analysis` | `serde`, `sha2` | `analyse_play` module; per-card costs, alternatives | `vendor/bridge-solver/Cargo.toml:15-18`, `vendor/bridge-solver/src/lib.rs:27-28` |
| `bench` | `cli`, `serde`, `serde_json`, `libc` | `solver-bench` | `vendor/bridge-solver/Cargo.toml:20-23,49-52` |
| `dds-reference` | `bench` + native | links arm64 `libdds.a`, `libc++`; panics if absent | `vendor/bridge-solver/Cargo.toml:25-29`, `vendor/bridge-solver/build.rs:16-40` |
| debug features | — | behaviour-altering: `debug_mtdf`, `debug_fast`, `debug_search`, `no_tricks_pruning`, `no_fast_tricks`, `no_slow_tricks` | `vendor/bridge-solver/Cargo.toml:31-37` |

The default graph still declares `bridge-types` and `bridge-encodings` as **git**
dependencies (`vendor/bridge-solver/Cargo.toml:68-69`), even though both sources
are now vendored (§7) and the library modules only use `bridge-types`
(`vendor/bridge-solver/src/convert.rs:13`, `vendor/bridge-solver/src/par.rs:23`); `bridge-encodings` is used only by the
CLI, bench and wasm (`vendor/bridge-solver/src/bin/bridge-solver/main.rs:31`,
`vendor/bridge-solver/src/bin/solver-bench/main.rs:36`, `vendor/bridge-solver/wasm/src/lin_input.rs:13`). `vendor/bridge-solver/THIRD-PARTY-NOTICES`
covers 42 packages for the distributed binaries (`vendor/bridge-solver/THIRD-PARTY-NOTICES:4,21`),
not the library's minimal graph; both `bridge-types` and `bridge-encodings` are
Unlicense (`vendor/bridge-solver/THIRD-PARTY-NOTICES:73-80`).

**Overlap we already own.** There are three overlaps, not one.

- *Play analysis.* Their `cost` is literally our `dd_tricks_lost` — `evaluate_play`
  computes `|prev - curr|` (`src/performance_analysis.rs:690-711`), the same
  one-sided difference they define (`vendor/bridge-solver/src/analyse_play.rs:15-18,243-248`),
  and our `dd_tricks` is the accumulated total from the opening value. So
  adopting `running_trace` would replace the **backend call** inside
  `analyze_board` without re-typing the layer; what stays ours is the per-seat
  aggregation and `TrickDelta` accounting (`src/performance_analysis.rs:781-870`).
  Keeping both implementations would still leave two analyses that can disagree,
  which is the risk `docs/adr/0004-pure-core.md` and the seam exist to avoid.
- *Lead statistics.* `LeadSimulation` aggregates `solve_all_cards` over sampled
  Deals into `CardLeadResult`/`LeadStatistics`
  (`src/simulation/solver.rs:138-159`, `src/simulation/lead.rs:128-176,241-293`).
  Their `node_alternatives` prices one node of one fully known Deal and the
  library exports no deal generator or Monte Carlo loop
  (`vendor/bridge-solver/src/lib.rs:41-61`), so it *complements* a single-Deal
  lead report and cannot replace the simulation. Their `order_leads`
  (`vendor/bridge-solver/src/bridge_solver.rs:136-330`) is search ordering, not
  lead statistics.
- *Importers.* We own LIN and PBN parsers under the default `lin`/`pbn` features
  (`Cargo.toml:25`): `src/linparser/`, `src/pbnparser/` and the serializers in
  `src/record/serialize/{lin,pbn}.rs`. `bridge-encodings` covers five formats
  where we cover two, and is more tolerant on LIN input, but it produces a
  board-shaped `LinData` rather than our `Board` (no contract, no per-card seat)
  and its readers return `Result` while never constructing `ParseError::Lin`.
  Adopting it would duplicate an importer we default-enable, not add a
  capability we lack.

## 7. The vendored type crates

Both are untracked nested checkouts, `vendor/bridge-types` and
`vendor/bridge-encodings`; neither is wired into any build (see "Verdict on the
premise").

### `bridge-types`: the type layer, with a mirrored suit order

- It is a types crate, not a solver: no `Position`, no search, and **no error
  type at all**. Fallible entry points return `Option`
  (`vendor/bridge-types/src/deal.rs:65`, `vendor/bridge-types/src/hand.rs:469`)
  and the checks are opt-in (`vendor/bridge-types/src/deal.rs:109-122`).
- A `Deal` is four public fields (`vendor/bridge-types/src/deal.rs:10-15`) and
  `Hand::from_cards` is total (`vendor/bridge-types/src/hand.rs:18-21`), so ours
  converts in memory. Nothing enforces 13 cards per seat: `is_complete` is a
  52-card total and `is_valid` is no-duplicates
  (`vendor/bridge-types/src/deal.rs:109-122`).
- **Suit order mirrors ours and `Ord` with it** (`Clubs = 0 … Spades = 3`,
  `vendor/bridge-types/src/card.rs:7-12`, against
  `src/shape.rs:899-911`). Seats agree (`North = 0 … West = 3`,
  `vendor/bridge-types/src/direction.rs:8-13` vs `src/record/deal.rs:390-399`),
  so the transpose risk is confined to suits.
- `DdTable` is the one boundary type that is safer than a raw array: private
  layout, accessors taking a `Direction` and a `Strain`
  (`vendor/bridge-types/src/dd_table.rs:52-55,127,135`), and a doc block that
  names the three row orders which were in circulation before it
  (`vendor/bridge-types/src/dd_table.rs:27-45`).
- Their `Board` is a PBN interchange record, not our input: `dealer` is
  `Option`, `contract` is `Option<String>`, declarer a separate
  `Option<Direction>`, `play` an `Option<PlaySequence>` of
  `Trick { leader, cards: [Option<Card>; 4], .. }`, and there is **no claim
  field** (`vendor/bridge-types/src/board.rs:107-170`,
  `vendor/bridge-types/src/play.rs:5-17`). Our validated `Board` cannot be
  reconstructed from it without inventing the parts it leaves out.
- Licence Unlicense (`vendor/bridge-types/Cargo.toml:5`); HEAD `cc75919`, 9
  commits past the revision the solver's lock pins.

### `bridge-encodings`: five formats against our two

- Modules `lin`, `oneline`, `pbn`, `printall`, `zrd`, each read and write
  (`vendor/bridge-encodings/src/lib.rs:26-31`), plus a `DealReader` that
  auto-detects oneline, PBN `[Deal]` and printall — but not LIN
  (`vendor/bridge-encodings/src/reader.rs:124-160`).
- PBN produces a typed auction with per-call annotations and resolved `[Note]`s,
  play with per-trick winners, both DD-table encodings, `extra_tags`, and a
  `PbnDocument` for byte-exact editing (`vendor/bridge-encodings/src/pbn/reader.rs:157`,
  `vendor/bridge-encodings/src/pbn/document.rs:92`). Those last three are gaps in
  our importer, not duplicates of it.
- LIN produces a `LinData` with auction, `play: Vec<Card>` and a claim, but no
  contract and no per-card seat (`vendor/bridge-encodings/src/lin/mod.rs:38-63`).
- The readers are infallible behind a `Result` API: `parse_lin` is
  `Ok(from_tokens(..))` (`vendor/bridge-encodings/src/lin/mod.rs:122-124`) and
  `ParseError::Lin` is never constructed anywhere in the crate, so a returned
  `Err` is not a correctness signal one can rely on.
- Its own dependency is `bridge-types` **by git** plus `thiserror`
  (`vendor/bridge-encodings/Cargo.toml:9-11`), so redirecting it alone still
  leaves the types edge; no `build.rs`, no `unsafe`.
- Verdict: duplicate-plus-gap. It would duplicate an importer we already
  default-enable, and its output is not our `Board`.

### The pin mismatch is a provenance problem, not a build problem

The solver's lock pins `bridge-types` at `57f3224` and `bridge-encodings` at
`446bff5`; both are ancestors of the vendored HEADs, 9 and 20 commits behind
(`git rev-list --count <pin>..HEAD`). The drift is additive and includes exactly
the features the newer crates advertise: `DdTable::NULL` and `is_solved` are
absent from `dd_table.rs` at the pin, and `PbnDocument` editing is one of the 20
commits. Both crates still declare `0.1.0`
(`vendor/bridge-types/Cargo.toml:3`, `vendor/bridge-encodings/Cargo.toml:3`), so
a path redirect, which matches on version, cannot tell the two revisions apart —
it builds whatever the checkout's HEAD is, not the revision their benchmarks were
measured on. That is why "sources present" and "reproduces the measured build"
are separate claims in this note.

## What this note does not settle

- Whether the Rust port is faster or slower than our vendored DDS **3.1 on
  x86-64**. Every published comparison is against DDS 2.9 on arm64
  (`vendor/bridge-solver/bench/comparison/METHODOLOGY.md:9-13`), and our own wins/losses are unmeasured against it.
- Whether the seam should adopt `bridge_types::DdTable` (or other
  `bridge_types` types) as its public `Table`/`Par` types. Doing so removes the
  reindexing that a shim would otherwise need, at the cost of a foreign type in
  our public API, and the mirrored suit order (see §7) is a live hazard on any
  other conversion path. That is a design decision, not a fact in the tree.
- The TT capacity of our default DDS context (`src/solver/dds.rs:44`), which is
  needed to size a like-for-like comparison.
- The memory cost of a bridge-solver context under *our* context-per-job policy
  (`src/solver/pool.rs:24-38`); the crate measures cache construction at about a
  megabyte for `CutoffCache::new(16)` (`vendor/bridge-solver/src/par.rs:104-110`) but not under that
  usage pattern.
- What, if anything, the 9- and 20-commit drift between the pinned revisions and
  the vendored HEADs changes numerically. The published benchmarks were taken on
  the pinned revisions (`vendor/bridge-solver/Cargo.lock:65-89`), and a path
  redirect would build the newer source.

## Recommended next step

**Drop it** (as a replacement backend).

The premise now holds in the narrow sense only: there is no native step
(`vendor/bridge-solver/build.rs:16-18`), and the sources are in the tree, but an
unpatched build still is not offline-buildable, and wiring it would build the
vendored HEAD rather than the revision the benchmarks used.

The decisive evidence is shape, not speed. Three seam methods matter in
production, not two: `solve_tricks`
(`src/simulation/solver.rs:126`), `solve_all_cards`
(`src/simulation/solver.rs:152`) and `analyse_play`
(`src/performance_analysis.rs:760`). The first fits behind a ~30-line shim. The
second fights the interface: there is no card-returning solve, `node_alternatives`
needs a fabricated play prefix, cannot start mid-trick, recomputes `total` from
West's hand size, returns no equivalence classes, and costs one cold solve and
one ~1 MB cache pair per legal card against DDS's single `solve_board`
(`src/solver/dds.rs:182-188`). The third fits behind a shim
of ~40 lines plus two conversions, and its `cost` is already our
`dd_tricks_lost`.

Against that, the published gains are a 1.06x per-core geomean over DDS 2.9,
1.08x on 500 random deals and 1.12x at one thread on 240 deals
(`vendor/bridge-solver/bench/results/release-profile.md:709-715,734-741,1030-1045`),
on arm64, against a DDS version we do not use. A head-to-head measurement would
buy a number for an engine that does not fit one of the two hot paths, and the
genuinely additive capabilities — `node_alternatives` and `optimal_line`
(`vendor/bridge-solver/src/analyse_play.rs:266,389`) — are an analysis-layer
question that does not require replacing the solver.
