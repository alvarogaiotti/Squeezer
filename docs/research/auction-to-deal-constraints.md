# Auction → Deal constraints in rusty-bidding-bot: what the tree actually provides

This is a research note, not a design. Every claim is cited by file and line;
where the tree is silent the note says so. Nothing was built or benchmarked for
it: every performance figure quoted is a number already written down elsewhere in
either tree, not a measurement made for this note.

`RBB/` is shorthand for `vendor/rusty-bidding-bot/` throughout; every path
resolves by substitution. That checkout is an untracked nested git repository at
tag `v0.10.0` (`091175f`), not a submodule (`.gitmodules:1-3` lists only
`vendor/dds`; `git status` shows `?? vendor/rusty-bidding-bot/`).

Vocabulary follows `CONTEXT.md`: **Auction** (ordered calls), **Call**, **Bid**,
**Deal** (un-played four hands), **Hand**, **Seat**, **Known cards**.

## Verdict on the premise

> Given a dealer, a vulnerability and an Auction, can we derive the set of Deals
> consistent with it, cheaply enough to condition deal generation and simulation
> sampling on it?

**Partly — the reverse mapping exists, but not as a Deal-set, and what it encodes
is much narrower than what a bridge player reads into the auction.**

- **A mapping from Auction to constraints does exist, and it needs no hands.**
  `Engine::interpret(dealer, vul, scoring, &[Call])` (`RBB/crates/engine/src/engine.rs:505-518`)
  replays an auction and returns, per call, `Step::knowledge: SeatKnowledge`
  (`RBB/crates/engine/src/engine.rs:40-51`), plus a final `Position` carrying one
  `SeatKnowledge` per seat (`RBB/crates/engine/src/position.rs:76-108,120-124`).
  This is the reverse direction and it is the one thing the tree has that we lack.
- **There is no Deal-level predicate in their tree, and none is needed.** Their
  side of the job is to emit constraints; ours is to generate from them, and a
  Deal that is generated under those constraints satisfies them by construction.
  The interpreter deliberately does not check the caller's hand
  (`RBB/docs/LANGUAGE.md:494-499`), and `compare` never checks a Deal against an
  auction either — it replays a known `Board`'s reference auction and compares the
  engine's *choices* (`RBB/crates/compare/src/board.rs:212-247`). The consequence
  is not a gap but a division of labour: turning their `SeatKnowledge` into our
  generator's constraint is ours to write and test.
- **What is derived is per-seat marginals over six summary fields**: HCP, the
  four suit lengths, balance, and the point counts `pts`, `tp`, `dp`
  (`RBB/crates/engine/src/knowledge.rs:119-143`).
  Everything else a call "shows" — honours, suit quality, controls, keycards,
  exact shape — is stored as an opaque `Expr` and is *unreadable by the other
  seats* (`RBB/crates/engine/src/eval.rs:1128-1150`).
- **Cost splits in two.** One interpretation per auction is a one-off of roughly
  a millisecond; replaying per Deal is 10²–10⁴× our `Dealer::deal()` budget. Only
  the once-per-auction shape fits our loop (§3).
- **Fit with our model is closer than the constraint type suggests.** Any
  shape-only condition an auction can emit is expressible as a `HandConstraint`
  today: shape is a finite domain of 1,379 exact distributions held as a bitset,
  so length ranges are pattern unions (`25+xx`, `35+xx`) or a compile step into
  that table — not a missing capability. What genuinely does not fit is points
  beyond HCP, honours/controls/keycards, and any cross-seat condition (§4).
- **Dependency-wise it is pure Rust** for `bidspec` and `rbb-engine` (no
  `build.rs`, no native step), but it needs two first-party **git** dependencies
  and two convention cards, and its expression evaluator is private, so "their
  rules without their bid chooser" is not a library surface we can call (§5).

### The crate split: which crate does what

The brief suspected `bidspec` and `compare` are the relevant crates and `engine`
is not. **That is wrong on both counts, in opposite directions.**

| Crate | Verdict | Evidence |
|---|---|---|
| `bidspec` | **Relevant as the spec language only.** It parses `.bid` rules to an AST/JSON IR and validates them against a card vocabulary. It has no hand, no auction state, no knowledge, and never sees a `Deal` or `Auction`. | `RBB/crates/bidspec/src/lib.rs:27-47`; exports `parse`, `compile`, `ast::Module` (`:23-25`); "must not depend on the engine" (`RBB/crates/bidspec/Cargo.toml:9`) |
| `engine` | **The only relevant crate for the question.** It owns `Position`, `SeatKnowledge`, the condition evaluator, the interval narrowing, `interpret`/`advance`/`choose`, and the only constraint→hands filter in the tree. | `RBB/crates/engine/src/lib.rs:9-19,39-49`; `RBB/crates/engine/src/engine.rs:473-518` |
| `compare` | **Not the reverse direction.** It is a forward corpus harness: replay a reference auction over a whole `Board`, score agreement, par, divergences. Its one conditioning routine samples a partner from a measured pool of hands, not from constraints. | `RBB/crates/compare/src/lib.rs:1-14,173-221`; `RBB/crates/compare/src/board.rs:212-247`; `RBB/crates/compare/src/simulate.rs:1-8,57-104` |
| `assets` | Embeds the rules and stock cards at build time; optional. | `RBB/crates/assets/build.rs:1-14`; `RBB/crates/assets/src/lib.rs:14` |
| `cli`, `wasm`, `workbench` | Front ends. `wasm` is the one route to the knowledge data from outside Rust, and it truncates it (§3). | `RBB/crates/wasm/src/api.rs:961-983` |

So: **`bidspec` gives us their rule language and data; `engine` gives us the
mapping; `compare` gives us neither** — it is a consumer of both that happens to
contain the closest existing experiment in auction-conditioned sampling
(`simulate`, §1).

