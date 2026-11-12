# SESSION.md

## Open issues

### Backend (`dds`)
- The `dds` crate's own `ParCalculator` has default method bodies but no `impl`
  for any solver, so it cannot compute par contracts. `CalcAllTables` also
  discards its `AllParResults` out-param.
- `dds/src/ddserror.rs`: unknown DDS return codes fall back to
  `DdsErrorKind::UnknownFault`, whose `Display` text is "fopen() failed or
  wrong number of boards". `RETURN_TOO_MANY_BOARDS` (-101) and
  `RETURN_THREAD_MISSING` (-104) hit this fallback and print a misleading
  message. Add explicit variants/messages.
- `ConstructDdsDealError::DuplicatedCard` is never constructed: the deal
  builder does not detect duplicated cards.
- `DdTableDealPbn` and `DdTableDealsPbn` in `dds/src/tables.rs` have no callers;
  they could go with the rest of the PBN table API later.
- `dds::context`'s `dds_c_calc_dd_table`/`dds_c_calc_par` output
  `DdTableResults<NotPopulated>`; no public transition advances the type-state
  marker, so callers read `res_table` directly. A `populated()`-style
  transition would help.
- `bindings::ddsffi::MAXNOOFBOARDS` is retained but has no Rust consumer now
  that the batch surface is gone.
- Design ticket 04 is ambiguous about the DDS batch entry points: it lists them
  under "Dropped" *and* says they are "kept as a documented future throughput
  option". Implementation ticket 04 deleted them. A later reader wanting the
  batch path back should re-bind them from the vendored 3.1 sources.
- The `dds` crate's legacy `DdTableCalculator for DoubleDummySolver` impl
  (`CalcDDtable`/`CalcAllTables`) has no caller and lost its direct tests when
  they moved to the parallel-safe `dds_c_calc_dd_table` shim. Its first real
  consumer will be the seam's `DdTableCalculator` (still unimplemented).
- `dds/src/deal.rs` documents `DdsDeal::current_trick_rank` and
  `DDSCurrTrickRank` as rank *bit-flags*, but DDS reads them as sequential
  ranks 2..14. The code is right and the docs are wrong.

### Solver seam
- The blanket `impl<S: Solver + Send + Sync> ContextFactory for S` gives every
  concrete solver two `new_context` methods with different return types, so
  `my_solver.new_context()` is ambiguous and callers must write
  `Solver::new_context(&my_solver)` or go through a `Box<dyn ContextFactory>`.
  The shared name is mandated by ADR-0005; a future rename of one would remove
  the footgun.
- The seam's cold capability traits have no implementor: `PlayAnalyser` is now
  wired for `DdsSolver` (ticket 13), but `DdTableCalculator` and `ParCalculator`
  are still unimplemented, so no DD table or par is reachable through the seam.
- `SolverPool` and its dyn sibling `DynSolverPool` reuse one context per rayon
  *job*, not per worker: `map_init` calls the initializer for each chunk, so a
  stolen chunk creates an extra context and the design's "one context per
  worker, reused across the run" is only approximated. `with_min_len` keeps the
  count near the worker count and `workers × TT-cap` still bounds
  *instantaneous* memory, but true per-worker reuse would need a
  thread-local/per-worker context table.
- `DynSolverPool` duplicates `SolverPool`'s rayon-pool plumbing (thread count
  clamping, `with_min_len` chunking, `try_map`). A `PoolFactory` trait with an
  associated context type would collapse the two, at the cost of a blanket impl
  for `Box<dyn ContextFactory>`.

### Record, `Position`, `Board`
- `Playthrough::new` starts at index 0 (before the opening lead); the design
  never fixed a starting index, so a later caller could want the cursor to begin
  at the current play.
- `CardEvaluation.position` is the *first* sampled deal's `Position`, cloned
  rather than shared. Positions are identical across deals (the prefix is
  deal-independent), so this is only a clone cost.
- `Position::after_play`/`from_board`/`Playthrough` inherit `from_deal_prefix`'s
  prefix validation, so a play record inconsistent with the deal (a card not
  held by its seat, a card played twice, or a non-rotating table) is rejected
  at derivation as well as at `Board` import. Fine for board-derived positions;
  worth knowing for direct callers.

