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
        .arg("--quiet")
        .arg("--")
        .arg("--verbose")
        .arg(temp_file_path)
        .output()
        .expect("Failed to execute command");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    println!("stdout: {}", stdout);
    println!("stderr: {}", stderr);

    assert!(output.status.success());
    assert!(stdout.contains("Line 1: Total Dependencies = 0, Dependency Distance Cost = 0, Depth = 0, Transitive Dependencies = 0"));
    assert!(stdout.contains("Line 2: Total Dependencies = 1, Dependency Distance Cost = 1, Depth = 1, Transitive Dependencies = 1"));
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
        .arg("--quiet")
        .arg("--")
        .arg(temp_dir.to_str().unwrap())
        .output()
        .expect("Failed to execute command");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    println!("stdout: {}", stdout);
    println!("stderr: {}", stderr);

    assert!(output.status.success());
    assert!(stdout.contains("File: temp_test_dir/file1.rs, Overall Complexity Score:"));
    assert!(stdout.contains("File: temp_test_dir/file2.ts, Overall Complexity Score:"));
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
        .arg("--quiet")
        .arg("--")
        .arg("--json")
        .arg(temp_file_path)
        .output()
        .expect("Failed to execute command");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    println!("stdout: {}", stdout);
    println!("stderr: {}", stderr);

    assert!(output.status.success());
    assert!(stdout.contains("\"file_path\": \"temp_test_file_json.rs\""));
    assert!(stdout.contains("\"line_metrics\": ["));
    assert!(stdout.contains("\"overall_complexity_score\":"));

    fs::remove_file(temp_file_path).expect("Unable to remove test file");
}

#[test]
fn test_complex_rust_analysis() {
    let fixture_path = "tests/fixtures/complex_rust_code.rs";

    let output = Command::new("cargo")
        .arg("run")
        .arg("--package")
        .arg("lintric-cli")
        .arg("--quiet")
        .arg("--")
        .arg("--verbose")
        .arg(fixture_path)
        .output()
        .expect("Failed to execute command");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    println!("stdout: {}", stdout);
    println!("stderr: {}", stderr);

    assert!(output.status.success());
    assert!(stdout.contains("Line 8: Total Dependencies = 1, Dependency Distance Cost = 1, Depth = 1, Transitive Dependencies = 1"));
    assert!(stdout.contains("Line 16: Total Dependencies = 2, Dependency Distance Cost = 7, Depth = 1, Transitive Dependencies = 2"));
    assert!(stdout.contains("Line 17: Total Dependencies = 2, Dependency Distance Cost = 9, Depth = 1, Transitive Dependencies = 2"));
    assert!(stdout.contains("Line 22: Total Dependencies = 1, Dependency Distance Cost = 1, Depth = 1, Transitive Dependencies = 1"));
    assert!(stdout.contains("Line 23: Total Dependencies = 1, Dependency Distance Cost = 1, Depth = 2, Transitive Dependencies = 2"));
    assert!(stdout.contains("Line 27: Total Dependencies = 1, Dependency Distance Cost = 1, Depth = 1, Transitive Dependencies = 1"));
    assert!(stdout.contains("Line 28: Total Dependencies = 2, Dependency Distance Cost = 3, Depth = 2, Transitive Dependencies = 2"));
    assert!(stdout.contains("Line 30: Total Dependencies = 1, Dependency Distance Cost = 15, Depth = 1, Transitive Dependencies = 1"));
    assert!(stdout.contains("Line 31: Total Dependencies = 1, Dependency Distance Cost = 11, Depth = 1, Transitive Dependencies = 1"));
}

#[test]
fn test_complex_typescript_analysis() {
    let fixture_path = "tests/fixtures/complex_typescript_code.ts";

    let output = Command::new("cargo")
        .arg("run")
        .arg("--package")
        .arg("lintric-cli")
        .arg("--quiet")
        .arg("--")
        .arg("--verbose")
        .arg(fixture_path)
        .output()
        .expect("Failed to execute command");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    println!("stdout: {}", stdout);
    println!("stderr: {}", stderr);

    assert!(output.status.success());
    assert!(stdout.contains("Line 8: Total Dependencies = 1, Dependency Distance Cost = 1, Depth = 1, Transitive Dependencies = 1"));
    assert!(stdout.contains("Line 16: Total Dependencies = 4, Dependency Distance Cost = 27, Depth = 1, Transitive Dependencies = 3"));
    assert!(stdout.contains("Line 17: Total Dependencies = 4, Dependency Distance Cost = 31, Depth = 1, Transitive Dependencies = 3"));
    assert!(stdout.contains("Line 23: Total Dependencies = 1, Dependency Distance Cost = 1, Depth = 1, Transitive Dependencies = 1"));
    assert!(stdout.contains("Line 28: Total Dependencies = 2, Dependency Distance Cost = 3, Depth = 2, Transitive Dependencies = 2"));
    assert!(stdout.contains("Line 29: Total Dependencies = 0, Dependency Distance Cost = 0, Depth = 0, Transitive Dependencies = 0"));
    assert!(stdout.contains("Line 30: Total Dependencies = 1, Dependency Distance Cost = 15, Depth = 1, Transitive Dependencies = 1"));
    assert!(stdout.contains("Line 31: Total Dependencies = 1, Dependency Distance Cost = 11, Depth = 1, Transitive Dependencies = 1"));
}