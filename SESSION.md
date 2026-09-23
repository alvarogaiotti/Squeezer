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
- After removing the PBN entry points, `DdsDealPbn`, `BoardsPbn`,
  `DdTableDealPbn`, `DdTableDealsPbn`, `PlayTracePBN` and `PlayTracesPBN`
  have no callers. Ticket 04 should delete them with the batch rework.
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
- `SolverPool` reuses one context per rayon *job*, not per worker: `map_init`
  calls the initializer for each chunk, so a stolen chunk creates an extra
  context and the design's "one context per worker, reused across the run" is
  only approximated. `with_min_len` keeps the count near the worker count and
  `workers × TT-cap` still bounds *instantaneous* memory, but true per-worker
  reuse would need a thread-local/per-worker context table. Revisit in ticket
  04/10 if the approximation hurts TT warmth or memory.

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