### Contract vocabulary
- `ContractError`'s `Display` keeps the pre-move wording verbatim ("no contract
  dealer", "Not a strain.", "Wrong contract level") because ticket 09 changed
  the error's type, not its output. `CONTEXT.md` calls the role the *declarer*,
  so a later cleanup should align the strings and their casing.
- `Contract::strain()` carries `#[allow(clippy::cast_possible_truncation)]` but
  performs no cast.
- The LIN importer derives seats from the `md|` digit and the PBN importer maps
  `[Dealer]`/`[Declarer]` through `Seat::try_from(...).ok()`, so the new
  `SeatError` payload is discarded and reported as the parser's own error kind.

### Card types and shape
- `Card::from_u8` is `pub(crate)` in `squeezer-cards` with no caller; it is kept
  only as the counterpart of the public `Card::as_u8`, and `dead_code` hides it.
- The `squeezer-cards` samplers are behind the `rand` feature, so
  `cargo test -p squeezer-cards` (no features) never exercises `FastSampler` or
  `UniformSampler`; only the library's dealer tests do. A card-crate sampler test
  would need `--features rand` to run in CI's feature matrix.

### Model crate
- `AsDDSContract` and `ContractScorer` for `Contract` were dead impls in the
  root and are deleted rather than re-homed (ticket 11): `Contract` now lives in
  `squeezer-model`, so both the trait and the type are foreign to the root. The
  solver ticket must reintroduce them through local newtypes if a caller needs
  them. `impl From<Strain> for DdsSuit` became the free function
  `dds_bridge::dds_strain`, and `impl From<Seat> for DdsHandEncoding` became
  `dds_bridge::dds_hand_encoding`.
- `shapeparser::validate_shape` returns the model-local `CreationShapeError`
  instead of `DealerError`; it has no caller, but the public signature changed.
- The root keeps a `squeezer::contract` shim while the prelude also globs
  `squeezer_model::*`, which re-exports the model's `contract` module. Both
  paths name the same items, but `squeezer::contract` and
  `squeezer::prelude::contract` resolve through different modules.

### Parsing and serialization
- `.gitignore`'s global `*.lin`/`*.pbn` rules left the seven `tests/*.lin`
  fixtures untracked, so a clean checkout could not run `tests/lin_parser.rs`
  and any new `tests/*.pbn` fixtures would hit the same trap. Negation rules
  `!tests/*.lin`/`!tests/*.pbn` now un-ignore them and the LIN fixtures are
  staged (ticket 01, PBN interchange).
- The quickcheck round-trip property's `AnyBoard` has no `shrink`, so a failure
  reports "No Arguments Provided" instead of the offending board.
- `linparser::LinMetadata` stores rooms as `rooms: Vec<Option<Room>>` parallel
  to `boards`, because `Board` has no room field and LIN's `qx|` is per-board.
- An empty `Auction` (`Some(Auction::new(vec![]))`) cannot survive a LIN round
  trip: the writer emits no `mb|` and the importer reconstructs `None`.
  `Board::new` permits it, so the model is wider than the format.
- `LinFile::write_lin` comma-joins `vg|`/`rs|`/`pn|` fields and the importer
  splits on `,`, so a value containing a comma re-parses as extra fields. LIN
  has no escaping, so this is inherently lossy; documented on the writer.
- A `Board` with `contract: Some` but `auction: None`, or a `PlayRecord` whose
  seats do not follow the contract's leader/`trick_winner` rotation, cannot
  round-trip through LIN. The PBN board writer carries the contract explicitly,
  so that part round-trips through PBN instead.
- `Board::new` normalises an empty `PlayRecord` to `None` but keeps an empty
  `Auction` as `Some(Auction::new(vec![]))`: the two empty representations are
  asymmetric. Neither writer emits a section for the empty case, so an empty
  auction cannot round-trip. The PBN importer also normalises an empty
  `[Auction]` body to `None`, and the PBN round-trip generator only builds
  auctions through the helper that returns `None` for zero calls, so neither
  property covers the asymmetry.
- PBN import format has no mandatory tags (PBN 2.1 §3.4). The importer still
  requires `[Deal]` (a board needs cards); a missing `[Dealer]` falls back to
  the `[Deal]` first letter and a missing `[Vulnerable]` defaults to
  `Vulnerability::None`. An explicit tag always wins, even when `[Dealer]`
  differs from the `[Deal]` letter.