## 1. Direction

**The tree maps an Auction to per-seat constraints, and never maps an Auction to
a set of Deals.** It also does not validate an auction against a known Deal; it
only *reads* the auction.

### Entry points

| Function | Signature | Direction |
|---|---|---|
| `bidspec::parse` | `pub fn parse(source: &str, file: &str) -> Result<Module, Vec<Diagnostic>>` (`RBB/crates/bidspec/src/lib.rs:29`) | rule text → IR; **no Auction, no Deal** |
| `bidspec::compile` | `pub fn compile(source: &str, file: &str, registry: &bridge_card::Registry) -> Result<Module, Vec<Diagnostic>>` (`RBB/crates/bidspec/src/lib.rs:35-39`) | the same, checked against a card vocabulary |
| `Engine::start` | `pub fn start(&self, dealer: Direction, vul: Vulnerability, scoring: ScoringMethod) -> Position` (`RBB/crates/engine/src/engine.rs:473`) | seed, no auction |
| `Engine::advance` | `pub fn advance(&self, pos: &mut Position, call: &Call) -> Step` (`:483`) | one call → knowledge; **no hand** |
| `Engine::interpret` | `pub fn interpret(&self, dealer: Direction, vul: Vulnerability, scoring: ScoringMethod, calls: &[Call]) -> Interpretation` (`:505-511`) | whole auction → knowledge; **no hand** |
| `Engine::choose` | `pub fn choose(&self, pos: &Position, hand: &Hand) -> Choice` (`:489`) | forward: hand → call |
| `Engine::step` | `pub fn step(&self, pos: &mut Position, hand: &Hand, actual: &Call) -> (Choice, Step)` (`:497`) | both at once, for replay |
| `Engine::review` | `pub fn review(&self, …, calls: &[Call], hands: &[Option<Hand>; 4]) -> Vec<ReviewRow>` (`RBB/crates/engine/src/review.rs:391-397`) | auction → per-seat knowledge per call, hands optional |
| `rbb_compare::run` / `run_with` | `run(opts: &Options, progress: &(dyn Fn(usize, usize) + Sync)) -> Result<Report, String>` (`RBB/crates/compare/src/lib.rs:173`), `run_with(...)` (`:179`) | forward corpus comparison |
| `board::compare` | `pub fn compare(engine: &Engine, scenario: &str, board: &Board) -> Option<BoardResult>` (`RBB/crates/compare/src/board.rs:212`) | forward, over a known `Board` |
| `compare::simulate::run` | `pub fn run(spec: &Spec, pool: &[Hand], samples: usize, seed: u64) -> Result<Vec<SimRow>, String>` (`RBB/crates/compare/src/simulate.rs:57`) | generates Deals around known hands |

`Interpretation` is exactly the reverse artefact:

```rust
// RBB/crates/engine/src/engine.rs:54-59
pub struct Interpretation {
    pub steps: Vec<Step>,
    pub position: Position,
}
```

and each `Step` carries `pub knowledge: SeatKnowledge` plus the rule that gave
the call its meaning, its explanation, alert and artificial flag
(`RBB/crates/engine/src/engine.rs:40-51`). The doc comment on `advance` is
"Interpret one more call: what it shows, and its effect on the state"
(`:482`); on `interpret`, "Replay `calls`, working out what each call showed"
(`:504`).

### The mapping is hand-free, but card-dependent

`advance` passes `hand: None` into the evaluator (`RBB/crates/engine/src/engine.rs:223`,
`ctx(pos, actor, None, e)`), and the docs say so: "It does not check the caller's
hand, which it cannot see" (`RBB/docs/LANGUAGE.md:499`). So the meaning is

```
(rules, NS card, EW card, dealer, vulnerability, scoring, calls) → [SeatKnowledge; 4]
```

The cards are not optional. `Engine::new(ns: &Card, ew: &Card, rules: &RuleSet)`
panics if a card is not in the rules' vocabulary (`RBB/crates/engine/src/engine.rs:162-171`),
and the card decides which modules are active and what every `param` resolves to
(`RBB/crates/engine/src/system.rs:86-131`), plus how points are counted
(`RBB/crates/engine/src/facts.rs:45-52`). The contract states it: a call is read
"with the caller's card" (`RBB/docs/CONTRACT.md:145-146`). **Consequence for us:
the same auction means different things under 2/1, SAYC, Precision or Acol, so
conditioning requires knowing the pair's system, not just the calls.**

### What is *not* there

- **No Deal filter.** `board::compare` replays the reference auction and, at each
  call, asks what the engine would have bid with the real hand; the "knowledge"
  it uses is a counter of decisions where partner's HCP floor exceeded 0
  (`RBB/crates/compare/src/board.rs:179-191`). That is a diagnostic, not a filter.
- **The only constraint→hands code is private and fixed-pool.**
  `fn consistent_hands(&self, pos, actor, key) -> Arc<Consistent>`
  (`RBB/crates/engine/src/engine.rs:314`) filters `self.pool` — 20,000 precomputed
  `Facts` from 5,000 seeded random deals (`RBB/crates/engine/src/sample.rs:9,26-42`)
  — by one seat's ranges and residual expressions (`engine.rs:329-357`). It exists
  only to compute a scalar `descriptiveness` score for ranking, is not exported,
  and returns hand *indices*, not Deals.
- **The closest thing to conditioning is empirical, not derived.**
  `compare::simulate` deals partner from "a pool of hands that made partner's
  call (e.g. the hands BBA opens 1NT)" (`RBB/crates/compare/src/simulate.rs:1-8`),
  loaded from `probes/pools/`; the one shipped pool is "25,012 hands BBA
  (Basic-Bridge card, "1NT opening shape 5422" on) opened 1NT"
  (`RBB/probes/pools/bba-1nt-15-17-with-5422.txt:1`). Their own answer to "which
  hands make this call" is measurement, not the constraint machinery.

