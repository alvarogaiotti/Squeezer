# SESSION.md

## Open issues

- `src/bidding_rule_engine.rs` is dead (commented out at `src/lib.rs:114`)
  and would not compile as-is (`serde_yaml` absent). Slated for deletion in
  `.scratch/architecture-followups/issues/03-bbo-async-repair.md`.
- `BBOClient::download()` in both clients still discards the fetched LIN
  body (`let _lin = ...`); persisting it is a TODO.
- `dds` `ParCalculator` has default method bodies but no `impl` for any
  solver, so the parent crate cannot compute par contracts. `CalcAllTables`
  also discards its `AllParResults` out-param.
- `dds/src/ddserror.rs`: unknown DDS return codes now fall back to
  `DdsErrorKind::UnknownFault`, whose `Display` text is "fopen() failed or
  wrong number of boards". `RETURN_TOO_MANY_BOARDS` (-101) and
  `RETURN_THREAD_MISSING` (-104) hit this fallback and print a misleading
  message. Ticket 04 could add explicit variants/messages.
- `ConstructDdsDealError::DuplicatedCard` is never constructed: the deal
  builder does not detect duplicated cards, so the variant is dead.
- Ticket 04 deleted the PBN batch types with no callers (`DdsDealPbn`,
  `BoardsPbn`, `PlayTracePBN`, `PlayTracesPBN`). `DdTableDealPbn` and
  `DdTableDealsPbn` in `dds/src/tables.rs` still have no callers; they are part
  of the table/par surface ticket 04 kept, and could go with the rest of the
  PBN table API later.
- `dds::context`'s `dds_c_calc_dd_table`/`dds_c_calc_par` output
  `DdTableResults<NotPopulated>`; no public transition advances the
  type-state marker, so callers read `res_table` directly. Tickets 03/04 may
  want a `populated()`-style transition.
- `linparser` swallows malformed input instead of erroring: `parse_claim`
  and `parse_vugraph_data` use `unwrap_or(0)`, and
  `parse_room_and_board_number` `unwrap()`s the board-number parse. Relevant
  to ticket 02's goal of failing at import rather than yielding corrupt data.
- `linparser::compute_contract` still recomputes vulnerability from the board
  number (`Vulnerable::from_number_and_seat`) instead of using the parsed
  `sv|` vulnerability, so a non-standard vulnerability is silently dropped.
  It now also duplicates `record::auction::Auction::contract`; ticket 07 should
  delete the parser copy.
- `DealError::MissingCard` is unreachable: `Deal::new` checks each seat holds
  exactly 13 cards first, so once duplicates are ruled out the union is
  necessarily all 52 cards.
- `dds_bridge::TryFrom<&PlayRecord> for PlayTraceBin` has no caller yet; ticket
  07/13 should route the analysis port through `PlayRecord` and drop
  `PlaySequence`.
- The LIN DTOs (`LinDeal`, `LinBidding`, `RecordedBid`, `PlaySequence`) were
  kept in ticket 05 because deleting them forces the full parser rework; ticket
  07 deletes them and rewires the parser to build `Board`/`Auction`/
  `PlayRecord`. `LinDeal` now stores `[Hand; 4]`.
- Pre-existing clippy warnings: `decimal_bitwise_operands` at
  `src/performance_analysis.rs:834` (test code), plus ~10 warnings in the
  `dds` crate (`cast_possible_*`, `large_stack_arrays`, …).
- `Cargo.lock` is git-tracked yet also listed in `.gitignore`; the ignore
  rule has no effect on tracked files, which is confusing.
- `record::position::Position` is only constructible by deriving from a
  `Board`, yet the design calls it a pure function of `Deal` + `PlayRecord`
  prefix + `Strain`. The DDS/solver-port ticket will likely need a
  parts constructor (`Position::new` or similar) to hand a conditioned deal's
  position to the solver without first assembling a full `Board`.
- `record::position::trick_winner` reimplements the winner rule already in
  `performance_analysis::find_trick_winner_naive`. `record` is deliberately
  `dds`/`lin`-free, so it cannot reuse that function; a future consolidation
  could move the winner rule down into `record` and have the analysis port
  call it.
- `Position::after_play` requires a contract even mid-trick, because
  `Position::trump` is mandatory. The design's wording ("requires a contract
  once play crosses a trick boundary") can be misread as allowing
  contract-free mid-trick derivation; the struct cannot represent an unknown
  strain.
- `Playthrough::new` starts at index 0 (before the opening lead); the design
  never fixed a starting index, so this is an unforced choice a later ticket
  could revisit if callers want the cursor to begin at the current play.
