use lintric_core::languages::rust::dependency_resolver::nested_scope_resolver::*;
use lintric_core::models::{Definition, DefinitionType, Position, ScopeTree, ScopeType};

fn create_complex_scope_tree() -> ScopeTree {
    let mut tree = ScopeTree::new();

    // Global scope (0) already exists

    // Function scope
    let func_scope = tree.create_scope(
        Some(0),
        ScopeType::Function,
        Position { start_line: 1, start_column: 1, end_line: 1, end_column: 1 },
        Position { start_line: 20, start_column: 1, end_line: 20, end_column: 1 },
    );

    // Block scope within function
    let block_scope = tree.create_scope(
        Some(func_scope),
        ScopeType::Block,
        Position { start_line: 5, start_column: 5, end_line: 5, end_column: 5 },
        Position { start_line: 10, start_column: 5, end_line: 10, end_column: 5 },
    );

    // Nested function scope
    let nested_func_scope = tree.create_scope(
        Some(func_scope),
        ScopeType::Function,
        Position { start_line: 12, start_column: 8, end_line: 12, end_column: 8 },
        Position { start_line: 16, start_column: 8, end_line: 16, end_column: 8 },
    );

    // Add symbols to different scopes
    if let Some(scope) = tree.get_scope_mut(func_scope) {
        scope.add_symbol(
            "func_var".to_string(),
            Definition::new_simple(
                "func_var".to_string(),
                DefinitionType::VariableDefinition,
                Position { start_line: 2, start_column: 10, end_line: 2, end_column: 10 },
            ),
        );
    }

    if let Some(scope) = tree.get_scope_mut(block_scope) {
        scope.add_symbol(
            "block_var".to_string(),
            Definition::new_simple(
                "block_var".to_string(),
                DefinitionType::VariableDefinition,
                Position { start_line: 6, start_column: 10, end_line: 6, end_column: 10 },
            ),
        );
    }

    if let Some(scope) = tree.get_scope_mut(nested_func_scope) {
        scope.add_symbol(
            "nested_var".to_string(),
            Definition::new_simple(
                "nested_var".to_string(),
                DefinitionType::VariableDefinition,
                Position { start_line: 13, start_column: 10, end_line: 13, end_column: 10 },
            ),
        );
    }

    tree
}

#[test]
fn test_scope_chain_traversal() {
    let scope_tree = create_complex_scope_tree();
    let walker = ScopeChainWalker::new(3, &scope_tree); // nested_func_scope

    // Should be able to find func_var from parent scope
    let result = walker.find_symbol_in_chain("func_var");
    assert!(result.is_some());
    let (scope_id, definition) = result.unwrap();
    assert_eq!(definition.name, "func_var");
    assert_eq!(scope_id, 1); // func_scope
}

#[test]
fn test_scope_accessibility_rules() {
    let scope_tree = create_complex_scope_tree();
    let walker = ScopeChainWalker::new(1, &scope_tree); // func_scope

    // Child can access parent
    assert!(walker.check_accessibility(3, 1)); // nested_func_scope -> func_scope
    assert!(walker.check_accessibility(2, 1)); // block_scope -> func_scope

    // Parent cannot access child
    assert!(!walker.check_accessibility(1, 3)); // func_scope -> nested_func_scope
    assert!(!walker.check_accessibility(1, 2)); // func_scope -> block_scope

    // Same scope
    assert!(walker.check_accessibility(1, 1));
}

#[test]
fn test_scope_distance_calculation() {
    let scope_tree = create_complex_scope_tree();
    let resolver = NestedScopeResolver::new(scope_tree);

    // Distance from nested function to function
    let distance = resolver.calculate_scope_distance(3, 1);
    assert_eq!(distance, 1);

    // Distance from block to function
    let distance = resolver.calculate_scope_distance(2, 1);
    assert_eq!(distance, 1);

    // Distance from nested function to global
    let distance = resolver.calculate_scope_distance(3, 0);
    assert_eq!(distance, 2);

    // Same scope
    let distance = resolver.calculate_scope_distance(1, 1);
    assert_eq!(distance, 0);

    // Not in parent chain
    let distance = resolver.calculate_scope_distance(2, 3);
    assert_eq!(distance, usize::MAX);
}

#[test]
fn test_closure_capture_types() {
    let by_value = CaptureType::ByValue;
    let by_ref = CaptureType::ByReference;
    let by_mut_ref = CaptureType::ByMutableReference;

    assert_eq!(by_value, CaptureType::ByValue);
    assert_eq!(by_ref, CaptureType::ByReference);
    assert_eq!(by_mut_ref, CaptureType::ByMutableReference);
    
    assert_ne!(by_value, by_ref);
    assert_ne!(by_ref, by_mut_ref);
}

#[test]
fn test_scope_search_result() {
    let definition = Definition::new_simple(
        "test_var".to_string(),
        DefinitionType::VariableDefinition,
        Position { start_line: 5, start_column: 10, end_line: 5, end_column: 10 },
    );

    let search_result = ScopeSearchResult {
        definition: definition.clone(),
        scope_id: 1,
        scope_distance: 2,
    };

    assert_eq!(search_result.definition.name, "test_var");
    assert_eq!(search_result.scope_id, 1);
    assert_eq!(search_result.scope_distance, 2);
}