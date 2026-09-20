# SESSION.md

## Open issues

- `src/bidding_rule_engine.rs` is dead (commented out at `src/lib.rs:114`)
  and would not compile as-is (`serde_yaml` absent). Slated for deletion in
  `.scratch/architecture-followups/issues/03-bbo-async-repair.md`.
- `BBOClient::download()` in both clients still discards the fetched LIN
  body (`let _lin = ...`); persisting it is a TODO.

## Resolved

- Benches/tests vs feature gates: `[[bench]]`/`[[test]]` targets that need
  the solver or LIN parser now carry `required-features`, so
  `cargo clippy --all-targets` passes on every feature set
  (`bbo`-only, `bbo_async`-only, `dds,lin`, no-default).
