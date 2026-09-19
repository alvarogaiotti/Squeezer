# BBO async repair and sync/async trait seam

Status: needs-triage

## Problem

`AsyncBBOClient` is the default-feature BBO surface yet is non-functional:

- `is_logged()` returns `Ok(false)` in both branches
  (`src/bbo_async/mod.rs:128-133`).
- `get_links("", &mut vec)` runs the link regex against an empty string
  (`:197-202`), so `download()` (`:234-255`) is a no-op.
- `login` (`:137`) and `get_in_interval` (`:178`) are private.
- The shared `BBOClient` trait (`src/bbohelpers.rs:30-47`) is sync-only and
  implemented solely by `BlockingBBOClient`; the async client duplicates
  the link regex, interval slicing, and error-display impls.
- Only test hits the live site with hardcoded credentials
  (`src/bbo/mod.rs:263-281`); async has zero tests.
- `bidding_rule_engine.rs` is dead (commented out in `lib.rs:113-114`)
  and would not compile as-is (uses `serde_yaml`, absent from `Cargo.toml`).

## Direction

Slice 1 (mechanical, agent-ready): fix `is_logged`, pass fetched text to
`get_links`, make the client usable; add mock-based unit tests.
Slice 2 (needs decision): one shared client core vs sync/async split —
async trait, duplicated logic moved to `bbohelpers`, live-net test
strategy, and whether `bidding_rule_engine` revives (add `serde_yaml`) or
is deleted.

## Comments

- Slice 1 done 2026-09-19: `is_logged` else-branch returns `true`
  (decision extracted as tested `logged_in_from_body`); `get_in_interval`
  passes fetched page text to `get_links`; `login`/`get_in_interval` now
  `pub` like the sync surface. Offline tests: status decision, link
  extraction (multi-link + empty), client construction. Network paths
  remain untestable (hardcoded URLs, no mock crates) — recorded for
  slice 2, which still needs the shared-core decision.
