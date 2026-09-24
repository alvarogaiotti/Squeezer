# Pure-Rust core with the solver behind a feature

The core domain — cards, hands, shapes, constraints, deals, ranges, and the LIN
parser — must compile with `--no-default-features` and no native dependency, so
a clean checkout builds and downstream users can depend on the domain model
without the solver. Double-dummy solving sits behind the `dds` feature; `serde`
support is optional but complete across the core types. This keeps the solver
abstraction honest and is the precondition for any future WASM or bindings work.
