use insta::assert_snapshot;
use lintric_cli::logger::Logger;
use lintric_cli::Outcome;
use std::sync::{Arc, Mutex};

struct BufLogger {
    out: String,
    err: String,
}

impl BufLogger {
    fn new() -> Self {
        Self {
            out: String::new(),
            err: String::new(),
        }
    }
}

struct SharedLogger(Arc<Mutex<BufLogger>>);

impl Logger for SharedLogger {
    fn info(&self, message: &str) {
        let mut g = self.0.lock().unwrap();
        g.out.push_str(message);
        g.out.push('\n');
    }
    fn warn(&self, message: &str) {
        let mut g = self.0.lock().unwrap();
        g.err.push_str(message);
        g.err.push('\n');
    }
    fn error(&self, message: &str) {
        let mut g = self.0.lock().unwrap();
        g.err.push_str(message);
        g.err.push('\n');
    }
}

#[test]
fn test_basic_analysis() {
    let shared = SharedLogger(Arc::new(Mutex::new(BufLogger::new())));
    lintric_cli::run_from_iter(
        [
            "lintric-cli",
            "--verbose",
            "tests/fixtures/temp_test_file.rs",
        ],
        &shared,
    );
    let out = shared.0.lock().unwrap().out.clone();
    assert_snapshot!(out);
}

#[test]
fn test_multiple_files_analysis() {
    let shared = SharedLogger(Arc::new(Mutex::new(BufLogger::new())));
    lintric_cli::run_from_iter(["lintric-cli", "tests/fixtures/multiple"], &shared);
    let out = shared.0.lock().unwrap().out.clone();
    assert_snapshot!(out);
}

#[test]
fn test_json_output() {
    let fixture_path = "tests/fixtures/complex_rust_code.rs";

    let shared = SharedLogger(Arc::new(Mutex::new(BufLogger::new())));
    lintric_cli::run_from_iter(["lintric-cli", "--json", fixture_path], &shared);
    let out = shared.0.lock().unwrap().out.clone();
    assert_snapshot!(out);
}

#[test]
fn test_complex_rust_analysis() {
    let fixture_path = "tests/fixtures/complex_rust_code.rs";

    let shared = SharedLogger(Arc::new(Mutex::new(BufLogger::new())));
    lintric_cli::run_from_iter(["lintric-cli", "--verbose", fixture_path], &shared);
    let out = shared.0.lock().unwrap().out.clone();
    assert_snapshot!(out);
}

#[test]
fn test_complex_typescript_analysis() {
    let fixture_path = "tests/fixtures/complex_typescript_code.ts";

    let shared = SharedLogger(Arc::new(Mutex::new(BufLogger::new())));
    lintric_cli::run_from_iter(["lintric-cli", "--verbose", fixture_path], &shared);
    let out = shared.0.lock().unwrap().out.clone();
    assert_snapshot!(out);
}

#[test]
fn test_debug_ast_rust() {
    let fixture_path = "tests/fixtures/complex_rust_code.rs";

    let shared = SharedLogger(Arc::new(Mutex::new(BufLogger::new())));
    lintric_cli::run_from_iter(["lintric-cli", "debug", "ast", fixture_path], &shared);
    let out = shared.0.lock().unwrap().out.clone();
    assert_snapshot!(out);
}

#[test]
fn test_debug_ast_typescript() {
    let fixture_path = "tests/fixtures/complex_typescript_code.ts";

    let shared = SharedLogger(Arc::new(Mutex::new(BufLogger::new())));
    lintric_cli::run_from_iter(["lintric-cli", "debug", "ast", fixture_path], &shared);
    let out = shared.0.lock().unwrap().out.clone();
    assert_snapshot!(out);
}

#[test]
fn test_debug_ir_rust() {
    let fixture_path = "tests/fixtures/complex_rust_code.rs";

    let shared = SharedLogger(Arc::new(Mutex::new(BufLogger::new())));
    lintric_cli::run_from_iter(["lintric-cli", "debug", "ir", fixture_path], &shared);
    let out = shared.0.lock().unwrap().out.clone();
    assert_snapshot!(out);
}

#[test]
fn test_debug_ir_typescript() {
    let fixture_path = "tests/fixtures/complex_typescript_code.ts";

    let shared = SharedLogger(Arc::new(Mutex::new(BufLogger::new())));
    lintric_cli::run_from_iter(["lintric-cli", "debug", "ir", fixture_path], &shared);
    let out = shared.0.lock().unwrap().out.clone();
    assert_snapshot!(out);
}

#[test]
fn test_successful_analysis_reports_success() {
    let shared = SharedLogger(Arc::new(Mutex::new(BufLogger::new())));
    let outcome =
        lintric_cli::run_from_iter(["lintric", "tests/fixtures/complex_rust_code.rs"], &shared);
    assert_eq!(outcome, Outcome::Success);
}

#[test]
fn test_missing_path_reports_failure() {
    let shared = SharedLogger(Arc::new(Mutex::new(BufLogger::new())));
    let outcome =
        lintric_cli::run_from_iter(["lintric", "tests/fixtures/no_such_file.rs"], &shared);

    assert_eq!(outcome, Outcome::Failure);
    let err = shared.0.lock().unwrap().err.clone();
    assert!(err.contains("is neither a file nor a directory"), "{err}");
}

#[test]
fn test_unsupported_file_type_is_only_a_warning() {
    let shared = SharedLogger(Arc::new(Mutex::new(BufLogger::new())));
    let outcome = lintric_cli::run_from_iter(["lintric", "Cargo.toml"], &shared);

    assert_eq!(outcome, Outcome::Success);
    let err = shared.0.lock().unwrap().err.clone();
    assert!(err.contains("Skipping unsupported file type"), "{err}");
}

#[test]
fn test_debug_ast_missing_file_reports_failure() {
    let shared = SharedLogger(Arc::new(Mutex::new(BufLogger::new())));
    let outcome = lintric_cli::run_from_iter(
        ["lintric", "debug", "ast", "tests/fixtures/no_such_file.rs"],
        &shared,
    );

    assert_eq!(outcome, Outcome::Failure);
    let err = shared.0.lock().unwrap().err.clone();
    assert!(err.starts_with("Error:"), "{err}");
}
