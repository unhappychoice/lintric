use insta::assert_snapshot;
use lintric_core::analyze_code;
use serde_json;
use std::env;

#[test]
fn test_analyze_code_typescript() {
    let file_path = "tests/typescript/fixtures/basic_typescript_code.ts".to_string();
    let (ir, result) = analyze_code(file_path.clone()).unwrap();

    assert_snapshot!(
        "typescript_basic_ir",
        serde_json::to_string_pretty(&ir).unwrap()
    );
    assert_snapshot!(
        "typescript_basic_metrics",
        serde_json::to_string_pretty(&result).unwrap()
    );
}

#[test]
fn test_typescript_class_method_dependency() {
    let file_path = "tests/typescript/fixtures/class_method_dependency.ts".to_string();
    let (ir, result) = analyze_code(file_path.clone()).unwrap();

    assert_snapshot!(
        "class_method_dependency_ir",
        serde_json::to_string_pretty(&ir).unwrap()
    );
    assert_snapshot!(
        "class_method_dependency_metrics",
        serde_json::to_string_pretty(&result).unwrap()
    );
}

#[test]
fn test_typescript_import_dependency() {
    let file_path = "tests/typescript/fixtures/import_dependency.ts".to_string();
    let (ir, result) = analyze_code(file_path.clone()).unwrap();

    assert_snapshot!(
        "import_dependency_ir",
        serde_json::to_string_pretty(&ir).unwrap()
    );
    assert_snapshot!(
        "import_dependency_metrics",
        serde_json::to_string_pretty(&result).unwrap()
    );
}

#[test]
fn test_typescript_function_parameter_dependency() {
    let file_path = "tests/typescript/fixtures/function_parameter_dependency.ts".to_string();
    let (ir, result) = analyze_code(file_path.clone()).unwrap();

    assert_snapshot!(
        "function_parameter_dependency_ir",
        serde_json::to_string_pretty(&ir).unwrap()
    );
    assert_snapshot!(
        "function_parameter_dependency_metrics",
        serde_json::to_string_pretty(&result).unwrap()
    );
}

#[test]
fn test_typescript_arrow_function_parameter_dependency() {
    let file_path = "tests/typescript/fixtures/arrow_function_parameter_dependency.ts".to_string();
    let (ir, result) = analyze_code(file_path.clone()).unwrap();

    assert_snapshot!(
        "arrow_function_parameter_dependency_ir",
        serde_json::to_string_pretty(&ir).unwrap()
    );
    assert_snapshot!(
        "arrow_function_parameter_dependency_metrics",
        serde_json::to_string_pretty(&result).unwrap()
    );
}
