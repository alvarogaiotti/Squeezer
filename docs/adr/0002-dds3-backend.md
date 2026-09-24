# DDS 3.1 is the sole solver backend, bound through its C ABI

We bind DDS 3.1 using the upstream pure-C `dds_c_*` context shim for solving,
DD tables, and par, plus the legacy `AnalysePlay*` entry points for play
analysis. We do not fork DDS or maintain a custom C++ shim. The source is
vendored as a submodule pinned to tagged releases (`v3.1.0` first) and compiled
in `build.rs`, because an unmodified checkout must build and 3.0 carries a
large performance regression.

## Considered Options

- **Staying on the 2.9 legacy C API**: rejects warm transposition-table reuse
  and real parallelism, both central to the card-EV work.
- **A custom C++ shim over the modern C++ API**: its only advantage would be
  context-aware play analysis, which requires patching DDS (a fork) and which
  we do not need, since `AnalysePlay` already reuses one context across a hand.

## Consequences

The binding spans two ABIs on purpose: the `dds_c_*` context shim is bound for
context-aware work (solve, DD table, par), and the legacy flat C API for the
capabilities the shim lacks at the pinned tag (`AnalysePlay*`,
`Par`/`DealerPar`, `ErrorMessage`). A reader who sees `dds_c_*` and
`AnalysePlayBin` side by side should not "unify" them — the split is forced by
what upstream exports, not an oversight.
