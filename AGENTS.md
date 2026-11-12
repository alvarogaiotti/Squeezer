# AGENTS.md

## Agent skills

### Issue tracker

Work is tracked as local markdown files under `.scratch/`. See `docs/agents/issue-tracker.md`.

### Triage labels

The five canonical triage roles use default label strings. See `docs/agents/triage-labels.md`.

### Domain docs

Single-context: one `CONTEXT.md` with `docs/adr/` at the repo root. See `docs/agents/domain.md`.

## Structure

- Root package `squeezer` is the library and the workspace root; `dds/` wraps Bo Haglund's DDS C++ solver; `squeezer_macros/` holds the proc-macro derives (`SuitIter`, `RawDDSRef`, `IntoRawDDS`, `RawDDSRefMut`); extracted crates live under `crates/` (currently `squeezer-bbo`, `squeezer-cards`, `squeezer-io` and `squeezer-model`).
- It is a Cargo workspace (resolver 3) with `crates/squeezer-bbo`, `crates/squeezer-cards`, `crates/squeezer-io`, `crates/squeezer-model`, `dds` and `squeezer_macros` as members; the root package is an implicit member. `cargo test` at the root tests only the library, so use `cargo test --workspace` for every member, or `-p <member>` for one.

## Native DDS library (vendored, built from source)

- `vendor/dds` is a git submodule pinned to the DDS `v3.1.0` tag (`fe6a6581a13535779eda5d9141aa704b3f6c8965`). A clean checkout must run `git submodule update --init vendor/dds` before building the default features (`dds`).
- The `dds` crate's `build.rs` compiles the 39 translation units under `vendor/dds/library/src/**/*.cpp` (`g++ -std=c++20 -O3 -fPIC -w`, `CXX` overridable), archives them with `ar crs`, and emits the link directives (`static=dds3`, plus `stdc++`, `pthread`, `m`). It owns `links = "dds"`.
- No prebuilt `libdds.so`/`libdds.a` is required or used, and there is no root `build.rs`. `cargo check --no-default-features` skips the C++ build entirely.

## Commands

- Default features are `lin`, `pbn`, `dds`: `cargo check` / `cargo test`
  (the root package only; add `--workspace` to include the members).
- Solver concurrency is owned by our own pool: each rayon *job* (chunk of work)
  gets its own DDS context (`SolverPool`/`DynSolverPool`), with at most one
  context live per worker at a time, so `workers × TT-cap` bounds instantaneous
  memory and plain `cargo test` (no thread cap) is safe. `AnalysePlayBin`
  builds its own local context per call, so the legacy play-analysis path is
  concurrent-safe too.
- The `dds` crate's own tests are parallel-safe under 3.1: the context shim
  takes a caller-owned context, and the legacy `SolveBoard`/`CalcDDtable`/
  `CalcAllTables` entry points draw their context from a `thread_local`. Run
  `cargo test -p dds` with the default harness.
- `cargo fmt --all`. CI (`.github/workflows/rust.yml`) checks out the `vendor/dds`
  submodule, runs `cargo test --workspace --verbose`, then `cargo fmt --all --check`.
- `cargo bench --bench <name>` (criterion, `harness = false`; see `Cargo.toml`).

## Features

- `simulation` requires `dds`; `performance_analysis` requires both `dds` and `lin` (it consumes LIN plays). Verify combos with `cargo check --no-default-features --features dds,lin`.
- `formatters` is opt-in (not in `default`): it enables `dep:colored` and, because the renderers live in the `dds`-gated simulation module, `dds`.
- Card primitives live in `squeezer-cards` (`crates/squeezer-cards`) with `rand`
  (the samplers) and `serde` optional and no workspace dependency;
  `squeezer-model` enables `rand` because its dealer draws from the samplers.
- Shape, shape parsing, hand constraints, the contract vocabulary, the
  evaluator, the dealer and the whole record layer (`Deal`, `Board`, `Auction`,
  `Play`, `Position`) live in `squeezer-model` (`crates/squeezer-model`), which
  depends on `squeezer-cards`, `rand`, `bitvec` and `itertools` with `serde`
  optional and names no solver, parser or network crate. The `Deal` and `Board`
  writers stay here, unconditional: exporting a board needs no parser.
- The LIN and PBN importers and the `LinFile`/`PbnFile` container writers live
  in `squeezer-io` (`crates/squeezer-io`), behind its `lin` and `pbn` features
  (both on by default). `nom` is optional and compiled only by those features,
  so `default-features = false` builds the crate without it. It depends on
  `squeezer-model` plus `squeezer-cards` (the parsers build `Cards`/`Card`) and
  names no solver, HTTP crate or the root. The root's `lin`/`pbn` features
  forward to it so `squeezer::linparser` and `squeezer::pbnparser` keep working.
- BBO downloads live in the `squeezer-bbo` member (`crates/squeezer-bbo`): its
  `blocking` feature uses `reqwest::blocking`, `async` (its default) uses async
  `reqwest`, and both share `scraper`, `url` and `time` for parsing hand-record
  links. Its live-network test has hardcoded credentials and is `#[ignore]`d —
  never run it.
- Clippy `pedantic` is enabled crate-wide with a broad allow list in each crate's `src/lib.rs`; don't chase the allowed lints.

## Gotchas

- Root `config.toml` is not read by Cargo (that needs `.cargo/config.toml`), so `target-cpu=native` is not applied; set `RUSTFLAGS` explicitly when needed.
- `tests/uniformity_check.rs` is not a `#[test]` function, so it never runs.
