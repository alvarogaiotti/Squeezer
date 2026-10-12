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
  seats do not follow the contract's leader/`trick_winner` rotation, also cannot
  round-trip through LIN. A PBN board writer (deferred) could carry the contract
  explicitly.
- The PBN board writer remains deferred: only `Deal::as_pbn`/`write_pbn` (the
  hand record) exist, and `Board` has no `as_pbn`/`write_pbn`. A future PBN
  parser should land the board writer beside it.
- `linparser` needs a derivable contract to attribute `pc|` seats; a board with
  a play but no auction fails at import rather than guessing a leader. The
  `PlayRecord` type itself is contract-free; this is only the importer's
  constraint.
- `linparser::ParseLinErrorKind` is a categorised taxonomy currently consumed
  only by the importer's own tests; the crate-wide boundary could absorb it.
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

### Dealer and conditioning
- `DealerBuilder` keeps two sources of truth for the available deck: `deck`
  (incremental, for eager duplicate rejection in `predeal*`) and the `known` set
  recomputed in `build()`. Unifying them would move `predeal`'s error to build
  time (relied on by a should-panic test).
- The bounded rejection budget default (`2_000_000`) is tuned to the tightest
  shipped test constraint (mean ~10⁵ attempts); rarer constraints — notably the
  untested `utils::dealer_of_lavazza_3nt_opening` — can now return
  `Unsatisfiable` where the old unbounded sampler would eventually succeed.
- `Dealer::predealt()` has no in-crate reader, and `Dealer::current_hand()` has
  only a test reader (its simulation caller was deleted with
  `CardPayoffSimulation`). Both are kept as public API.

### Simulation, card-EV and contract payoff
- `LeadAnalysis` is a thin view over a borrowed `CardEvaluation`, but
  `LeadSimulation::run` still materialises the legacy `RawLeadData`/
  `LeadStatistics` so the formatters, CSV/chart helpers and the bench keep
  working. A later ticket could port those consumers onto `LeadAnalysis` and
  delete the legacy types; `formatters::export::lead_*` and the `charts` module
  also duplicate the percentile/histogram logic now in `card_ev.rs`.
- Public-output change (ticket 10): `LeadStatistics.cards`/`RawLeadData.cards`
  now hold every **physical** legal lead (13 for example hand 0) instead of the
  DDS equivalence-class representatives (9), so rankings and any length-based
  consumer shifted. Pinned by
  `lead_view_lists_every_physical_card_not_just_class_representatives`.
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
