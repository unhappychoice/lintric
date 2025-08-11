use insta::assert_snapshot;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

#[test]
fn test_basic_analysis() {
    let temp_dir = PathBuf::from("tmp");
    fs::create_dir_all(&temp_dir).expect("Unable to create test directory");

    let temp_file_path = temp_dir.join("temp_test_file.rs");
    fs::write(&temp_file_path, "let a = 1;\nlet b = a + 1;\n").expect("Unable to write test file");

    let output = Command::new("cargo")
        .arg("run")
        .arg("--package")
        .arg("lintric-cli")
        .arg("--")
        .arg("--verbose")
        .arg(temp_file_path.to_string_lossy().replace("\\", "/"))
        .output()
        .expect("Failed to execute command");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let _stderr = String::from_utf8_lossy(&output.stderr);

    assert!(output.status.success());
    assert_snapshot!(stdout);
}

#[test]
fn test_multiple_files_analysis() {
    let temp_dir = PathBuf::from("tmp");
    fs::create_dir_all(&temp_dir).expect("Unable to create test directory");

    let file1_path = temp_dir.join("file1.rs");
    fs::write(&file1_path, "let x = 1;\nlet y = x + 1;\n").expect("Unable to write file1.rs");

    let file2_path = temp_dir.join("file2.ts");
    fs::write(&file2_path, "const a = 1;\nlet b = a + 1;\n").expect("Unable to write file2.ts");

    let output = Command::new("cargo")
        .arg("run")
        .arg("--package")
        .arg("lintric-cli")
        .arg("--")
        .arg(temp_dir.to_string_lossy().replace("\\", "/"))
        .output()
        .expect("Failed to execute command");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let _stderr = String::from_utf8_lossy(&output.stderr);

    assert!(output.status.success());
    assert_snapshot!(stdout);
}

#[test]
fn test_json_output() {
    let fixture_path = "../core/tests/rust/fixtures/complex_rust_code.rs";

    let output = Command::new("cargo")
        .arg("run")
        .arg("--package")
        .arg("lintric-cli")
        .arg("--")
        .arg("--json")
        .arg(fixture_path)
        .output()
        .expect("Failed to execute command");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let _stderr = String::from_utf8_lossy(&output.stderr);

    assert!(output.status.success());
    assert_snapshot!(stdout);
}

#[test]
fn test_complex_rust_analysis() {
    let fixture_path = "../core/tests/rust/fixtures/complex_rust_code.rs";

    let output = Command::new("cargo")
        .arg("run")
        .arg("--package")
        .arg("lintric-cli")
        .arg("--")
        .arg("--verbose")
        .arg(fixture_path)
        .output()
        .expect("Failed to execute command");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let _stderr = String::from_utf8_lossy(&output.stderr);

    assert!(output.status.success());
    assert_snapshot!(stdout);
}

#[test]
fn test_complex_typescript_analysis() {
    let fixture_path = "../core/tests/typescript/fixtures/complex_typescript_code.ts";

    let output = Command::new("cargo")
        .arg("run")
        .arg("--package")
        .arg("lintric-cli")
        .arg("--")
        .arg("--verbose")
        .arg(fixture_path)
        .output()
        .expect("Failed to execute command");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let _stderr = String::from_utf8_lossy(&output.stderr);

    assert!(output.status.success());
    assert_snapshot!(stdout);
}

#[test]
fn test_debug_ast_rust() {
    let fixture_path = "../core/tests/rust/fixtures/complex_rust_code.rs";

    let output = Command::new("cargo")
        .arg("run")
        .arg("--package")
        .arg("lintric-cli")
        .arg("--")
        .arg("debug")
        .arg("ast")
        .arg(fixture_path)
        .output()
        .expect("Failed to execute command");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let _stderr = String::from_utf8_lossy(&output.stderr);

    assert!(output.status.success());
    assert_snapshot!(stdout);
}

#[test]
fn test_debug_ast_typescript() {
    let fixture_path = "../core/tests/typescript/fixtures/complex_typescript_code.rs";

    let output = Command::new("cargo")
        .arg("run")
        .arg("--package")
        .arg("lintric-cli")
        .arg("--")
        .arg("debug")
        .arg("ast")
        .arg(fixture_path)
        .output()
        .expect("Failed to execute command");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let _stderr = String::from_utf8_lossy(&output.stderr);

    assert!(output.status.success());
    assert_snapshot!(stdout);
}

#[test]
fn test_debug_definition_rust() {
    let fixture_path = "../core/tests/rust/fixtures/complex_rust_code.rs";

    let output = Command::new("cargo")
        .arg("run")
        .arg("--package")
        .arg("lintric-cli")
        .arg("--")
        .arg("debug")
        .arg("definition")
        .arg(fixture_path)
        .output()
        .expect("Failed to execute command");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let _stderr = String::from_utf8_lossy(&output.stderr);

    assert!(output.status.success());
    assert_snapshot!(stdout);
}

#[test]
fn test_debug_definition_typescript() {
    let fixture_path = "../core/tests/typescript/fixtures/complex_typescript_code.rs";

    let output = Command::new("cargo")
        .arg("run")
        .arg("--package")
        .arg("lintric-cli")
        .arg("--")
        .arg("debug")
        .arg("definition")
        .arg(fixture_path)
        .output()
        .expect("Failed to execute command");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let _stderr = String::from_utf8_lossy(&output.stderr);

    assert!(output.status.success());
    assert_snapshot!(stdout);
}

#[test]
fn test_debug_dependency_rust() {
    let fixture_path = "../core/tests/rust/fixtures/complex_rust_code.rs";

    let output = Command::new("cargo")
        .arg("run")
        .arg("--package")
        .arg("lintric-cli")
        .arg("--")
        .arg("debug")
        .arg("dependency")
        .arg(fixture_path)
        .output()
        .expect("Failed to execute command");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let _stderr = String::from_utf8_lossy(&output.stderr);

    assert!(output.status.success());
    assert_snapshot!(stdout);
}

#[test]
fn test_debug_dependency_typescript() {
    let fixture_path = "../core/tests/typescript/fixtures/complex_typescript_code.rs";

    let output = Command::new("cargo")
        .arg("run")
        .arg("--package")
        .arg("lintric-cli")
        .arg("--")
        .arg("debug")
        .arg("dependency")
        .arg(fixture_path)
        .output()
        .expect("Failed to execute command");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let _stderr = String::from_utf8_lossy(&output.stderr);

    assert!(output.status.success());
    assert_snapshot!(stdout);
}
