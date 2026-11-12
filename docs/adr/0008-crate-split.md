# Split the library into six crates, one vocabulary each

A consumer that needs only `Card` and `Cards` otherwise resolves `nom`, `rayon`,
`reqwest`, `scraper`, `time` and the vendored DDS C++ build. The `dds` feature
isolates the native solver only for a build that controls its own feature set:
Cargo unifies features across a downstream graph, so a crate sharing that graph
with a solver consumer compiles the solver whatever its own manifest says. The
single crate also hides its own structure, with deal production spread across
four places and a flat prelude that makes every module's items look like one
vocabulary.

The repository therefore becomes a virtual Cargo workspace with
`resolver = "3"`: the six library crates below, each owning one vocabulary and
depending only on the crates beneath it, plus the `dds` FFI wrapper and
`squeezer_macros`. The root package is dissolved. There is no umbrella crate: a
`squeezer` facade would re-couple the graph and re-hide the boundaries it exists
to draw.

| Crate | Owns | Depends on |
| --- | --- | --- |
| `squeezer-cards` | `Card`, `Cards`, `Suit`, `InvalidCardError`, the card samplers | `rand` (optional), `serde` (optional) |
| `squeezer-model` | shape and shape parsing, hand constraints, the contract vocabulary (`Seat`, `Vulnerability`, `Strain`, `Doubled`, `Vulnerable`, `Contract`), `Deal`, `Dealer`, `Board`, `Auction`, `Play`, `Position`, the `Deal`/`Board` writers | `squeezer-cards`, `bitvec`, `itertools`, `rand`, `serde` (optional) |
| `squeezer-solver` | the solver trait, `SolverPool`, the DDS adapters, and the DDS backend behind a `dds` feature | `squeezer-cards`, `squeezer-model`, `rayon`, `dds` (optional) |
| `squeezer-io` | LIN and PBN parsing, and the `LinFile`/`PbnFile` container writers, behind `lin` and `pbn` features | `squeezer-model`, `squeezer-cards`, `nom`, `serde` (optional) |
| `squeezer-bbo` | BBO hand retrieval, behind `blocking` and `async` features | `reqwest`, `scraper`, `url`, `time`, `log`, `futures-util`, `tokio` |
| `squeezer-sim` | simulation and performance analysis, with `formatters` gating `colored` | `squeezer-model`, `squeezer-solver`, `squeezer-cards`, `serde` (optional) |

The `Deal`/`Board` writers stay unconditional, as they are today: exporting a
board needs no parser. Only the container writers sit behind `lin` and `pbn`.

`squeezer-solver` names `squeezer-cards` directly as well as `squeezer-model`:
the seam's `CardSolve` carries `Card` and `Cards`, and `squeezer-model` does not
re-export the card crate, so relying on it transitively would hide a real
dependency. Cards sits below the solver in the graph, so this keeps the order.

`squeezer-io` also names `squeezer-cards` directly: the importers build `Cards`
and `Card` values from file input, and the model re-exports none of the card
vocabulary. That is a lower layer, so it adds no cycle. During the migration the
root package kept re-export shims for the historical paths
(`squeezer::linparser`, `squeezer::pbnparser`, the prelude), so it depended on
the extracted crates it re-exported until the final ticket dissolved it.

`squeezer-sim` likewise names `squeezer-cards` directly: card-EV and lead
results carry `Card` and `Cards`, and the model re-exports none of that
vocabulary. It is a lower layer, so the graph stays acyclic.

`squeezer-bbo` depends on no other Squeezer crate, and nothing depends on
`squeezer-sim`, `squeezer-io` or `squeezer-bbo`, so the graph is acyclic by
construction rather than by convention. `squeezer-bbo::download` returns the
fetched LIN documents (`Vec<String>`),
`squeezer-io::linparser::parse_lin_documents` converts such a batch into
`Board`s, and the crate's `download_report`/`download_to_dir` keep per-hand
failures or write the raw documents without adding a Squeezer edge. The
conversion deliberately lives in the importer crate rather than behind a
`squeezer-bbo` feature, so the download crate keeps its edge to no other
Squeezer crate and a consumer that only fetches text never compiles `nom`.

