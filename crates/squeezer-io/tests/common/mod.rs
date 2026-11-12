// Copyright (C) 2024 Alvaro Gaiotti
// See end of file for license information

//! Shared helpers for the importer integration tests.
//!
//! `tests/` targets do not share code by default; this module is included by
//! each one that reads fixtures.

/// Reads one tracked fixture from `tests/fixtures`.
///
/// `path` is the bare file name: the fixture directory moves with the crate,
/// so a manifest-relative join keeps the tests independent of the caller's
/// working directory.
pub fn read_fixture(path: &str) -> String {
    let full = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join(path);
    std::fs::read_to_string(&full)
        .unwrap_or_else(|error| panic!("{} should be readable: {error}", full.display()))
}
