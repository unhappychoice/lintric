use insta::assert_snapshot;
use std::fs;
use std::path::PathBuf;
use lintric_cli::logger::Logger;
use std::sync::{Arc, Mutex};

struct BufLogger {
    out: String,
    err: String,
}

impl BufLogger {
    fn new() -> Self { Self { out: String::new(), err: String::new() } }
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
    let temp_dir = PathBuf::from("tmp");
    fs::create_dir_all(&temp_dir).expect("Unable to create test directory");

    let temp_file_path = temp_dir.join("temp_test_file.rs");
    fs::write(&temp_file_path, "let a = 1;\nlet b = a + 1;\n").expect("Unable to write test file");

    let shared = SharedLogger(Arc::new(Mutex::new(BufLogger::new())));
    lintric_cli::run_from_iter(
        [
            "lintric-cli",
            "--verbose",
            &temp_file_path.to_string_lossy().replace("\\", "/"),
        ],
        &shared,
    );
    let out = shared.0.lock().unwrap().out.clone();
    assert_snapshot!(out);
}

#[test]
fn test_multiple_files_analysis() {
    let temp_dir = PathBuf::from("tmp");
    fs::create_dir_all(&temp_dir).expect("Unable to create test directory");

    let file1_path = temp_dir.join("file1.rs");
    fs::write(&file1_path, "let x = 1;\nlet y = x + 1;\n").expect("Unable to write file1.rs");

    let file2_path = temp_dir.join("file2.ts");
    fs::write(&file2_path, "const a = 1;\nlet b = a + 1;\n").expect("Unable to write file2.ts");

    let shared = SharedLogger(Arc::new(Mutex::new(BufLogger::new())));
    lintric_cli::run_from_iter(
        [
            "lintric-cli",
            &temp_dir.to_string_lossy().replace("\\", "/"),
        ],
        &shared,
    );
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
