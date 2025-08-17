use insta::assert_snapshot;
use lintric_core::analyze_code;
use std::env;

#[test]
fn test_analyze_code_basic() {
    let file_path = "tests/rust/fixtures/basic_rust_code.rs".to_string();

    let (ir, result) = analyze_code(file_path.clone()).unwrap();

    assert_snapshot!("basic_ir", serde_json::to_string_pretty(&ir).unwrap());
    assert_snapshot!(
        "basic_metrics",
        serde_json::to_string_pretty(&result).unwrap()
    );
}

#[test]
fn test_rust_function_call_dependency() {
    let file_path = "tests/rust/fixtures/function_call_dependency.rs".to_string();
    let (ir, result) = analyze_code(file_path).unwrap();

    assert_snapshot!(
        "function_call_dependency_ir",
        serde_json::to_string_pretty(&ir).unwrap()
    );
    assert_snapshot!(
        "function_call_dependency_metrics",
        serde_json::to_string_pretty(&result).unwrap()
    );
}

#[test]
fn test_rust_struct_field_access_dependency() {
    let file_path = "tests/rust/fixtures/struct_field_access_dependency.rs".to_string();
    let (ir, result) = analyze_code(file_path).unwrap();

    assert_snapshot!(
        "struct_field_access_dependency_ir",
        serde_json::to_string_pretty(&ir).unwrap()
    );
    assert_snapshot!(
        "struct_field_access_dependency_metrics",
        serde_json::to_string_pretty(&result).unwrap()
    );
}

#[test]
fn test_rust_use_statements_dependency() {
    let file_path = "tests/rust/fixtures/use_statements_dependency.rs".to_string();
    let (ir, result) = analyze_code(file_path).unwrap();

    assert_snapshot!(
        "use_statements_dependency_ir",
        serde_json::to_string_pretty(&ir).unwrap()
    );
    assert_snapshot!(
        "use_statements_dependency_metrics",
        serde_json::to_string_pretty(&result).unwrap()
    );
}

#[test]
fn test_rust_macro_invocation_dependency() {
    let file_path = "tests/rust/fixtures/macro_invocation_dependency.rs".to_string();
    let (ir, result) = analyze_code(file_path).unwrap();

    assert_snapshot!(
        "macro_invocation_dependency_ir",
        serde_json::to_string_pretty(&ir).unwrap()
    );
    assert_snapshot!(
        "macro_invocation_dependency_metrics",
        serde_json::to_string_pretty(&result).unwrap()
    );
}

#[test]
fn test_rust_use_macro_dependency() {
    let file_path = "tests/rust/fixtures/use_macro_dependency.rs".to_string();
    let (ir, result) = analyze_code(file_path).unwrap();

    assert_snapshot!(
        "use_macro_dependency_ir",
        serde_json::to_string_pretty(&ir).unwrap()
    );
    assert_snapshot!(
        "use_macro_dependency_metrics",
        serde_json::to_string_pretty(&result).unwrap()
    );
}

#[test]
fn test_rust_closure_dependency() {
    let file_path = "tests/rust/fixtures/closure_dependency.rs".to_string();
    let (ir, result) = analyze_code(file_path).unwrap();

    assert_snapshot!(
        "closure_dependency_ir",
        serde_json::to_string_pretty(&ir).unwrap()
    );
    assert_snapshot!(
        "closure_dependency_metrics",
        serde_json::to_string_pretty(&result).unwrap()
    );
}

#[test]
fn test_pattern_match_bindings_definitions() {
    let file_path = "tests/rust/fixtures/pattern_match_bindings.rs".to_string();
    let (ir, result) = analyze_code(file_path).unwrap();

    assert_snapshot!(
        "pattern_match_bindings_ir",
        serde_json::to_string_pretty(&ir).unwrap()
    );
    assert_snapshot!(
        "pattern_match_bindings_metrics",
        serde_json::to_string_pretty(&result).unwrap()
    );
}

#[test]
fn test_rust_path_qualified_call_dependency() {
    let file_path = "tests/rust/fixtures/path_qualified_call_dependency.rs".to_string();
    let (ir, result) = analyze_code(file_path).unwrap();

    assert_snapshot!(
        "path_qualified_call_dependency_ir",
        serde_json::to_string_pretty(&ir).unwrap()
    );
    assert_snapshot!(
        "path_qualified_call_dependency_metrics",
        serde_json::to_string_pretty(&result).unwrap()
    );
}

#[test]
fn test_rust_method_call_dependency() {
    let file_path = "tests/rust/fixtures/method_call_dependency.rs".to_string();
    let (ir, result) = analyze_code(file_path).unwrap();

    assert_snapshot!(
        "method_call_dependency_ir",
        serde_json::to_string_pretty(&ir).unwrap()
    );
    assert_snapshot!(
        "method_call_dependency_metrics",
        serde_json::to_string_pretty(&result).unwrap()
    );
}

#[test]
fn test_rust_associated_function_and_ufcs_dependency() {
    let file_path = "tests/rust/fixtures/associated_function_and_ufcs.rs".to_string();
    let (ir, result) = analyze_code(file_path).unwrap();

    assert_snapshot!(
        "associated_function_and_ufcs_ir",
        serde_json::to_string_pretty(&ir).unwrap()
    );
    assert_snapshot!(
        "associated_function_and_ufcs_metrics",
        serde_json::to_string_pretty(&result).unwrap()
    );
}
