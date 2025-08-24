use insta::assert_snapshot;
use lintric_core::analyze_code;
use std::env;
use std::fs;

macro_rules! test_rust_analysis {
    ($test_name:ident, $fixture_name:literal, $ir_snapshot:literal, $metrics_snapshot:literal) => {
        #[test]
        fn $test_name() {
            let file_path = format!(
                "tests/integration/language/rust/fixtures/{}.rs",
                $fixture_name
            );
            let (ir, result) = analyze_code(file_path.clone()).unwrap();

            // Read source code for snapshot
            let source_code = fs::read_to_string(&file_path).unwrap();
            let ast_result = lintric_core::get_s_expression_from_content(
                source_code.clone(),
                lintric_core::Language::Rust,
            );
            let ast_string = ast_result.unwrap_or_else(|e| format!("AST Error: {}", e));

            // Create comprehensive IR snapshot
            let ir_snapshot_content = format!(
                "Source Code:\n{}\n\nAST:\n{}\n\nIR:\n{:#?}",
                source_code, ast_string, ir
            );

            assert_snapshot!($ir_snapshot, ir_snapshot_content);
            assert_snapshot!(
                $metrics_snapshot,
                serde_json::to_string_pretty(&result).unwrap()
            );
        }
    };
}

test_rust_analysis!(
    test_analyze_code_basic,
    "basic_rust_code",
    "basic_ir",
    "basic_metrics"
);

test_rust_analysis!(
    test_rust_function_call_dependency,
    "function_call_dependency",
    "function_call_dependency_ir",
    "function_call_dependency_metrics"
);

test_rust_analysis!(
    test_rust_struct_field_access_dependency,
    "struct_field_access_dependency",
    "struct_field_access_dependency_ir",
    "struct_field_access_dependency_metrics"
);

test_rust_analysis!(
    test_rust_use_statements_dependency,
    "use_statements_dependency",
    "use_statements_dependency_ir",
    "use_statements_dependency_metrics"
);

test_rust_analysis!(
    test_rust_macro_invocation_dependency,
    "macro_invocation_dependency",
    "macro_invocation_dependency_ir",
    "macro_invocation_dependency_metrics"
);

test_rust_analysis!(
    test_rust_use_macro_dependency,
    "use_macro_dependency",
    "use_macro_dependency_ir",
    "use_macro_dependency_metrics"
);

test_rust_analysis!(
    test_rust_closure_dependency,
    "closure_dependency",
    "closure_dependency_ir",
    "closure_dependency_metrics"
);

test_rust_analysis!(
    test_pattern_match_bindings_definitions,
    "pattern_match_bindings",
    "pattern_match_bindings_ir",
    "pattern_match_bindings_metrics"
);

test_rust_analysis!(
    test_rust_path_qualified_call_dependency,
    "path_qualified_call_dependency",
    "path_qualified_call_dependency_ir",
    "path_qualified_call_dependency_metrics"
);

test_rust_analysis!(
    test_rust_method_call_dependency,
    "method_call_dependency",
    "method_call_dependency_ir",
    "method_call_dependency_metrics"
);

test_rust_analysis!(
    test_rust_associated_function_and_ufcs_dependency,
    "associated_function_and_ufcs",
    "associated_function_and_ufcs_ir",
    "associated_function_and_ufcs_metrics"
);

test_rust_analysis!(
    test_forward_reference_scope,
    "forward_reference_scope",
    "forward_reference_scope_ir",
    "forward_reference_scope_metrics"
);

test_rust_analysis!(
    test_hoisting,
    "hoisting_test",
    "hoisting_test_ir",
    "hoisting_test_metrics"
);

test_rust_analysis!(
    test_dependency_resolution_bugs,
    "dependency_resolution_bugs",
    "dependency_resolution_bugs_ir",
    "dependency_resolution_bugs_metrics"
);

pub mod dependency_resolution_validation_tests;
pub mod dependency_resolver;