So the plain answer to question 1: **reverse mapping yes (to per-seat
constraints), forward validation no, Deal-set no.** It is not "one-way only", but
it is not a filter either — it is an interpreter that produces necessary
conditions on each seat, which we would have to turn into a filter ourselves.

## 2. Expressiveness of an auction as a constraint

### The condition vocabulary

The rule language is genuinely rich (HCP, lengths, shape patterns, suit quality,
honour cards, stoppers, controls, losers, keycards, partnership totals, state).
The check at load time accepts exactly the tables in `eval.rs`
(`RBB/docs/CONTRACT.md:47-54`; `RBB/crates/engine/src/eval.rs:1882-1901` for
own-hand attributes). Representative terms:

| Family | Term | Implementation |
|---|---|---|
| HCP, points | `hcp`, `points`, `suit_points`, `tp(x)`, `length_points` | `RBB/crates/engine/src/eval.rs:1884-1894` |
| Suit lengths | `S H D C`, `M`, `x` | `eval.rs:687,710-715,838-840` |
| Shape | `balanced`, `semibalanced`, `shape 4333`, `shape 5-4-x-x`, `shortest`, `longest` | `eval.rs:486-492,1889-1893`; `RBB/crates/engine/src/facts.rs:486-503` |
| Suit quality, honours | `quality(x)`, `top5(x)`, `has(Q,x)` | `eval.rs:1007-1012,1027-1036`; `facts.rs:403-408,445-450` |
| Stoppers, shortness | `stop(x)`, `bare(x)`, `bare_suits` | `eval.rs:1013-1026`; `facts.rs:430-459` |
| Controls, losers, keycards | `controls`, `losers`, `quick_tricks`, `keycards(x)`, `tens` | `eval.rs:1006,1885-1898`; `facts.rs:375-399,412-426` |
| Partnership | `we.fit(x)`, `we.hcp`, `we.tp(x)`, `we.keycards(x)`, `safe_level(x)` | `eval.rs:1225-1261`; `RBB/crates/engine/src/macros.rs:349-405` |
| Opponents | `they.hcp`, `they.fit(x)`, `they.level`, `they.strain` | `eval.rs:1288-1326` |
| History | `partner.last`, `bypassed(x)`, `jumped`, `denied(x)`, `cued(x)`, `asked`, `answered` | `eval.rs:468-485,1090-1131` |
| State | `we.forcing`, `we.trump`, `systems on`, `they.still_bidding` | `eval.rs:1205-1224` |
| Judgment hooks | `slam_try` (31 combined), `grand_try` (35) | `eval.rs:930-937` |

### What becomes a constraint on another seat

Only six summary fields narrow, and only those are summarised in a form a partner
can read (`RBB/crates/engine/src/knowledge.rs:119-143`):

```rust
pub struct SeatKnowledge {
    pub hcp: Range,
    pub len: [Range; 4],
    pub balanced: Tri,
    pub pts: [Range; 2],
    pub tp: [Range; 4],
    pub dp: Range,
    pub shown: Vec<String>,     // human-readable, not queryable
    #[serde(skip)]
    pub constraints: Vec<Expr>, // the residual, not narrowed
}
```

The crate says it plainly:

> "Narrowing is interval reasoning. It is sound (never excludes a hand that
> satisfies the constraints) but not complete: a disjunction narrows to the hull
> of its branches, and **terms other than HCP, suit lengths, balance and the
> point counts are kept in `constraints` without narrowing**."
> — `RBB/crates/engine/src/knowledge.rs:4-10`

`SeatKnowledge::add` records *every* constraint and narrows by the ones the
interval machinery understands (`knowledge.rs:290-330`); `held_by_ranges` is the
allow-list of "already exact" forms (`knowledge.rs:997-1015`). The narrowing
grammar covers conjunctions/disjunctions/negations of comparisons where one side
reduces to a single attribute among `hcp`, `points`, `suit_points`, `tp(x)`,
the four suit letters, `length_points` and `hcp+length_points`
(`knowledge.rs:732-807,873-991`).

**The reading side is the sharp limit.** For anything else, a partner's value is
*not* the stored constraint — it is a default:

```rust
// RBB/crates/engine/src/eval.rs:1136-1146
"has" | "stop" | "semibalanced" | "bare" => Val::Bool(Tri::Unknown),
...
"keycards" | "controls" | "losers" | "quality" | "top5" | "quick_tricks" => {
    Val::Num(Range::new(0, 40))
}
```

So `has(A,S)` about partner is Unknown, and `quality(S)`, `controls`,
`keycards` about partner are `0..40`. The single exception is
`we.keycards(t)` on the asking side, which scans the partner's stored
constraints for a keycard answer (`eval.rs:646-681`) — a hand-rolled query, not a
range. Nothing else reads `constraints` (`knowledge.rs:372`; `engine.rs:138`).

### The auction features asked about

