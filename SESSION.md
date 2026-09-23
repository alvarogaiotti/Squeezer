# SESSION.md

## Open issues

- `src/bidding_rule_engine.rs` is dead (commented out at `src/lib.rs:114`)
  and would not compile as-is (`serde_yaml` absent). Slated for deletion in
  `.scratch/architecture-followups/issues/03-bbo-async-repair.md`.
- `BBOClient::download()` in both clients still discards the fetched LIN
  body (`let _lin = ...`); persisting it is a TODO.
- `dds/src/analyseplay.rs:286`: `PlayTraceBin::iter` slices `rank` from
  `&self.suit[..number]` (copy-paste), so the iterator yields suit as rank.
- `dds/src/analyseplay.rs:230`: `PlayTracesBin::is_empty` returns
  `self.len() != 0` (inverted).
- `dds` `ParCalculator` has default method bodies but no `impl` for any
  solver, so the parent crate cannot compute par contracts. `CalcAllTables`
  also discards its `AllParResults` out-param.
- `linparser` swallows malformed input instead of erroring: `parse_claim`
  and `parse_vugraph_data` use `unwrap_or(0)`, and
  `parse_room_and_board_number` `unwrap()`s the board-number parse. Relevant
  to ticket 02's goal of failing at import rather than yielding corrupt data.
- `From<LinDeal> for Deal` (`src/deal.rs:1574`) ignores the parsed
  `lin_deal.vulnerability` and recomputes it from the board number, so a
  non-standard vulnerability is silently dropped.
- `dds/src/ddserror.rs` is not rustfmt-clean under the current toolchain:
  `cargo fmt --all` reorders its imports and rewraps several `write!`/
  `assert_ne!` calls. Kept out of the vendored-DDS build commit to avoid
  unrelated churn.
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
