use std::process::Command;
use std::fs;
use std::path::PathBuf;

#[test]
fn test_basic_analysis() {
    let temp_file_path = "temp_test_file.rs";
    fs::write(temp_file_path, "let a = 1;\nlet b = a + 1;\n").expect("Unable to write test file");

    let output = Command::new("cargo")
        .arg("run")
        .arg("--package")
        .arg("lintric-cli")
        .arg("--")
        .arg("--verbose")
        .arg(temp_file_path)
        .output()
        .expect("Failed to execute command");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let _stderr = String::from_utf8_lossy(&output.stderr);

    assert!(output.status.success());
    assert!(stdout.contains("| 1    | 0          | 0         | 0     | 0               |"));
    assert!(stdout.contains("| 2    | 1          | 1         | 1     | 1               |"));
    assert!(stdout.contains("Overall Complexity Score:"));

    fs::remove_file(temp_file_path).expect("Unable to remove test file");
}

#[test]
fn test_multiple_files_analysis() {
    let temp_dir = PathBuf::from("temp_test_dir");
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
        .arg(temp_dir.to_str().unwrap())
        .output()
        .expect("Failed to execute command");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let _stderr = String::from_utf8_lossy(&output.stderr);

    assert!(output.status.success());
    assert!(stdout.contains("| temp_test_dir/file1.rs | 2.30                     |"));
    assert!(stdout.contains("| temp_test_dir/file2.ts | 2.30                     |"));
    assert!(stdout.contains("Total Files Analyzed: 2"));

    fs::remove_dir_all(&temp_dir).expect("Unable to remove test directory");
}

#[test]
fn test_json_output() {
    let temp_file_path = "temp_test_file_json.rs";
    fs::write(temp_file_path, "let val = 10;\n").expect("Unable to write test file");

    let output = Command::new("cargo")
        .arg("run")
        .arg("--package")
        .arg("lintric-cli")
        .arg("--")
        .arg("--json")
        .arg(temp_file_path)
        .output()
        .expect("Failed to execute command");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let _stderr = String::from_utf8_lossy(&output.stderr);

    assert!(output.status.success());
    assert!(stdout.contains("\"file_path\": \"temp_test_file_json.rs\""));
    assert!(stdout.contains("\"line_metrics\": ["));
    assert!(stdout.contains("\"overall_complexity_score\":"));

    fs::remove_file(temp_file_path).expect("Unable to remove test file");
}

#[test]
fn test_complex_rust_analysis() {
    let fixture_path = "../core/tests/fixtures/complex_rust_code.rs";

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
    assert!(stdout.contains("| 8    | 1          | 1         | 1     | 1               |"));
    assert!(stdout.contains("| 16   | 6          | 47        | 1     | 4               |"));
    assert!(stdout.contains("| 17   | 6          | 51        | 2     | 5               |"));
    assert!(stdout.contains("| 22   | 1          | 1         | 1     | 1               |"));
    assert!(stdout.contains("| 23   | 1          | 1         | 2     | 2               |"));
    assert!(stdout.contains("| 27   | 1          | 1         | 1     | 1               |"));
    assert!(stdout.contains("| 28   | 2          | 3         | 2     | 2               |"));
    assert!(stdout.contains("| 30   | 1          | 15        | 1     | 1               |"));
    assert!(stdout.contains("| 31   | 1          | 11        | 1     | 1               |"));
}

#[test]
fn test_complex_typescript_analysis() {
    let fixture_path = "../core/tests/fixtures/complex_typescript_code.ts";

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
    assert!(stdout.contains("| 12   | 1          | 27        | 1     | 1               |"));
    assert!(stdout.contains("| 20   | 6          | 34        | 1     | 3               |"));
    assert!(stdout.contains("| 21   | 6          | 40        | 1     | 3               |"));
    assert!(stdout.contains("| 27   | 1          | 1         | 1     | 1               |"));
    assert!(stdout.contains("| 31   | 1          | 1         | 1     | 1               |"));
    assert!(stdout.contains("| 32   | 2          | 3         | 2     | 2               |"));
    assert!(stdout.contains("| 34   | 1          | 15        | 1     | 1               |"));
    assert!(stdout.contains("| 35   | 1          | 11        | 1     | 1               |"));
    assert!(stdout.contains("| 38   | 1          | 37        | 1     | 1               |"));
    assert!(stdout.contains("| 40   | 2          | 74        | 1     | 1               |"));
    assert!(stdout.contains("| 43   | 2          | 9         | 2     | 3               |"));
}
