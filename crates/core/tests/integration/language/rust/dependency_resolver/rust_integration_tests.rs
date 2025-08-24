use insta::assert_snapshot;
use lintric_core::analyze_code;
use std::fs;

macro_rules! test_dependency_resolver_analysis {
    ($test_name:ident, $fixture_name:literal, $ir_snapshot:literal) => {
        #[test]
        fn $test_name() {
            let file_path = format!(
                "tests/integration/language/rust/dependency_resolver/fixtures/{}.rs",
                $fixture_name
            );
            let (ir, _result) = analyze_code(file_path.clone()).unwrap();

            let source_code = fs::read_to_string(&file_path).unwrap();
            let ast_result = lintric_core::get_s_expression_from_content(
                source_code.clone(),
                lintric_core::Language::Rust,
            );
            let ast_string = ast_result.unwrap_or_else(|e| format!("AST Error: {}", e));

            let ir_snapshot_content = format!(
                "Source Code:\n{}\n\nAST:\n{}\n\nIR:\n{:#?}",
                source_code, ast_string, ir
            );

            assert_snapshot!($ir_snapshot, ir_snapshot_content);

            // Verify dependencies are detected
            assert!(!ir.dependencies.is_empty(), "Should have dependencies");
        }
    };
}

test_dependency_resolver_analysis!(
    test_basic_symbol_resolution,
    "basic_symbol_resolution",
    "dependency_resolver_basic_symbol_resolution_ir"
);

test_dependency_resolver_analysis!(
    test_method_resolution,
    "method_resolution",
    "dependency_resolver_method_resolution_ir"
);

test_dependency_resolver_analysis!(
    test_generic_resolution,
    "generic_resolution",
    "dependency_resolver_generic_resolution_ir"
);

test_dependency_resolver_analysis!(
    test_complex_trait_hierarchy,
    "complex_trait_hierarchy",
    "dependency_resolver_complex_trait_hierarchy_ir"
);

test_dependency_resolver_analysis!(
    test_scope_resolution,
    "scope_resolution",
    "dependency_resolver_scope_resolution_ir"
);

test_dependency_resolver_analysis!(
    test_module_resolution,
    "module_resolution",
    "dependency_resolver_module_resolution_ir"
);

test_dependency_resolver_analysis!(
    test_forward_references,
    "forward_references",
    "dependency_resolver_forward_references_ir"
);

test_dependency_resolver_analysis!(
    test_closure_dependencies,
    "closure_dependencies",
    "dependency_resolver_closure_dependencies_ir"
);

test_dependency_resolver_analysis!(
    test_advanced_generics,
    "advanced_generics",
    "dependency_resolver_advanced_generics_ir"
);

test_dependency_resolver_analysis!(
    test_macro_dependencies,
    "macro_dependencies",
    "dependency_resolver_macro_dependencies_ir"
);

test_dependency_resolver_analysis!(
    test_async_dependencies,
    "async_dependencies",
    "dependency_resolver_async_dependencies_ir"
);

test_dependency_resolver_analysis!(
    test_pattern_matching,
    "pattern_matching",
    "dependency_resolver_pattern_matching_ir"
);

#[test]
fn test_specific_dependency_types() {
    let file_path =
        "tests/integration/language/rust/dependency_resolver/fixtures/method_resolution.rs";
    let (ir, _result) = analyze_code(file_path.to_string()).unwrap();

    // Check that we have some dependencies
    assert!(!ir.dependencies.is_empty(), "Should have some dependencies");
}

#[test]
fn test_trait_method_dependencies() {
    let file_path =
        "tests/integration/language/rust/dependency_resolver/fixtures/complex_trait_hierarchy.rs";
    let (ir, _result) = analyze_code(file_path.to_string()).unwrap();

    // Verify trait method dependencies are resolved
    let trait_methods: Vec<_> = ir
        .dependencies
        .iter()
        .filter(|d| d.symbol == "name" || d.symbol == "speak" || d.symbol == "fur_color")
        .collect();

    assert!(
        !trait_methods.is_empty(),
        "Should resolve trait method dependencies"
    );
}

#[test]
fn test_generic_type_dependencies() {
    let file_path =
        "tests/integration/language/rust/dependency_resolver/fixtures/generic_resolution.rs";
    let (ir, _result) = analyze_code(file_path.to_string()).unwrap();

    // Check for generic-related dependencies
    let generic_deps: Vec<_> = ir
        .dependencies
        .iter()
        .filter(|d| d.symbol.contains("Container") || d.symbol == "process")
        .collect();

    assert!(
        !generic_deps.is_empty(),
        "Should resolve generic type dependencies"
    );
}