- `dds/src/deal.rs` documents `DdsDeal::current_trick_rank` and
  `DDSCurrTrickRank` as rank *bit-flags*, but DDS reads them as sequential
  ranks 2..14: `init.cpp` does `bit_map_rank[dl.currentTrickRank[k]]` and
  `bit_map_rank` is indexed by absolute rank. `DdsDealBuilder::current_trick`
  already writes sequential ranks (via `DdsRank::new`), so the code is right
  and the docs are wrong; fix the comments when the batch surface is reworked.
- The blanket `impl<S: Solver + Send + Sync> ContextFactory for S` gives every
  concrete solver two `new_context` methods with different return types, so
  `my_solver.new_context()` is ambiguous and callers must write
  `Solver::new_context(&my_solver)` or go through a `Box<dyn ContextFactory>`.
  The shared name is mandated by the design (ADR-0005); a future rename of one
  of the two would remove the footgun.
- The cold capability traits in `src/solver` have no implementor yet: the DDS
  backend should be wired to `DdTableCalculator`/`ParCalculator` with tickets
  10/11 and to `PlayAnalyser` with ticket 13, alongside those traits' first
  consumers.
- `SolverPool` (and its dyn sibling `DynSolverPool`) reuses one context per
  rayon *job*, not per worker: `map_init` calls the initializer for each chunk,
  so a stolen chunk creates an extra context and the design's "one context per
  worker, reused across the run" is only approximated. `with_min_len` keeps the
  count near the worker count and `workers × TT-cap` still bounds
  *instantaneous* memory, but true per-worker reuse would need a
  thread-local/per-worker context table. Revisit in ticket 10 if the
  approximation hurts TT warmth or memory.
- `DynSolverPool` duplicates `SolverPool`'s rayon-pool plumbing (thread count
  clamping, `with_min_len` chunking, `try_map`). A `PoolFactory` trait with an
  associated context type would collapse the two, at the cost of a blanket impl
  for `Box<dyn ContextFactory>`; left split for now to keep the static path
  free of indirection and the type signatures obvious.
- Porting the `dds` table tests to the `dds_c_calc_dd_table` shim (for parallel
  safety) removed the only direct tests of the legacy
  `DdTableCalculator for DoubleDummySolver` impl (`CalcDDtable`/`CalcAllTables`).
  That impl has no caller in the root crate and is kept only with the
  `DoubleDummySolver` surface until ticket 13; its first real consumer will be
  the seam's `DdTableCalculator` wired in ticket 10/11.
- `dds::MAXNOOFBOARDS` (the crate-root re-export) was removed; the raw
  `bindings::ddsffi::MAXNOOFBOARDS` binding is retained but has no Rust
  consumer now that the batch surface is gone.
- Design ticket 04 is ambiguous about the DDS batch entry points: it lists them
  under "Dropped" *and* says they are "kept as a documented future throughput
  option". Implementation ticket 04's explicit deliverable says to delete
  `SolveAllBoardsBin`/`AnalyseAllPlaysBin`/`Boards`/`SolvedBoards`, so ticket 04
  deleted them. A later reader wanting the batch path back should re-bind them
  from the vendored 3.1 sources rather than assume they survived.