| Feature | Modelled? | Encoded as a constraint? | Evidence |
|---|---|---|---|
| Per-seat HCP | Yes | Yes, narrows | `knowledge.rs:120`; `eval.rs:1884` |
| Per-seat shape (balance) | Yes | Yes, narrows | `knowledge.rs:123`; `knowledge.rs:980-986` |
| Exact shape patterns | Yes (`shape 4333`, `5-4-x-x`) | **No** — opaque, does not narrow, and no term reads another seat's shape | `eval.rs:486-492`; `knowledge.rs:987` |
| Suit lengths | Yes | Yes, including sums and relations | `knowledge.rs:1034-1047` |
| Suit quality | Yes (`quality(x)`) | **No** | `eval.rs:1007-1009`; `eval.rs:1142` |
| Honour holdings | Yes (`has(Q,x)`, `top5(x)`, `stop(x)`) | **No** | `eval.rs:1013-1036`; `eval.rs:1136` |
| Control showing | As a ladder dialogue only | **No** — `cued`/`denied` bitmasks, first and second round recorded alike | `RBB/conventions/slam/control-bids.bid:60-88`; `RBB/crates/engine/src/engine.rs:1311-1329`; `RBB/docs/CONTRACT.md:592` |
| Keycard responses | Yes (`shows keycards(t) in 1\|4`) | Recorded, only the asker's `we.keycards` can read it; `partner.keycards` is `0..40` | `RBB/conventions/slam/rkcb-1430.bid:225-245`; `eval.rs:646-681,1142` |
| Relays | Partly: Gazzilli/XYZ show HCP/balance/length buckets; not full shape resolution | Buckets narrow; `shape` does not | `RBB/conventions/majors/gazzilli.bid:30-33,112-142`; `RBB/conventions/base/xyz.bid:18-33`; `RBB/conventions/slam/spiral.bid:39-43` |
| Competitive auctions | Yes: Michaels, Unusual 2NT, Cappelletti lengths and `hcp+length_points` | Yes, lengths and point sums narrow | `RBB/conventions/competitive/two-suited-overcalls.bid:95-169`; `RBB/conventions/competitive/cappelletti.bid:39-46` |
| Forcing passes | Yes, as *state* | **No** — `SideState` only; the opponents cannot read it | `RBB/crates/engine/src/position.rs:32-53,309-315`; `RBB/docs/CONTRACT.md:139-143` |

Three worked examples of the gap:

1. **A keycard auction conveys nothing numeric.** The 4NT ask has no `shows` at
   all; its strength test ("33 combined support points", "no two bare suits") is
   entirely in `when`, and a `when` about the caller's own hand "is not judged
   and stay[s] possible" (`RBB/docs/LANGUAGE.md:506-511`;
   `RBB/conventions/slam/rkcb-1430.bid:67-79,86-126`). What partner learns is the
   *state* `ask=keycards(trump)`, which a third party cannot query.
2. **A control-bid auction conveys masks, not cards.** `control-bids.bid` has
   **0 `shows` clauses in 23 rules**; meaning is `sets ask=control(C),
   ladder=control` (`RBB/conventions/slam/control-bids.bid:60-88`), and the
   contract admits first- and second-round controls are "recorded alike"
   (`RBB/docs/CONTRACT.md:592`) — an ace and a singleton are indistinguishable.
3. **A transfer completion conveys nothing.** The transfer bid shows `H>=5`
   (`RBB/conventions/notrump/jacoby-transfers.bid:18-19`); the completions have no
   `shows` (`:27-31`), so "no super-accept" is not encoded anywhere.

### What it silently does not constrain — the danger case

Counted statically over the 106 `.bid` files and 4,022 rule heads (rule heads are
indented call tokens; each rule is attributed any `shows` before the next rule
head):

| Measure | Count | Share |
|---|---:|---:|
| Rules | 4,022 | 100% |
| Rules with at least one `shows` | ~3,390 | ~84% |
| Rules with no `shows` | ~630 | ~16% |
| `shows` clauses (raw `^\s+shows ` lines) | 1,371 | — |
| `slam/` rules with `shows` | 192 of 358 | **54%** (vs ~84% overall) |

The mechanisms that lose meaning, each cited:

- **No `shows` ⇒ says nothing.** `descriptiveness_of` returns `0.0` when `shows`
  is absent (`RBB/crates/engine/src/engine.rs:387-390`); such rules rank last and
  transmit no constraint. Their own comment: "Nothing here says anything about
  the hand, so it ranks last" (`RBB/conventions/slam/slam-catch.bid:20-23`).
- **`prefer` ties skip negative inference.** Inference is driven by
  `(priority, descriptiveness)` only (`engine.rs:31-36,628`), and "Where `prefer`
  breaks a tie using the hand, the inference is skipped for that group"
  (`RBB/docs/LANGUAGE.md:518-519`; `RBB/docs/JUDGMENT-LAYER.md:443-446`).
  98 rules carry `prefer`.
- **Unresolvable terms are dropped from denials, treated as true.** "Terms that
  cannot be resolved to a number are dropped (treated as true), which loses
  information but never excludes a possible hand"
  (`RBB/crates/engine/src/eval.rs:1453-1475`).
- **A contradiction is swallowed.** `add` returns `false` and leaves the ranges
  unchanged, with only a warning (`RBB/crates/engine/src/knowledge.rs:290-303`;
  `engine.rs:614-616`). A self-contradictory or misread auction therefore yields
  *fewer* constraints, silently — not an error.
- **Judgment hooks are placeholders.** `slam_try`/`grand_try` are "combined HCP
  of at least 31 or 35" (`RBB/docs/LANGUAGE.md:670-671`;
  `RBB/crates/engine/src/eval.rs:930-937`); `slam/slam-entry.bid:5-6` says "No
  rules yet: what decides whether slam is worth looking for".
- **Disjunctions collapse to a hull.** "4 hearts or 4 spades" becomes 4+ in both
  (`RBB/crates/engine/src/knowledge.rs:4-10`), then possibly shaved later; the
  disjunction itself survives only as an opaque `Expr`.
- **No rule at all ⇒ pass.** `engine.rs:847-858`; `RBB/docs/LANGUAGE.md:670-672`.
  (The same doc claims a "built-in natural interpretation applies"
  (`RBB/docs/LANGUAGE.md:547-548`) — no such code exists, see SESSION.md.)

**And one failure in the other direction, which matters more for a filter.** The
reading is sound *relative to their rules and card*: "a hand the calls allow is
never excluded" (`RBB/docs/LANGUAGE.md:541-546`). It is not sound relative to a
foreign system. A call is read as the highest-priority rule that offers it, so

