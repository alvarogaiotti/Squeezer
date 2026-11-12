# AGENTS.md

## Agent skills

### Issue tracker

Work is tracked as local markdown files under `.scratch/`. See `docs/agents/issue-tracker.md`.

### Triage labels

The five canonical triage roles use default label strings. See `docs/agents/triage-labels.md`.

### Domain docs

Single-context: one `CONTEXT.md` with `docs/adr/` at the repo root. See `docs/agents/domain.md`.

## Structure

- The repository is a virtual Cargo workspace (resolver 3) with no root package and no umbrella crate. The six library crates live under `crates/`: `squeezer-cards`, `squeezer-model`, `squeezer-solver`, `squeezer-io`, `squeezer-bbo` and `squeezer-sim`. `dds/` wraps Bo Haglund's DDS C++ solver and `squeezer_macros/` holds the proc-macro derives (`SuitIter`, `RawDDSRef`, `IntoRawDDS`, `RawDDSRefMut`).
- The members are `crates/squeezer-bbo`, `crates/squeezer-cards`, `crates/squeezer-io`, `crates/squeezer-model`, `crates/squeezer-sim`, `crates/squeezer-solver`, `dds` and `squeezer_macros`. `dds` stays at the top level because its build script resolves the vendored sources as `../vendor/dds`.
- `cargo test` at the root runs every workspace member's default targets, and `cargo test --workspace` spells that out. Use `-p <member>` for one crate. There is no `squeezer` package to ask for.

## Native DDS library (vendored, built from source)

- `vendor/dds` is a git submodule pinned to the DDS `v3.1.0` tag (`fe6a6581a13535779eda5d9141aa704b3f6c8965`). A clean checkout must run `git submodule update --init vendor/dds` before building the default features (`dds`).
- The `dds` crate's `build.rs` compiles the 39 translation units under `vendor/dds/library/src/**/*.cpp` (`g++ -std=c++20 -O3 -fPIC -w`, `CXX` overridable), archives them with `ar crs`, and emits the link directives (`static=dds3`, plus `stdc++`, `pthread`, `m`). It owns `links = "dds"`.
- No prebuilt `libdds.so`/`libdds.a` is required or used, and there is no `build.rs` outside the `dds` member. `cargo check -p squeezer-cards --no-default-features` (and the same for the other pure crates) skips the C++ build entirely.

## Commands

- `cargo test --workspace` runs every crate's tests with its default features; add `--all-features` for the optional paths. A bare `cargo test` already covers every member because there is no root package, but `--workspace` makes that explicit. `cargo build --workspace --all-targets` compiles the tests, benchmarks and examples.
- Each pure crate must also check without defaults: `cargo check -p squeezer-cards --no-default-features`, and the same for `squeezer-model`, `squeezer-solver` and `squeezer-sim`; `squeezer-io` takes `--no-default-features` too (its parsers are the optional part).
- Solver concurrency is owned by our own pool: `SolverPool` slices a call into
  at most one chunk per worker, each chunk creates one DDS context and reuses it
  for every index in that chunk, and contexts are dropped with their chunk. At
  most one context is live per worker at any instant, and a call creates no more
  contexts than it has chunks (at most `workers`), so `workers × TT-cap` bounds
  instantaneous memory and plain `cargo test` (no thread cap) is safe.
  `AnalysePlayBin`
  builds its own local context per call, and so does the DD-table calculation,
  so both are concurrent-safe; a worker that invokes either transiently holds a
  second context besides its own.
- The `dds` crate's own tests are parallel-safe under 3.1: the context shim
  takes a caller-owned context, and the legacy `SolveBoard`/`CalcDDtable`/
  `CalcAllTables` entry points draw their context from a `thread_local`. Run
  `cargo test -p dds` with the default harness.
- `cargo fmt --all`. CI (`.github/workflows/rust.yml`) checks out the `vendor/dds`
  submodule, runs `cargo test --workspace --verbose`, the feature matrix, and
  `cargo fmt --all --check`.
