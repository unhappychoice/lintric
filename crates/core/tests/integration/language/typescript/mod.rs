use insta::assert_snapshot;
use lintric_core::analyze_code;
use std::env;
use std::fs;

macro_rules! test_typescript_analysis {
    ($test_name:ident, $fixture_name:literal, $ir_snapshot:literal, $metrics_snapshot:literal) => {
        #[test]
        fn $test_name() {
            let file_path = format!(
                "tests/integration/language/typescript/fixtures/{}.ts",
                $fixture_name
            );
            let (ir, result) = analyze_code(file_path.clone()).unwrap();

            // Read source code for snapshot
            let source_code = fs::read_to_string(&file_path).unwrap();
            let ast_result = lintric_core::get_s_expression_from_content(
                source_code.clone(),
                lintric_core::Language::TypeScript,
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

test_typescript_analysis!(
    test_analyze_code_typescript,
    "basic_typescript_code",
    "typescript_basic_ir",
    "typescript_basic_metrics"
);

test_typescript_analysis!(
    test_typescript_class_method_dependency,
    "class_method_dependency",
    "class_method_dependency_ir",
    "class_method_dependency_metrics"
);

test_typescript_analysis!(
    test_typescript_import_dependency,
    "import_dependency",
    "import_dependency_ir",
    "import_dependency_metrics"
);

test_typescript_analysis!(
    test_typescript_function_parameter_dependency,
    "function_parameter_dependency",
    "function_parameter_dependency_ir",
    "function_parameter_dependency_metrics"
);

test_typescript_analysis!(
    test_typescript_arrow_function_parameter_dependency,
    "arrow_function_parameter_dependency",
    "arrow_function_parameter_dependency_ir",
    "arrow_function_parameter_dependency_metrics"
);

test_typescript_analysis!(
    test_typescript_hoisting,
    "hoisting_test",
    "hoisting_test_ir",
    "hoisting_test_metrics"
);
pub mod dependency_resolver;