- Card-payoff perspective edge (until ticket 10 deletes `CardPayoffSimulation`):
  the engine computes declarer tricks as `13 - side_to_play`, which is only
  correct when the side to play is the *defenders*. For a contract/lead opening
  position that holds (the opening leader is declarer's LHO). For card payoff
  the position is *after* the candidate, so the complement flips the side when
  the candidate is a defender, and when the candidate completes a trick
  `to_play` becomes the trick winner (either side). The pre-refactor engine had
  the same `13 - score` assumption; ticket 10/13 should use
  `if to_play.is_same_line(declarer) { dd_tricks } else { 13 - dd_tricks }`.
- `Position::from_parts` is a `pub(crate)` constructor added so the simulation
  engine can build an opening position from a `Deal` + `Contract` without
  assembling a `Board`. Ticket 09/10 should decide whether to make it public or
  replace it with conditioned-deal construction.
- `simulation::solver::vulnerability_for` picks an arbitrary vulnerability
  consistent with a contract so the card-payoff board construction validates;
  it is only needed because card payoff still derives positions through
  `Board`. It goes away with `CardPayoffSimulation` in ticket 10.
- The card-payoff fan-out now reconstructs the seats of the `play_trace` cards
  (the last was played by `seat.prev()`, and so on) and lets `Board` validate
  them. The deleted `create_current_trick` reversed the trailing trick and set
  DDS's `first` to the seat to play rather than the trick leader, so the old
  mid-trick solves were almost certainly wrong for a non-empty trace.
- `performance_analysis.rs`'s `test_complete_analysis_pipeline` still calls
  `DoubleDummySolver::analyze_play` and only prints (no assertions); ticket 13
  ports it to the seam.
- `dds::utils::dd_score` was deleted with the batch surface. A future
  score-from-deal helper should go through the seam's `SolverContext`, not a
  fresh DDS solver.
- `linparser::LinMetadata` stores rooms as `rooms: Vec<Option<Room>>`
  parallel to `boards`, because `Board` has no room field and LIN's `qx|` is
  per-board (design ticket 02 says "room", singular). If a file-level room is
  really wanted, collapse the vec; ticket 08's `LinFile::as_lin` should decide
  which it consumes.
- The `md|` dealer/hand-order mismatch is now one-sided: the importer reads
  real LIN (`<dealer digit 1=S..4=E>` followed by hands `S,W,N,E`, fourth
  omitted hand = remainder), while `Deal::as_lin` still emits `[N,E,S,W]` with
  a dealer digit derived from the board number. Ticket 08 must fix the writer
  (`Deal::as_lin`/`Board::as_lin`), not the parser.
- `linparser` needs a derivable contract to attribute `pc|` seats (it reuses
  `record::position::trick_winner`, now `pub(crate)`); a board with a play but
  no auction fails at import rather than guessing a leader. The `PlayRecord`
  type itself is contract-free; this is only the LIN importer's constraint.
- `linparser::ParseLinErrorKind` is a categorised taxonomy (including
  `Player`, `Claim`, `Board`, `Metadata`) that ticket 08/12 will likely absorb
  into the crate-wide error model; it is currently only consumed by the
  importer's own tests.
- An empty `Auction` (`Some(Auction::new(vec![]))`) cannot survive a LIN round
  trip: the writer emits no `mb|` lines and the importer reconstructs `None`.
  `Board::new` permits the empty auction, so the model is wider than the
  format. The round-trip property generator avoids it; a future writer could
  reject or document it. (The analogous empty `PlayRecord` was fixed: `Board::new`
  now normalises `Some(empty)` to `None`.)
- `LinFile::write_lin` comma-joins `vg|`/`rs|`/`pn|` fields and the importer
  splits on `,`, so a player name, team or result containing a comma re-parses
  as extra fields. LIN has no escaping, so this is inherently lossy; documented
  on the writer, and the fixture round-trip test only covers comma-free values.
- A `Board` with `contract: Some` but `auction: None`, or a `PlayRecord` whose
  seats do not follow the contract's leader/`trick_winner` rotation, also cannot
  round-trip through LIN: the format derives the contract from the auction and
  attributes seats by rotation. `Board::new` accepts these, so LIN output is
  lossy for them (as the module docs say); a PBN board writer (deferred) could
  carry the contract explicitly.
- `Card::from_str` is order-sensitive and silently defaults the suit to spades:
  `"3C"` parses as the three of *spades* (the `'3'` sets the card under the
  current suit before `'C'` changes it), while `"C3"` is the intended three of
  clubs. LIN's `pc|` cards must be written suit-first; this is a footgun worth
  fixing or documenting in `card.rs`.
- The design ticket's LIN sketch writes `sv|…|rh||ah|Board N|` (vulnerability
  before the board-number tag), but the importer and the real BBO fixtures put
  `rh||ah|Board N|` before `sv|`. Ticket 08's writer follows the parser and the
  fixtures; the sketch is the inaccurate side.
- The PBN board writer remains deferred per design ticket 07: only
  `Deal::as_pbn`/`write_pbn` (the hand record) exist, and `Board` has no
  `as_pbn`/`write_pbn`. A future ticket adding the PBN parser should land the
  board writer beside it so the two agree.
- The quickcheck round-trip property's `AnyBoard` has no `shrink`
  implementation, so a failure reports "No Arguments Provided" instead of the
  offending board. Implementing `shrink` (or returning a `TestResult` that
  embeds the board) would make future failures debuggable.
- `DealerBuilder` keeps two sources of truth for the available deck: `deck`
  (incremental, for eager duplicate rejection in `predeal*`) and the `known`
  set recomputed in `build()`. Cross-seat conflicts surface in `resolve_known`
  while predeal-vs-predeal conflicts surface in `predeal`; unifying them would
  move `predeal`'s error to build time (relied on by a should-panic test).
- The bounded rejection budget default (`2_000_000`) is tuned to the tightest
  shipped test constraint (mean ~10⁵ attempts); rarer constraints — notably
  the untested `utils::dealer_of_lavazza_3nt_opening` — can now return
  `Unsatisfiable` where the old unbounded sampler would eventually succeed.
- `Dealer::predealt()` has no in-crate reader now that candidate validation
  uses `Dealer::current_hand()`; kept only as public API. Ticket 10/12 could
  retire it.
- `CardPayoffSimulation`'s `play_trace` field and trace-to-prefix
  reconstruction still exist despite ticket 09 assuming ticket 04 removed them
  (`create_current_trick` is gone); ticket 10 deletes the type.

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
