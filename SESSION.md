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
- `From<LinDeal> for Deal` (`src/deal.rs:1574`) ignores the parsed
  `lin_deal.vulnerability` and recomputes it from the board number, so a
  non-standard vulnerability is silently dropped.
- Pre-existing clippy warnings: `decimal_bitwise_operands` at
  `src/performance_analysis.rs:834` (test code), plus ~10 warnings in the
  `dds` crate (`cast_possible_*`, `large_stack_arrays`, …).
- `Cargo.lock` is git-tracked yet also listed in `.gitignore`; the ignore
  rule has no effect on tracked files, which is confusing.

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
