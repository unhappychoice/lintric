//! Measures how accurately Lintric detects line-level dependencies.
//!
//! Fixtures under `fixtures/` carry hand-written expectations as `//~ depends:` annotations.
//! The expectations are written from the language's semantics, independent of what the
//! analyzer currently produces, so this crate can report precision and recall rather than
//! merely detecting change.

pub mod analysis;
pub mod baseline;
pub mod comparison;
pub mod edge;
pub mod expectation;
pub mod fixtures;
pub mod report;
pub mod shift;

use std::path::{Path, PathBuf};

/// Directory holding the annotated fixtures.
pub fn fixtures_dir() -> PathBuf {
    manifest_dir().join("fixtures")
}

/// File holding the recorded accuracy numbers.
pub fn baseline_path() -> PathBuf {
    manifest_dir().join("baseline.json")
}

fn manifest_dir() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
}