Each crate defines its own error type instead of one crate-wide `SqueezerError`.
The point is that an error names the layer that produced it; one shared enum
would have to name every crate's errors and therefore depend on all of them,
inverting the graph. With no consumer outside this repository, the old enum is
deleted rather than moved.

`Hand` is deleted. It wrapped `Cards`, admitted any number of cards up to 13,
defaulted to empty, and forwarded every operation, so only its string and random
constructors enforced 13 cards. `Deal` becomes `[Cards; 4]` and is the only
owner of the four-seats-of-thirteen invariant. `hcp`, `len_of_suit`, `shape` and
`long_str` become `Cards` methods, and `Hand::from_str` disappears: the 13-card
rule belongs where a `Deal` is built, which is what the parsers already do. The
glossary loses its `Hand` entry.

Every DDS adapter lives in `squeezer-solver`, behind the same `dds` feature that
selects the backend; neither `squeezer-cards` nor `squeezer-model` names the
`dds` crate. Because the `dds` traits and the model types are both foreign to
`squeezer-solver`, the impls are written against solver-local newtypes or free
functions that adapt `Deal`, `Contract`, `Position`, `PlayRecord` and the seat,
suit and strain encodings. `AsDDSCard for Card` is deleted rather than adapted:
the solver converts cards itself. ADR-0004's pure-Rust core thereby becomes a
property of the dependency graph rather than a feature flag whose off state a
consumer's graph can enable by accident. This amends ADR-0004's mechanism, not
its guarantee.

## Considered Options

- **One crate with features** — rejected because feature unification does not
  isolate a downstream consumer: the crate still resolves every dependency, and
  a consumer cannot guarantee its own graph. It is also rejected because the
  module tree is not a set of seams. Modules reach into each other and several
  types sit in a layer where they do not belong, so features would gate code
  without enforcing a direction.
- **One crate per folder** — rejected because the module graph has cycles and
  several types live in the wrong layer. A crate per folder would make those
  cycles illegal and demand the same type-by-type reshuffle, with more manifests
  and no coherent vocabulary per crate.
- **Split only the solver** — rejected because it leaves the rest monolithic: a
  cards consumer still resolves `nom`, `reqwest` and `scraper`, and the module
  boundaries stay unenforced.

## Consequences

- This is a structural change with a real migration cost: files, tests, benches
  and fixtures move one crate at a time, and the workspace must compile and pass
  after every ticket. During the extraction, a bare `cargo test` at the root no
  longer tested the whole workspace — the extracted crates' tests appeared only
  under `--workspace` or `-p <crate>` — so those flags became the norm, and CI
  gained a feature-matrix check per crate. Once the final ticket dissolved the
  root package, bare `cargo test` again covers every member.
- No DDS impl survives on a public card or model type: `AsDDSCard for Card` is
  deleted, and the rest are replaced by solver-local adapters, used where the
  backend needs them. Nothing outside this repository implements or consumes
  them.
- `TryFrom<Strain> for Suit` does not survive as the impl it is today: `Suit`
  moves to `squeezer-cards`, which does not know `Strain`. The conversion
  becomes the inherent `Strain::as_suit() -> Option<Suit>`, which also makes the
  NoTrumps case explicit; `From<Suit> for Strain` stays in `squeezer-model`.
- ADR-0005's seam is unchanged in shape and moves to `squeezer-solver`, whose
  DDS backend stays behind the `dds` feature.
- The earlier research note's rule against splitting into eight crates was about
  solver isolation, which a feature flag already provides, so it does not forbid
  this split. This one is motivated by dependency hygiene and by giving each
  vocabulary one home; the feature flag remains and the crates do not replace
  it. That note now points here.
- `serde` stays optional and is forwarded per crate; no crate enables its own
  default features for another's sake.