> "A fallback rule that bids a call a higher rule also offers in the same
> position is read as the higher rule, which is what its hand did *not* have."
> — `RBB/docs/CONTRACT.md:365-373`

If we condition on an auction bid under BBO/2/1 by players who were not following
these rules, the derived constraints can be *too strong* and reject Deals that
are perfectly consistent with what those players meant. The filter's soundness is
only as good as the card's fidelity to the pair who bid.

## 3. Cost per Deal

**There is no benchmark in the tree.** No `benches/` directory, no criterion/iai/
divan dependency, no `#[bench]`, in any crate. The only numbers are prose:

| Measurement | Value | Evidence |
|---|---|---|
| 20,000 random boards through `rbb compare` | 20 s, machine shared with other runs | `RBB/docs/random-deals-comparison.md:85-87` |
| bba-cli reference bidding | 90–150 deals/s | same lines |
| Generating the 20,000-deal random sample | 0.2 s | same lines |
| Auction length | ≈9.1 calls per auction (1,554,148 calls / 170,633 boards) | `RBB/docs/JUDGMENT-LAYER.md:25` |
| Our `Dealer::deal()`, for comparison | 143.53 ns (unconstrained) to 1.7947 µs (shape+accept) | `docs/performance-optimization.md:127-137`, via `benches/deal_constraints.rs:23` |

The 20 s figure is wall-clock over a rayon pool sized to `cores - 2`
(`RBB/crates/engine/src/lib.rs:26-37`), so CPU cost per board is that number times
the pool width. Call it **~1 ms of wall per board, and the pool width times that
in CPU**, for a full replay+choice. A board is ~9 calls, so the *single*
interpretation we would reuse is around a millisecond cold, much less warm.

### What the work consists of

| Operation | Per auction or per deal? | Work | Evidence |
|---|---|---|---|
| `pos_key` | **per call** | formats dealer/vul/scoring plus `to_pbn()` of **every call so far** → O(n²) bytes per auction | `RBB/crates/engine/src/engine.rs:126-133` |
| `cands_at` | per call | `Mutex` lock even on hit; `Arc` clone; `Vec<String>` warnings clone | `engine.rs:194-207,485` |
| `candidates` | per distinct position | linear scan of the active rule list (≤1,658), `Bindings` per rule, `ctx.cond` per `when`, `expand` per call | `engine.rs:237-310`; `RBB/docs/WASM.md:635` |
| `consistent_key` | per position | `format!("{:?}\|{:?}", actor, constraints)` — Debug-format of every `Expr` | `engine.rs:137-139,295` |
| `descriptiveness_of` | per candidate per position | key built from sorted bindings; one pass over up to 20,000 hands **per candidate** | `engine.rs:379-470`, loop `:457-466` |
| `consistent_hands` | per distinct constraint set | `from_constraints` re-clones and re-narrows every `Expr` (O(k²)), then scans all 20,000 `Facts` | `engine.rs:314-374`; `knowledge.rs:370-376` |
| `SeatKnowledge` clones | **per call** | three deep clones per call (before, after, into `Step`), including `Vec<String>` + `Vec<Expr>` | `engine.rs:574,582,709` |
| `apply_deck_hcp` | per call | up to 4 rounds × 4 seats × 5 bounds, each `add` → `to_string` | `RBB/crates/engine/src/position.rs:317-358` |
| `Facts::new` | per hand | 13-card loop, no heap | `RBB/crates/engine/src/facts.rs:67-91` |
| testing one `Hand` against one `SeatKnowledge` | per deal | 1 HCP + 2 point + 4 length + 1 dp range compares, then one `Ctx::cond` per residual `Expr` | `engine.rs:329-357` is the only in-tree example |

Caches are per `Engine` and keyed by auction content, behind `std::sync::Mutex`
(`engine.rs:141-152`); `cands_at` clears at 400,000 entries
(`engine.rs:116-118,202-204`), the other three caches have no limit. They make
the *replay* cheap across boards that share prefixes; they do nothing for a
per-deal check.

### What is cheap, and what is 10×–100× too slow

- **The Deal check itself is cheap.** Testing a `Facts` against one seat's
  `SeatKnowledge` is a handful of integer range comparisons plus one `cond` per
  residual expression: tens of nanoseconds in the shape shown at
  `engine.rs:329-357`. Four seats is ~4× that.
- **Replaying the auction per Deal is not.** Every `advance` allocates a fresh
  `String` key that re-serialises the whole prefix, takes a mutex, clones three
  `SeatKnowledge` values (deep-copying every `Expr` and every `shown` string), and
  rewrites the `shows` and denial expression trees. At ~9 calls that is O(n²)
  copying per auction. Against a 143 ns–1.8 µs `Dealer::deal()`, even a warm
  replay is **10²–10⁴× too slow** for the inner loop.
- **Cold positions are 10³× worse again.** A new auction prefix scans every rule,
  then for every candidate with a `shows` it filters a 20,000-hand pool, and for
  each distinct constraint set scans the pool again. That is 10⁵–10⁶ condition
  evaluations per novel auction (`engine.rs:246,457-466,329-357`). Fine once per
  auction; fatal per Deal.
- **Not incremental.** The only public routes from "a Deal and an auction" to an
  answer are `bid`, `bid_deal` and `review`, and all three call `start()` then
  replay from the dealer (`engine.rs:512-513,886-912`; `review.rs:395-462`).
  There is no resumable prefix, no "interpret from a cached position", and no API
  taking precomputed constraints plus a hand.
- **The evaluator is private.** `mod eval;` is private and only `check_terms`,
  `terms_reference`, `Term` are re-exported (`RBB/crates/engine/src/lib.rs:12,66`);
  `Ctx` and `Bindings` are not (`eval.rs:353-367`). `SeatKnowledge.constraints`
  is a public field (`knowledge.rs:142`), but **we cannot evaluate those `Expr`s
  against a hand without reimplementing the evaluator**, and the residual is
  exactly where honours, quality, controls and keycards live.