- `cargo bench -p <crate> --bench <name>` (criterion, `harness = false`; see the
  owning crate's manifest). The dealer benchmark lives in `squeezer-model`, the
  card-sampling one in `squeezer-cards` (behind its `rand` feature).

## Features

- The Monte Carlo simulations (contract payoff, card EV and opening lead), the
  result and statistics types, performance analysis, and the terminal
  renderers live in `squeezer-sim` (`crates/squeezer-sim`). It depends on
  `squeezer-cards`, `squeezer-model`, `squeezer-solver` (default features off)
  and `rayon`, with `colored` optional behind `formatters` and `serde` optional.
  Its `dds` feature (on by default) forwards to `squeezer-solver/dds`: with it
  off the builder's `DdsSolver` fallback is compiled out and an explicit
  `ContextFactory` is required, which is how the mock-solver tests run without
  linking DDS.
- The performance-analysis pipeline test lives in `squeezer-io/tests` with its
  LIN fixture and takes `squeezer-sim` and `squeezer-solver` as dev-dependencies,
  so the sim crate's library and tests name no parser.
- Card primitives live in `squeezer-cards` (`crates/squeezer-cards`) with `rand`
  (the samplers) and `serde` optional and no workspace dependency;
  `squeezer-model` enables `rand` because its dealer draws from the samplers.
- Shape, shape parsing, hand constraints, the contract vocabulary, the dealer
  and the whole record layer (`Deal`, `Board`, `Auction`, `Play`, `Position`
  and their writers) live in `squeezer-model`
  (`crates/squeezer-model`), which depends on `squeezer-cards`, `rand`, `bitvec`
  and `itertools` with `serde` optional and names no solver, parser or network
  crate. The `Deal` and `Board` writers stay here, unconditional: exporting a
  board needs no parser.
- The solver seam (`Solver`, `SolverContext`, `ContextFactory`, the
  `DdTableCalculator`/`ParCalculator`/`PlayAnalyser` traits, `SolverPool`,
  `SolverError`) and every DDS adapter live in
  `squeezer-solver` (`crates/squeezer-solver`), which depends on
  `squeezer-cards`, `squeezer-model`, `rayon` and optional `dds`. Its `dds`
  feature is on by default (the backend is the crate's job); `squeezer-sim`
  depends on it with default features off and forwards its own `dds` feature.
  Its example-hand fixtures sit behind the `test-support` feature, which
  `squeezer-sim` enables from its dev-dependencies.
- The LIN and PBN importers and the `LinFile`/`PbnFile` container writers live
  in `squeezer-io` (`crates/squeezer-io`), behind its `lin` and `pbn` features
  (both on by default). `nom` is optional and compiled only by those features,
  so `default-features = false` builds the crate without it. It depends on
  `squeezer-model` plus `squeezer-cards` (the parsers build `Cards`/`Card`) and
  names no solver, HTTP crate or other Squeezer crate above it.
- BBO downloads live in the `squeezer-bbo` member (`crates/squeezer-bbo`): its
  `blocking` feature uses `reqwest::blocking`, `async` (its default) uses async
  `reqwest`, and both share `scraper`, `url` and `time` for parsing hand-record
  links. `download` returns the LIN document of every fetched hand as
  `Vec<String>`; turning that batch into `Board`s is `squeezer-io`'s
  `linparser::parse_lin_documents`, so `squeezer-bbo` keeps its ADR-0008 edge
  to no other Squeezer crate. Its live-network test reads `BBO_USERNAME` and
  `BBO_PASSWORD` from the environment and is `#[ignore]`d — never run it.
- Clippy `pedantic` is enabled crate-wide with a broad allow list in each crate's `src/lib.rs`; don't chase the allowed lints.

## Gotchas

- Root `config.toml` is not read by Cargo (that needs `.cargo/config.toml`), so `target-cpu=native` is not applied; set `RUSTFLAGS` explicitly when needed.
- `crates/squeezer-cards/tests/sampler_choice.rs` and
  `tests/uniformity_check.rs` are gated on the crate's `rand` feature through
  `required-features`, and the default feature set leaves `rand` off. CI runs
  them explicitly with `cargo test -p squeezer-cards --features rand`, and the
  `--all-features` run covers them too. Locally, `cargo test -p squeezer-cards
  --no-default-features` never compiles them.