- The PBN importer tokenises a section body on whitespace, so an annotation
  pasted onto its call or card (`1S!`, `HK=1=`) is one token and fails as a
  call/card error. Only whitespace-separated annotations are ignored; PBN's
  self-delimiting tokens would need a section tokenizer, not a `split_whitespace`.
- The auction parser does not detect the natural end of an auction (a pass-out),
  so calls after it are consumed: `Pass Pass Pass Pass 1S` imports as a 1S
  contract instead of a passed-out board. The writer treats a bid plus at least
  three trailing passes as complete and emits no `*`, so a completion rule must
  be shared with the writer (and may require narrowing ticket 05's generator)
  before the importer can reject or ignore the extras.
- A `*`-cut auction that stops before the final contract is rejected when the
  `[Declarer]`/`[Contract]` tags name a later contract. PBN 2.1 allows the tags
  to state the outcome of a cut auction; `Board::new` requires the stored
  contract to equal the auction's own result. A contracted board always
  satisfies that equality, so the writer's output is unaffected; a contractless
  board with an incomplete auction gains a derived contract on re-import, which
  is a ticket 05 representability gap.
- PBN irregularity tokens (`^I`, `^S`, `^R`, `^L`) are unknown call/card tokens
  and fail at import. `^R` (a revoke) is card data the model could represent,
  since `Board` deliberately does not validate revokes; only the other three
  need to stay rejected.
- An empty `[Auction]` body imports as `None`, not `Some(Auction::new(vec![]))`,
  mirroring LIN: neither writer emits an empty section, so the empty auction
  representation still cannot round-trip.
- The PBN round-trip generator (`AnyPbnBoard`) repeats `arbitrary_board`'s
  assembly with a narrower policy (no claim, plain calls, derived contract). A
  third interchange format would duplicate it again; a shared representability
  policy (or one parameterised builder) would collapse the copies.
- `arbitrary_auction` can emit calls after the auction's natural pass-out: it
  fixes the call count up front, so once three closing passes have ended the
  auction it still offers higher bids (and, from the other line, a double of
  the last bid) for the remaining slots, e.g. `1S Pass Pass Pass 1NT`.
  `Auction::validate` accepts them, the PBN writer's `auction_is_complete` sees
  only the trailing passes and emits no `*`, and the importer consumes every
  call, so the properties round-trip — but a third-party PBN reader would stop
  at the pass-out and discard or flag the extras. Rejecting a call after a
  pass-out in `Auction::validate` is the principled fix.
- `AnyPbnBoard` has no `shrink`, like `AnyBoard`, so a PBN round-trip failure
  reports "No Arguments Provided" rather than the offending board.
- `linparser` needs a derivable contract to attribute `pc|` seats; a board with
  a play but no auction fails at import rather than guessing a leader. The
  `PlayRecord` type itself is contract-free; this is only the importer's
  constraint.
- `linparser::ParseLinErrorKind` is a categorised taxonomy currently consumed
  only by the importer's own tests; the crate-wide boundary could absorb it.
- `.opencode/` is untracked and not ignored, so a broad `git add -A` would put
  local agent tooling into a feature commit; it belongs beside `/.scratch/` in
  `.gitignore`.
- `ParseLinError` is public under `squeezer::linparser` but not re-exported by
  `squeezer::prelude`, unlike the other module errors.
- `benches/bench_payoff.rs` passed `"A AKjt Kqjt KQT9"` to `Cards::from_str`;
  the old parser silently dropped every lowercase rank letter, so the bench
  predealt 8 of the intended 13 cards. Ticket 15 made lowercase ranks valid, so
  the bench now predeals the full hand and its dealer/solve cost changes.
- `Hand::from_str` reports a malformed string as `DealerError::InvalidCards`
  (ticket 15) but a wrong card count as a free-form `DealerError::Message`, so
  the two failure modes are not distinguishable by variant.
- `Card::from_str` now accepts both `T` and `10` for the ten, but `Card`'s
  `rankchar`/`Display` only ever emit `T`, so `"10"` is an input-only spelling.

### BBO client
- `crates/squeezer-bbo/src/blocking.rs`'s ignored live test keeps a real BBO
  username and password in the repository. It should read them from the
  environment (`BBO_USERNAME`/`BBO_PASSWORD`) before anyone un-ignores it.
- Both clients' `download` discards the fetched LIN (a `TODO` in each), so the
  crate cannot yet hand the records to `squeezer-io` itself.
- The ignored live test in `crates/squeezer-bbo/src/blocking.rs` calls
  `get_in_interval` with a start time later than its end time (2022-04-01 vs
  2022-03-03). The client swaps the arguments, so it would still issue the
  request, but the interval is not the one the test names; re-check the dates
  before un-ignoring it.

### Dealer and conditioning
- `DealerBuilder` keeps two sources of truth for the available deck: `deck`
  (incremental, for eager duplicate rejection in `predeal*`) and the `known` set
  recomputed in `build()`. Unifying them would move `predeal`'s error to build
  time (relied on by a should-panic test).
- The bounded rejection budget default (`2_000_000`) is tuned to the tightest
  shipped test constraint (mean ~10⁵ attempts); rarer constraints — notably the
  untested `examples/lavazza_3nt.rs` dealer — can now return
  `Unsatisfiable` where the old unbounded sampler would eventually succeed.
- `utils::weak2_hand_descriptor` is still a library demo predicate after the
  Polish Club, Lavazza and Zar helpers moved to `examples/`; either move it too
  or rename it, since `*_descriptor` now names a deleted type.
- `HcpRange::new(5, 3)` clamps to `(3, 3)` (via `Bounded::new`), while
  `HcpRange::from((5, 3))` and `HcpRange::from_range(5..=3)` swap to `(3, 5)`.
  Both keep `min <= max`, but the two spellings describe different windows;
  route every constructor through one normalization helper.
- `Dealer::predealt()` has no in-crate reader, and `Dealer::current_hand()` has
  only a test reader (its simulation caller was deleted with
  `CardPayoffSimulation`). Both are kept as public API.

### Simulation, card-EV and contract payoff
- `LeadAnalysis` is a thin view over a borrowed `CardEvaluation`, but
  `LeadSimulation::run` still materialises the legacy `RawLeadData`/
  `LeadStatistics` so the terminal formatters and the bench keep working. A
  later ticket could port those consumers onto `LeadAnalysis` and delete the
  legacy types.
- Public-output change (ticket 10): `LeadStatistics.cards`/`RawLeadData.cards`
  now hold every **physical** legal lead (13 for example hand 0) instead of the
  DDS equivalence-class representatives (9), so rankings and any length-based
  consumer shifted. Pinned by
  `lead_view_lists_every_physical_card_not_just_class_representatives`.
- `formatters` is not a default feature, so the formatter doctest only runs
  under `--features formatters`. CI's plain `cargo test --workspace` covers 56
  of the 57 doctests until ticket 16 adds the feature matrix.
- `Denomination::difference` with `Baseline::Fixed` is not antisymmetric across
  `(a, b)`/`(b, a)`; `PayoffMatrix::calculate_rankings` only aggregates
  `option_a`, so a `Fixed` denomination can skew rankings. Documented as
  unsupported in the pairwise path, but nothing enforces that at runtime.
- Contract payoff exposes the `Denomination` only through
  `PairwiseComparison.difference`; `DealResult` still carries `score` but no
  per-contract `value` (unlike card-EV's `CardDealOutcome.value`).
- `CardEvaluation` reports board/position derivation failures as
  `SqueezerError::InvalidInput` carrying the per-deal index, not as the typed
  `BoardError`/`PositionError` boundary variants.
- `CardEvBuilder::seat` is ignored for an empty play trace: the opening
  position always uses `contract.leader()` (now via
  `from_deal_prefix_with_contract`). Passing `.seat()` with no trace silently
  does nothing.
- `CardStatistics::from_outcomes` is the dominant remaining simulation-only
  hotspot (38.64% of the mock card-EV profile, with the public
  `BTreeMap<i32, usize>` `score_distribution` insert at 13.97% of the
  remainder). It runs once per card per simulation, so it is ~0.0008% of a
  native 1,000-deal lead run and was left unoptimised deliberately. Any fast
  path must either change the public map type or assume `score` is a function of
  `declarer_tricks`, which the public constructor cannot assume for
  caller-built `CardDealOutcome`s.
- End-to-end simulation timings are ~100% native DDS, so Rust-side wins only
  show up in the mock-solver benchmark `benches/simulation_paths.rs`. A real
  1,000-deal lead iteration is ~44.8 s and payoff ~7.17 s; post-processing is
  0.35 ms and 0.40 ms respectively. See `docs/performance-optimization.md`.

### Analysis and miscellaneous
- `analyze_board` bounds its backend with `SqueezerError: From<A::Error>`, so a
  third-party `PlayAnalyser` with a local error type cannot use the convenience
  entry point without a `From` impl.
- `PlayerRecord::plays()` yields an iterator while the `plays` field is private,
  so index-based assertions have to use `.nth()` rather than `[i]`.
- DDS caps `SolvedPlay.number` at 49 even for a full 52-card play; the seam
  repeats the last count to cover the forced final trick.
- `PreprocessedDeal` is a 48-card type unrelated to `analyze_hand`, which uses a
  52-card scratch buffer; the module-level SIMD/preprocessing framing no longer
  describes the analysis path, and `find_trick_winner` is benchmark-only.
- `find_trick_winner_naive` now delegates to the canonical
  `record::position::trick_winner` and ignores its `lead_suit` parameter, which
  is redundant with `original_cards[0].suit()` for every caller. The canonical
  rule's `max_by_key` keeps the *last* equal-strength play, whereas the old fold
  kept the first: irrelevant for a real, distinct-card trick, but a behavior
  difference on degenerate duplicate-card input such as the random bench.
- `Cards::insert` and related bit operations trust `Card::offset`; `Card::JOKER`
  has offset 255, so inserting it shifts by more than 63 and panics in debug
  builds. The HCP fast path intentionally treats the otherwise-invalid bit 63
  as zero, matching the old set-bit walk.
- `performance_analysis::analyze_hand` panics when a complete trick lacks DD
  results, but silently analyzes only the available prefix of a short final
  trick. The allocation-free implementation preserves that asymmetry.
- `Cargo.lock` is git-tracked yet also listed in `.gitignore`; the ignore rule
  has no effect on tracked files, which is confusing.
- Pre-existing clippy warnings: `decimal_bitwise_operands` at
  `src/performance_analysis.rs` (test code), plus ~10 warnings in the `dds`
  crate (`cast_possible_*`, `large_stack_arrays`, …).

## Resolved

- The card vocabulary (`Suit`, `Card`, `Cards`, `InvalidCardError` and the
  samplers) lives in `squeezer-cards`, which names no workspace crate and no
  DDS type (ticket 10). The two DDS adapters became root-side code in
  `dds_bridge`: a local `DdsCardAdapter` newtype implements the foreign
  `AsDDSCard` trait, and `dds_suit` is the free function replacing
  `From<Suit> for DdsSuit`.
- `impl TryFrom<usize>`/`impl TryFrom<i32> for Suit` now return the
  card-local `InvalidSuitError` instead of the dealer error, with the same
  `Display` text (ticket 10). `TryFrom<&[Card]> for Cards` was deleted rather
  than moved: its error was the root boundary `SqueezerError`, which cannot be
  named from the cards crate, and it had no caller or test.
- `Deal::as_pbn`/`write_pbn` stayed source-compatible while becoming
  dealer-aware: `as_pbn_from(first)`/`write_pbn_from(first)` take the first
  listed seat and the old `as_pbn()`/`write_pbn()` delegate with `North`, so
  the existing golden test and `test_pbn_format` were untouched (ticket 02,
  PBN interchange).
- Benches/tests vs feature gates: `[[bench]]`/`[[test]]` targets that need
  the solver or LIN parser now carry `required-features`, so
  `cargo clippy --all-targets` passes on every feature set
  (`bbo`-only, `bbo_async`-only, `dds,lin`, no-default).
- `dds/src/analyseplay.rs`: `PlayTraceBin::iter` sliced `rank` from `suit`,
  `PlayTracesBin::is_empty` was inverted, and `PlayTraceBinIter::next` never
  advanced its index. All three fixed with regression tests (ticket 02).
- `dds/src/ddserror.rs` is now rustfmt-clean: the file was rewritten for the
  `thiserror` migration in ticket 02, so the pre-existing fmt drift is gone.
- `linparser` malformed-input handling: `parse_claim`/`parse_vugraph_data`/
  `parse_room_and_board_number` no longer swallow input; the parser fails with a
  categorised `ParseLinError` (ticket 07).
- The LIN DTOs (`LinDeal`, `LinBidding`, `RecordedBid`, `PlaySequence`) and the
  `LinDeal -> Deal`/`Board` shims are deleted; the parser builds
  `Board`/`Auction`/`RecordedCall`/`PlayRecord` directly (ticket 07).
- `linparser::compute_contract` is gone; the parser derives the contract via
  `record::auction::Auction::contract` (ticket 07).
- The `md|` dealer/hand-order mismatch is fixed: the writer emits the stored
  `Board.dealer` and hands `S,W,N,E`, matching the importer (ticket 08).
- `DealError::MissingCard` was unreachable (13 cards per seat plus
  pairwise-disjoint hands imply all 52 are present) and was deleted; the
  deprecated `Error::description` impl was already absent (ticket 05/12).
- `CardPayoffSimulation` (and its `play_trace` field, trace-to-prefix
  reconstruction, `vulnerability_for` board construction, and the `13 - score`
  perspective edge) is deleted; card-EV uses the perspective rule
  `if to_play.is_same_line(declarer)`. Empty `PlayRecord` normalises to `None`
  (ticket 10).
- `performance_analysis` is ported onto `Board`/`PlayRecord`/`PlayAnalyser`;
  the DD-relative names are `DdOptimal`/`dd_tricks_lost`/`dd_*` (no more
  `is_optimal`/`CardResult::Optimal`), and
  `TryFrom<&PlayRecord> for PlayTraceBin` has its caller (ticket 13).
- The `Denomination` non-`Clone` question is settled: `ContractPayoffBuilder`
  and `ContractPayoffSimulation` store the denomination by value and never clone
  it (ticket 11).
- `Position` is publicly constructible from a `Deal` + play prefix + `Strain`
  (`Position::new`, `from_deal_prefix`, `from_deal_prefix_with_contract`); the
  `pub(crate) from_parts` shim and `simulation::solver::vulnerability_for` are
  gone, `CardEvaluation` no longer assembles a `Board` per sampled deal, and
  `record::position::trick_winner` is the single rule that
  `find_trick_winner_naive` delegates to (ticket 14, ADR-0007). `Position::new`
  validates the snapshot invariant (table ≤ 3, per-seat hands ≤ 13, distinct
  union, `(52 - |hands|) % 4 == table.len()`, consecutive table seats) rather
  than a 52-card equality, so a completed-trick position is representable.
- `Card`/`Cards` parsing requires suit-before-rank and rejects rank-first, bare
  ranks and unknown characters; `InvalidCardError` carries the rejected input,
  is wrapped by `SqueezerError::InvalidCard`, and surfaces via
  `DealerError::InvalidCards` (ticket 15).

## 2026-10-25 — incidental notes

- `docs/` had no `research/` directory; it now holds repo research notes
  (`docs/research/`), while ADRs stay in `docs/adr/`.
- `vendor/Dealer3/`, `vendor/bridge-solver/` and `vendor/rusty-bidding-bot/` are
  untracked nested git checkouts, not submodules: `.gitmodules` lists only
  `vendor/dds`, so a clean clone does not get them and `git status` shows them
  as untracked.
- `benches/bench_leadsim.rs:28` names its criterion benchmark "Payoff
  benchmark", copied from `benches/bench_payoff.rs:33`; the two benches are
  indistinguishable by name in criterion output.
- `benches/simulation_paths.rs` runs entirely on a mock `FixedContext` and its
  body names no DDS type, yet `Cargo.toml:79-82` requires `dds` for that bench;
  the feature is required for a bench that never touches the backend.
- The crate doc claims "~10-50 microseconds" with shape constraints
  (`src/dealer.rs:891-892`); the measured medians are 143 ns-1.79 µs
  (`docs/performance-optimization.md:127-137`). The doc is one to two orders out.
- In the vendored tree, `vendor/rusty-bidding-bot/docs/LANGUAGE.md:547-548` says
  an unmatched call gets a built-in natural interpretation, but the engine passes
  (`crates/engine/src/engine.rs:855`) and the same doc says so at `:670-672`.
  `docs/LANGUAGE.md:646-648` also says partner's keycards are untracked, while
  `eval.rs:646-681` tracks them for `we.keycards`.
- `HandConstraint` has no public constructor from `Shapes` or `[LenRange; 4]`, yet
  both inputs are public (`src/shape.rs:356,767,821`) and the bridge
  `from_compiled` is `pub(crate)` (`src/hand_constraint.rs:284`). Length-range
  constraints are therefore expressible only by writing out pattern strings
  (`25+xx`), which is an API-surface gap rather than a capability one.
- `vendor/rusty-bidding-bot/conventions/conventions.toml:12` declares
  `engine = "0.7.0" (unreleased)` against a 0.10.0 workspace, and
  `vendor/rusty-bidding-bot/crates/assets/build.rs:53-66,146-152` locates its
  inputs as `CARGO_MANIFEST_DIR/../..` plus the workspace `Cargo.lock`, so
  `rbb-assets` cannot be consumed outside that checkout's layout.
- Nothing in the repo measures candidate throughput: `attempts` is only
  surfaced on `DealerError::Unsatisfiable`, and `benches/deal_constraints.rs`
  times `dealer.deal()` alone, so the published ns/deal figures cannot be
  converted to ns/candidate (the rejection factor is unrecorded).
- `Cards::pick` hardcodes `FastSampler` (`src/card.rs:476`), so the public
  random-hand API cannot ask for `UniformSampler`; the only uniformity
  comparison is a disabled test (`tests/uniformity_check.rs:5-6`).
- `vendor/Dealer3/performance-pretest-2026-01-07.md:11-16` still headlines Rust
  at 2.3x slower than C, while its own later section (`:255-260`) and
  `bench/results/CURRENT.md:34` report ~10% slower. Nothing points forward from
  the pre-test, so it reads as current.
- `Evaluator` (`src/evaluator.rs:25-53`) is exported but unreachable from the
  constraint layer; the doc comments at `src/dealer.rs:271` still
  claim per-seat constraints cover "controls", which no longer matches
  `ConstraintBranch { shapes, hcp }` (`src/hand_constraint.rs:123-128`).
- `Evaluator` has no `Debug`/`Clone`/serde derives, unlike every constraint type
  (`src/evaluator.rs:25` vs `src/hand_constraint.rs:123-136`), and
  `Evaluator::new` copies a caller slice with no length check
  (`src/evaluator.rs:38-42`), so more than 13 values indexes past `vals`.


## 2026-10-25 — vendor/bridge-solver follow-ups

Found while reading the vendored solver for
`docs/research/bridge-solver-vs-our-seam.md`; none blocks anything today.

- `vendor/bridge-solver/src/analyse_play.rs` derives `total` from
  `Hands::num_tricks()`, which is `hands[WEST].size()` (`src/hands.rs:58-60`), at
  `:193`, `:276` and `:399`. Any mid-trick or partially played input therefore
  computes every declarer total against a short `total`, and `total - ns` for an
  EW declarer can underflow. Harmless only because all three entry points are
  documented to receive a full un-played deal.
- `optimal_line`'s docstring says the caches "live for the whole walk" (`:381-383`),
  but every solve goes through `solve_position_ns`, which allocates a fresh cache
  pair per call (`:721-751`). Cost is one solve per candidate card, not one walk.
- `running_trace` indexes `keys[0]` and `keys[k + 1]` without checking
  `keys.len() == plays.len() + 1` (`:204`, `:234`, `:239`), so a short slice
  panics instead of returning an error.
- `vendor/bridge-encodings` returns `Result` from its LIN/PBN readers but never
  constructs `ParseError::Lin` (`src/lin/mod.rs:122-124`, `src/error.rs:12`), so
  the error channel is not a correctness signal.
- `vendor/bridge-encodings/README.md:133` documents `oneline::deal_to_oneline`
  with a two-argument signature; the function is `format_oneline(deal)` and
  `deal_to_oneline` does not exist anywhere in `src/`.
- `vendor/bridge-solver/src/cache.rs` is a one-line doc stub while `src/lib.rs:30`
  declares `mod cache;`; the real caches live in `search.rs` and `pattern.rs`.
- `AGENTS.md:46` says `performance_analysis` requires `dds` and `lin`. The module
  is gated on `dds` alone (`src/lib.rs:132`); `lin` is needed only by
  `tests/performance_analysis_test.rs` (`Cargo.toml:86-88`).