- **The serialised form is even narrower.** `constraints` is `#[serde(skip)]`
  (`knowledge.rs:141-142`), and the WASM JSON emits only `hcp`, four `lengths`,
  `balanced`, `summary`, `shown` (`RBB/crates/wasm/src/api.rs:961-983`). Points,
  `tp`, `dp` and the residual expressions never cross that boundary.

**Verdict:** the cost profile permits exactly one shape — **interpret the auction
once (≈1 ms, amortised over the thousands of Deals in a solve), keep the four
`SeatKnowledge`s, and test each Deal with our own code.** It does not permit
calling into their engine per Deal, and it does not permit using their residual
constraints at all without a second evaluator.

## 4. Fit with our model

Two corrections to the premise first, because they change where to look:

- **`src/dealproduction.rs` is not the production module.** It is 123 lines and
  contains only `DealerError` and its trait impls (`src/dealproduction.rs:4-55,65-123`).
  Generation is `src/record/deal.rs`: `DealerBuilder` (`:896`), the
  `Dealer` trait (`:1442-1478`), `StandardDealer` and its `deal` (`:1600-1641`).
- **`HandConstraint` is `(shape set, HCP)`** — a union of branches, each a
  `Shapes` bitset plus an `HcpRange` (`src/hand_constraint.rs:123-128,135-146`).
  No point counts, honours, controls, keycards or seat parameter. It has no
  *front end* for length ranges, but that is not a limit on what it can express.

### Shape is a finite domain, so shape constraints are already complete

`Shapes` is a 1,379-bit table — one bit per exact distribution
(`src/shape.rs:58,66`) — plus conservative per-suit bounds used as a pre-check
(`src/shape.rs:459-487`). Membership is eight integer comparisons and one bit
test. Because the universe is finite and small, **every shape-only constraint is
expressible as a set of those bits**, including arbitrary per-suit length ranges:

```rust
// 15-17 HCP, 5+ hearts, 2-3 spades. Patterns are S,H,D,C.
HandConstraint::new(&["25+xx", "35+xx"], HcpRange::new(15, 17))
```

`+` and `x` are ordinary modifiers and jokers (`src/shapeparser/scanner.rs:42-46`),
and `x7+xx` is already used in-tree for "hearts longest, 7+"
(`src/hand_constraint.rs:51`). The equivalent box of ranges is also constructible
one layer down: `Shapes::from(&[LenRange; 4])` (`src/shape.rs:767-800`) and
`LenRange` (`src/shape.rs:821`) are public. What is not public is the bridge into
`HandConstraint`: `from_compiled(shapes, hcp)` is `pub(crate)`
(`src/hand_constraint.rs:284`). So the gap is one constructor, not a modelling
gap, and disjunctions are covered too by `HandConstraint::any`
(`src/hand_constraint.rs:254-270`).

The compiled form costs nothing extra at check time: the range test is exactly
what `pre_check` already does, and the bitset is the fallback. What the
pattern-string form charges is authoring effort — a four-suit box such as
H 5-8, S 2-3, D 4-5, C 0-3 is 4x2x2x4 = 64 strings — which is where a
length-range constructor would earn its keep.

History agrees with that shape of conclusion, but not with "length ranges were
slower, so we dropped them". `LenRange` was not removed: commit `6c319bb`
("refactor: deepen deal constraints and unify parallel domain types",
2026-09-19) says `HcpRange` and `LenRange` "become newtypes over a shared
`Bounded<const MAX: u8>` core", and `Shapes::from(&[LenRange; 4])` survives
(`src/shape.rs:767`) with its doctest intact. What that commit did collapse is
the nine-layer API above it — "`DealerBuilder` -> `HandDescriptor` ->
`HandType` -> `Shape`/`Shapes` -> `shapeparser`" — and it made "the `min`/`max`
HCP cache ... internal implementation detail". Its own profiling note is that
Polish Club throughput stayed "unchanged (1.20 vs 1.22 us/deal)", not that ranges
were slow. The only measured constraint-check regression in the same area is the
discarded `high_card_points` mask refactor, 1.2839 ns vs 1.2749 ns
(`docs/performance-optimization.md:196-204`). So the missing piece is an API
surface that the deep-module pass intentionally narrowed, not a representation
that lost a benchmark.

### Side by side

| Concern | Ours | Theirs |
|---|---|---|
| Unit of constraint | One `Hand`, one seat at a time: `pub fn check(&self, hand: Hand) -> bool` (`src/hand_constraint.rs:300-306`) | One seat: `SeatKnowledge` per seat (`knowledge.rs:119-143`) |
| HCP | Yes, `HcpRange` (`hand_constraint.rs:230-234`) | Yes, narrows (`knowledge.rs:120`) |
| Suit lengths | Expressible as pattern unions (`25+xx`, `35+xx`) or by compiling a `[LenRange; 4]` box through `Shapes::from`; no public `HandConstraint` constructor takes either, and the branch has no runtime range representation — it is a 1,379-bit table (`src/shape.rs:58,66`) | Yes, `len: [Range; 4]` (`knowledge.rs:122`), including sums/relations |
| Balance | Yes, the 4333/4432/5332 families (`add_balanced`, `hand_constraint.rs:386-394`) | Yes, `Tri` (`knowledge.rs:123`) |
| Points beyond HCP | No | Yes, `pts`, `tp`, `dp` (`knowledge.rs:126-134`) |
| Honours / quality / controls | No | Recorded, not readable (§2) |
| Per-deal predicate | `AcceptFunction = Box<dyn Fn(&Deal) -> bool + Send + Sync>` (`src/record/deal.rs:88`), installed by `with_function` (`:1202`) | None; `consistent_hands` is private and pool-bound |
| Generation | Rejection sampling, `attempt_budget` default 2,000,000 (`src/record/deal.rs:98,1600-1641`) | None |
| Conditioning that is exact | Known cards are folded into the hand before filling (`known + cards_to_add`, `:1628`), so never rejected (`Dealer::known`, `:1464-1469`) | n/a |
| Cross-seat coupling | Only `AcceptFunction`; nothing else sees partner's hand (`:88,1202,1633`) | Per-seat marginals plus a deck cap: HCP ≤ 40 − others' minima, lengths ≤ 13 − others' minima, iterated (`position.rs:317-358`) |

