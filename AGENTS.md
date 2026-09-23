# AGENTS.md

## Agent skills

### Issue tracker

Work is tracked as local markdown files under `.scratch/`. See `docs/agents/issue-tracker.md`.

### Triage labels

The five canonical triage roles use default label strings. See `docs/agents/triage-labels.md`.

### Domain docs

Single-context: one `CONTEXT.md` with `docs/adr/` at the repo root. See `docs/agents/domain.md`.

## Structure

- Root package `squeezer` is the library; `dds/` wraps Bo Haglund's DDS C++ solver; `squeezer_macros/` holds the proc-macro derives (`SuitIter`, `RawDDSRef`, `IntoRawDDS`, `RawDDSRefMut`).
- Not a Cargo workspace: `dds` and `squeezer_macros` are path dependencies. Target them with `--manifest-path`, e.g. `cargo test --manifest-path dds/Cargo.toml`.

## Native DDS library (vendored, built from source)

- `vendor/dds` is a git submodule pinned to the DDS `v3.1.0` tag (`fe6a6581a13535779eda5d9141aa704b3f6c8965`). A clean checkout must run `git submodule update --init vendor/dds` before building the default features (`dds`).
- The `dds` crate's `build.rs` compiles the 39 translation units under `vendor/dds/library/src/**/*.cpp` (`g++ -std=c++20 -O3 -fPIC -w`, `CXX` overridable), archives them with `ar crs`, and emits the link directives (`static=dds3`, plus `stdc++`, `pthread`, `m`). It owns `links = "dds"`.
- No prebuilt `libdds.so`/`libdds.a` is required or used, and there is no root `build.rs`. `cargo check --no-default-features` skips the C++ build entirely.

## Commands

- Default features are `lin`, `dds`, `bbo_async`: `cargo check` / `cargo test`.
- DDS keeps global state that segfaults under parallel test threads. Run `cargo test -- --test-threads=1`; simulation tests additionally serialize through a `dds_test_lock`.
- `cargo fmt --all`. CI (`.github/workflows/rust.yml`) runs `cargo test --verbose --features dds` then `cargo fmt --all`.
- `cargo bench --bench <name>` (criterion, `harness = false`; see `Cargo.toml`).

## Features

- `simulation` requires `dds`; `performance_analysis` requires both `dds` and `lin` (it consumes LIN plays). Verify combos with `cargo check --no-default-features --features dds,lin`.
- `bbo` = blocking (`reqwest::blocking`); `bbo_async` = async (`reqwest`, default). Both share `scraper` + `url` for parsing hand-record links. Enabling `bbo` compiles a live-network test with hardcoded credentials in `src/bbo/mod.rs` — do not run it.
- Clippy `pedantic` is enabled crate-wide with a broad allow list in `src/lib.rs`; don't chase the allowed lints.

## Gotchas

- Root `config.toml` is not read by Cargo (that needs `.cargo/config.toml`), so `target-cpu=native` is not applied; set `RUSTFLAGS` explicitly when needed.
- `tests/uniformity_check.rs` and the helper in `tests/performance_analysis_test.rs` are not `#[test]` functions, so they never run.
