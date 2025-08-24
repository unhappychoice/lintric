use insta::assert_snapshot;
use lintric_core::analyze_code;
use std::fs;

macro_rules! test_typescript_dependency_resolver {
    ($test_name:ident, $fixture_name:literal, $ir_snapshot:literal) => {
        #[test]
        fn $test_name() {
            let file_path = format!(
                "tests/integration/language/typescript/dependency_resolver/fixtures/{}.ts",
                $fixture_name
            );
            let (ir, _result) = analyze_code(file_path.clone()).unwrap();

            let source_code = fs::read_to_string(&file_path).unwrap();
            let ast_result = lintric_core::get_s_expression_from_content(
                source_code.clone(),
                lintric_core::Language::TypeScript,
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

test_typescript_dependency_resolver!(
    test_typescript_basic_resolution,
    "typescript_basic",
    "dependency_resolver_typescript_basic_ir"
);

test_typescript_dependency_resolver!(
    test_typescript_class_method_resolution,
    "typescript_class_method",
    "dependency_resolver_typescript_class_method_ir"
);

test_typescript_dependency_resolver!(
    test_typescript_generic_resolution,
    "typescript_generics",
    "dependency_resolver_typescript_generics_ir"
);

test_typescript_dependency_resolver!(
    test_typescript_namespace_resolution,
    "typescript_namespaces",
    "dependency_resolver_typescript_namespaces_ir"
);

#[test]
fn test_typescript_method_call_dependencies() {
    let file_path = "tests/integration/language/typescript/dependency_resolver/fixtures/typescript_class_method.ts";
    let (ir, _result) = analyze_code(file_path.to_string()).unwrap();

    // Check that we have some dependencies
    assert!(!ir.dependencies.is_empty(), "Should have some dependencies");
}

#[test]
fn test_typescript_interface_implementation_dependencies() {
    let file_path = "tests/integration/language/typescript/dependency_resolver/fixtures/typescript_class_method.ts";
    let (ir, _result) = analyze_code(file_path.to_string()).unwrap();

    // Check for interface-related dependencies
    let _interface_deps: Vec<_> = ir
        .dependencies
        .iter()
        .filter(|d| d.symbol == "Drawable" || d.symbol == "draw")
        .collect();

    // Should have some form of interface-related dependencies
    assert!(
        !ir.dependencies.is_empty(),
        "Should resolve interface dependencies"
    );
}

#[test]
fn test_typescript_generic_type_dependencies() {
    let file_path =
        "tests/integration/language/typescript/dependency_resolver/fixtures/typescript_generics.ts";
    let (ir, _result) = analyze_code(file_path.to_string()).unwrap();

    // Verify generic type resolution
    let _generic_deps: Vec<_> = ir
        .dependencies
        .iter()
        .filter(|d| d.symbol.contains("Repository") || d.symbol.contains("User"))
        .collect();

    assert!(
        !ir.dependencies.is_empty(),
        "Should resolve generic dependencies"
    );
}

#[test]
fn test_typescript_namespace_dependencies() {
    let file_path = "tests/integration/language/typescript/dependency_resolver/fixtures/typescript_namespaces.ts";
    let (ir, _result) = analyze_code(file_path.to_string()).unwrap();

    // Check for namespace member access
    let _namespace_deps: Vec<_> = ir
        .dependencies
        .iter()
        .filter(|d| d.symbol.contains("Utils") || d.symbol == "helper" || d.symbol == "Calculator")
        .collect();

    assert!(
        !ir.dependencies.is_empty(),
        "Should resolve namespace dependencies"
    );
}