### Could an auction become one more predicate?

**Mechanically yes, and for shape and HCP it fits `HandConstraint` as it stands;
the rest of what an auction shows needs more than that type can hold.**

- The seam already exists and is exactly the right shape:
  `constraints_respected` runs per seat (`src/record/deal.rs:1655-1666`) and
  `AcceptFunction` receives `&Deal` once per candidate (`:1633`). A closure that
  holds four `SeatKnowledge`-equivalent range sets and tests each seat would drop
  straight in.
- It *can* be a `HandConstraint` for the HCP-and-shape part, including the shape
  output an auction actually emits: "5+ hearts, 2–3 spades, 15–17" is
  `new(&["25+xx", "35+xx"], HcpRange::new(15, 17))`. The friction is authoring,
  not expressiveness: a four-suit range box must be expanded to strings, and
  `from_compiled` is private. A public length-range or `Shapes` constructor would
  remove that friction without adding a second runtime representation.
- What cannot be a `HandConstraint` is everything the shape+HCP table cannot
  hold: `pts`, `tp`, `dp`, honours, controls, keycards, and any condition over two
  seats at once. Those need either more summary dimensions (cheap, still O(1)) or
  the Deal-level `AcceptFunction`.
- Generation strategy does not need to change: ours is already rejection sampling,
  and the accept hook is already the intended escape hatch for cross-hand logic
  ("combined HCP", "NS 8+ spades" in `benches/deal_constraints.rs:52-67`). What
  changes is the *acceptance rate*: four seat-marginal constraints multiply, and
  the tightest auctions (a 1NT opener plus a 3NT responder plus two silent
  opponents, all with ranges) are exactly where rejection sampling degrades. That
  number is unmeasured (§ "What this note does not settle").
- Simulation needs no new machinery: `generate_deals` calls `dealer.deal()`
  sequentially before any parallel solve (`src/simulation/solver.rs:70-80,111,148`),
  so conditioning reaches every simulation through the `Dealer` value already
  passed to each builder. Known cards and auction ranges are orthogonal: Known
  cards shrink the deck, ranges test the completed hand.

**So:** the shape/HCP part is not a new module at all — it is the existing
constraint with a wider front end, and the generation loop does not change. The
full `SeatKnowledge` model still needs a home for points, honours, controls and
keycards, and that is where either a small extension to `HandSummary` or a
Deal-level predicate is required.

## 5. Dependency shape

| Crate | Kind | Direct non-workspace deps | `build.rs` | Native step |
|---|---|---|---|---|
| `bidspec` | lib | `bridge-card` (git, tag `v0.15.1`), `serde`, `serde_json`, `toml` | none | **none** |
| `rbb-engine` | lib | `bridge-types` (git), `bridge-card` (git), `bidspec`, `serde`, `serde_json` | none | **none** |
| `rbb-compare` | lib | the two above + `bridge-encodings` (git), `bridge-solver` (git), `rayon`, `toml` | none | none by default, but pulls a solver |
| `rbb-assets` | lib | `bridge-card` | **yes** | none, but layout-bound |
| `rbb-cli` / `rbb-workbench` / `rbb-wasm` | bins/libs | `clap`, `egui`/`eframe`, `wasm-bindgen` | none | none |

Evidence: `RBB/Cargo.toml:1-25`; `RBB/crates/bidspec/Cargo.toml:9-14`;
`RBB/crates/engine/Cargo.toml:9-14`; `RBB/crates/compare/Cargo.toml:9-19`;
`RBB/crates/assets/Cargo.toml:8`.

- **Git dependencies, not vendored.** `bridge-types` and `bridge-card` come from
  `github.com/bridge-craftwork` (`RBB/Cargo.toml:20-21`); a `rbb-compare` consumer
  adds `bridge-encodings` and `bridge-solver` (`compare/Cargo.toml:14-15`), four
  distinct repositories. `bridge-card` is pinned to tag `v0.15.1`; `bridge-types`
  has **no tag or branch**, so the manifest points at a moving default branch
  (the lock pins `0206ec5…`). None of these repositories is in our tree.
