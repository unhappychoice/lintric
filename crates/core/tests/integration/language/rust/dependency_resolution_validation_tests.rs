use lintric_core::analyze_code;
use std::path::PathBuf;

/// Test to validate specific dependency resolution edge cases that were previously broken
#[test]
fn test_constructor_dependencies_from_imports() {
    let mut fixture_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    fixture_path.push("tests/integration/language/rust/fixtures/dependency_resolution_bugs.rs");

    let (ir, _result) = analyze_code(fixture_path.to_string_lossy().to_string()).unwrap();
    let dependencies = &ir.dependencies;

    // HashMap::new() on line 22 should have dependency to HashMap import on line 1
    let hashmap_new_deps: Vec<_> = dependencies
        .iter()
        .filter(|dep| dep.source_line == 22 && dep.symbol == "HashMap")
        .collect();

    assert!(
        hashmap_new_deps.iter().any(|dep| dep.target_line == 1),
        "Missing dependency from HashMap::new() call to HashMap import. Found dependencies: {:?}",
        hashmap_new_deps
    );
}

#[test]
fn test_struct_field_type_dependencies() {
    let mut fixture_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    fixture_path.push("tests/integration/language/rust/fixtures/dependency_resolution_bugs.rs");

    let (ir, _result) = analyze_code(fixture_path.to_string_lossy().to_string()).unwrap();
    let dependencies = &ir.dependencies;

    // Struct field type on line 5 should depend on HashMap import on line 1
    let field_type_deps: Vec<_> = dependencies
        .iter()
        .filter(|dep| dep.source_line == 5 && dep.symbol == "HashMap")
        .collect();

    assert!(
        field_type_deps.iter().any(|dep| dep.target_line == 1),
        "Missing dependency from struct field type to import. Found dependencies: {:?}",
        field_type_deps
    );
}

#[test]
fn test_function_parameter_type_dependencies() {
    let mut fixture_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    fixture_path.push("tests/integration/language/rust/fixtures/dependency_resolution_bugs.rs");

    let (ir, _result) = analyze_code(fixture_path.to_string_lossy().to_string()).unwrap();
    let dependencies = &ir.dependencies;

    // Function parameter type on line 9 should depend on HashMap import on line 1
    let param_type_deps: Vec<_> = dependencies
        .iter()
        .filter(|dep| dep.source_line == 9 && dep.symbol == "HashMap")
        .collect();

    assert!(
        param_type_deps.iter().any(|dep| dep.target_line == 1),
        "Missing dependency from function parameter type to import. Found dependencies: {:?}",
        param_type_deps
    );
}

#[test]
fn test_no_duplicate_dependencies_in_function_calls() {
    let mut fixture_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    fixture_path.push("tests/integration/language/rust/fixtures/use_statements_dependency.rs");

    let (ir, _result) = analyze_code(fixture_path.to_string_lossy().to_string()).unwrap();
    let dependencies = &ir.dependencies;

    // Count dependencies from line 14 (my_function call) with symbol "my_function"
    let function_call_deps: Vec<_> = dependencies
        .iter()
        .filter(|dep| dep.source_line == 14 && dep.symbol == "my_function")
        .collect();

    // Should have exactly 1 dependency, not 2 (no duplication)
    assert_eq!(
        function_call_deps.len(),
        1,
        "Function call should have exactly 1 dependency, not duplicated. Found: {:?}",
        function_call_deps
    );
}

#[test]
fn test_external_method_calls_no_incorrect_dependencies() {
    let mut fixture_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    fixture_path.push("tests/integration/language/rust/fixtures/dependency_resolution_bugs.rs");

    let (ir, _result) = analyze_code(fixture_path.to_string_lossy().to_string()).unwrap();
    let dependencies = &ir.dependencies;

    // Vec::new() on line 24 should NOT have dependency to TestStruct::new on line 9
    let vec_new_deps: Vec<_> = dependencies
        .iter()
        .filter(|dep| dep.source_line == 24 && dep.symbol == "new" && dep.target_line == 9)
        .collect();

    assert!(
        vec_new_deps.is_empty(),
        "Vec::new() should not depend on TestStruct::new. Found incorrect dependencies: {:?}",
        vec_new_deps
    );

    // HashMap::new() on line 22 should NOT have dependency to TestStruct::new on line 9
    let hashmap_new_deps: Vec<_> = dependencies
        .iter()
        .filter(|dep| dep.source_line == 22 && dep.symbol == "new" && dep.target_line == 9)
        .collect();

    assert!(
        hashmap_new_deps.is_empty(),
        "HashMap::new() should not depend on TestStruct::new. Found incorrect dependencies: {:?}",
        hashmap_new_deps
    );

    // map.insert and vec_data.push should not have any dependencies to local methods
    let insert_deps: Vec<_> = dependencies
        .iter()
        .filter(|dep| dep.source_line == 23 && dep.symbol == "insert")
        .collect();

    assert!(
        insert_deps.is_empty(),
        "map.insert should not have dependencies to local methods. Found: {:?}",
        insert_deps
    );

    let push_deps: Vec<_> = dependencies
        .iter()
        .filter(|dep| dep.source_line == 25 && dep.symbol == "push")
        .collect();

    assert!(
        push_deps.is_empty(),
        "vec_data.push should not have dependencies to local methods. Found: {:?}",
        push_deps
    );
}

#[test]
fn test_usage_nodes_no_duplication() {
    let mut fixture_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    fixture_path.push("tests/integration/language/rust/fixtures/use_statements_dependency.rs");

    let (ir, _result) = analyze_code(fixture_path.to_string_lossy().to_string()).unwrap();
    let usage_nodes = &ir.usage;

    // Count usage nodes for my_function on line 14
    let my_function_usages: Vec<_> = usage_nodes
        .iter()
        .filter(|usage| usage.name == "my_function" && usage.position.start_line == 14)
        .collect();

    // Should have exactly 1 usage node (CallExpression), not an additional Identifier
    assert_eq!(
        my_function_usages.len(),
        1,
        "Function call should have exactly 1 usage node, not duplicated. Found: {:?}",
        my_function_usages
    );

    // Ensure it's the CallExpression type
    assert_eq!(
        my_function_usages[0].kind,
        lintric_core::models::UsageKind::CallExpression,
        "Function call usage should be CallExpression type"
    );
}