- **Pure Rust by default: yes for the two crates that matter.** Neither
  `bidspec` nor `rbb-engine` has a `build.rs`, no `[build-dependencies]` exists
  anywhere in the workspace, and their crates.io deps are `serde`, `serde_json`,
  `toml`. `bridge-solver` (only reached from `compare`/`wasm`) is declared with
  no features (`RBB/crates/compare/Cargo.toml:15`), and the copy of that
  repository in our tree returns from its build script unless the
  `dds-reference` feature is set (`vendor/bridge-solver/build.rs:16-17` — a
  different commit from RBB's lock, so indicative rather than authoritative).
- **`rbb-assets` is optional and not relocatable.** It reads
  `CARGO_MANIFEST_DIR/../..` plus `conventions/`, `cards/bbsa/`, and the workspace
  `Cargo.lock` (looking for a `bridge-card` entry) (`RBB/crates/assets/build.rs:53-66,146-152`),
  and emits `include!`d Rust (`RBB/crates/assets/src/lib.rs:14`). We would not use
  it: `rbb_engine::load_rules(dir: &Path)` walks a directory at runtime
  (`RBB/crates/engine/src/lib.rs:112-120,165-206`).
- **The rules are separable; the mapping is not separable from the chooser.** The
  data is plainly separable: 106 `.bid` files, a `conventions.toml` manifest,
  plain-text `.bbsa` cards. But `advance`/`interpret` are methods on `Engine`
  (`engine.rs:483,505`), `advance_from` needs `Engine::cands_at` and the per-side
  `System` and `params`, and `Engine::new` requires two cards
  (`engine.rs:162-171`). `Position` and `SeatKnowledge` are standalone public
  types (`position.rs:76-108`; `knowledge.rs:119`), so *state* is reusable without
  choosing, but *interpreting* is not.
- **What a card costs us.** Interpretation needs a `bridge_card::Card` per side.
  Their stock cards are BBA-derived (`21GF-*`, `Basic-Bridge`, `Precision*`), and
  `.bbsa` import lives in `bridge-card` (`RBB/crates/cli/src/main.rs:1278-1285`).
  A BBO/LIN auction whose pair's system we do not have a card for is read under
  whatever card we supply — which changes its meaning (§1).
- **Licence:** MIT OR Apache-2.0 (`RBB/Cargo.toml:16`; `RBB/LICENSE-MIT:1-3`;
  `RBB/README.md:39-44`). The `.bbsa` cards are described as copies of
  `Practice-Bidding-Scenarios/bbsa/` with no notice file
  (`RBB/cards/README.md:7-8`); rule provenance is per-file "Sources" sections
  (104 of 105 `.notes.md`), which say "standard practice, not yet cited to a book
  or article" where that is the case (`RBB/conventions/notrump/stayman.notes.md:539`).
  No per-file SPDX headers under `conventions/` or `cards/`.
- **Stability:** workspace version `0.10.0` (`RBB/Cargo.toml:14`), README status
  "early design" (`RBB/README.md:10`). The **rule language** has a versioned
  contract (`LANGUAGE_VERSION = 1`, `RBB/crates/engine/src/contract.rs:13`;
  `RBB/docs/CONTRACT.md:200-215`), but there is no Rust API stability statement,
  no `publish = false` and no crates.io publication claim — "not stated". The
  manifest drift (`engine = "0.7.0" (unreleased)`,
  `RBB/conventions/conventions.toml:12,6`) against a 0.10.0 workspace is a
  documentation smell, not a build break.

## What this note does not settle

- **Whether it builds offline from this checkout.** The two git dependencies are
  not vendored, `bridge-types` has no tag, and the nested checkout is untracked.
  Nothing here was compiled; a build may need the GitHub git cache populated.
- **The joint acceptance rate on our dealer.** Four per-seat marginals are
  necessary conditions, so conjunction is still sound; nothing in the tree says
  how many random Deals pass all four at once, and that number — not the
  per-predicate cost — decides whether rejection sampling survives.
- **The number of residual `Expr`s (`R`) per seat after a real auction.** No doc
  or test states it; structurally it is one `shows` plus one `Not` per outranking
  candidate per call, minus what `held_by_ranges` drops, plus the deck caps.
- **Warm single-auction latency.** The 20 s / 20,000-board figure is parallel
  wall-clock on a shared machine. Nobody has isolated one `interpret` call.
- **Whether the residual `constraints` are worth reimplementing.** We know they
  are public data and a private evaluator; we do not know how often a real auction
  leaves anything there that is not already a range.
- **Whether `HandConstraint` wants a length-range front end.** Expressiveness is
  not in question (§4); the open questions are authoring ergonomics versus the
  cross-product of pattern strings, and whether exposing one `Shapes`-taking
  constructor is enough to keep the single fused membership check.
- **Whether the soundness claim holds for auctions bid under other systems.** It
  is sound relative to their rules and card (`RBB/docs/LANGUAGE.md:541-546`);
  soundness against a human pair's actual agreements is a modelling question the
  tree does not address, and the top-priority reading rule
  (`RBB/docs/CONTRACT.md:365-373`) cuts the other way.
- **Card availability.** Which of their stock cards (if any) matches the systems
  behind the auctions we actually want to condition on is unstated, and the card
  is not optional (§1).
- **Whether their `balanced`/point-count conventions (`general.style`, tens,
  length points) agree with ours.** `Valuation::for_card` reads the card
  (`RBB/crates/engine/src/facts.rs:45-52`) and the docs argue about ½ vs 1 point
  per length card (`RBB/docs/LANGUAGE.md:369-379`). A range borrowed from their
  model is only as good as the agreement between the two point counts.

## Recommended next step

**`/prototype` — measure one real auction against our dealer.** The decision rests
on two facts that argument cannot settle and one prototype can: (1) whether the
crate builds from the untracked checkout without network access, and (2) what
four seat-marginal range sets do to our acceptance rate and per-Deal cost.
Concretely: take one complete auction (e.g. a 1NT–3NT or a 2/1 game force) with a
card that describes it, call `interpret` **once**, keep the four `SeatKnowledge`
ranges, compile each seat's HCP and length ranges into a `HandConstraint` with
pattern unions (`25+xx`-style, as §4 shows) and attach it with
`with_hand_constraint`, then measure (a) build yes/no, (b) `Dealer::deal()` time
and attempts against the 143 ns–1.8 µs baseline in
`docs/performance-optimization.md:127-137`, (c) the share of `Deal`s that pass all
four seats at once. No `src/` change is needed for this: the pattern-string path is
public, and anything the strings cannot say (points, honours) is exactly what the
note says does not fit. If the build gate fails on the unvendored git
dependencies, the answer is **drop it**; if acceptance is workable, the finding is
a capability and the follow-on is `/grill-with-docs` → `/to-spec` → `/to-tickets`.
