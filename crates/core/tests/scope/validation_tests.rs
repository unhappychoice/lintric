use lintric_core::{
    models::{Accessibility, Definition, DefinitionType, Position, Scope, ScopeType, SymbolTable},
    scope_aware_resolver::ScopeValidator,
};

#[test]
fn test_scope_structure_validation_valid() {
    let mut symbol_table = SymbolTable::new();

    // Create a valid scope hierarchy
    let func_scope_id = symbol_table.scopes.create_scope(
        Some(0),
        ScopeType::Function,
        Position {
            start_line: 5,
            start_column: 1,
            end_line: 5,
            end_column: 1,
        },
        Position {
            start_line: 10,
            start_column: 1,
            end_line: 10,
            end_column: 1,
        },
    );

    let _block_scope_id = symbol_table.scopes.create_scope(
        Some(func_scope_id),
        ScopeType::Block,
        Position {
            start_line: 7,
            start_column: 5,
            end_line: 7,
            end_column: 5,
        },
        Position {
            start_line: 9,
            start_column: 5,
            end_line: 9,
            end_column: 5,
        },
    );

    let validator = ScopeValidator::new("rust".to_string());
    let errors = validator.validate_scope_structure(&symbol_table).unwrap();

    assert_eq!(errors.len(), 0); // Should have no errors for valid structure
}

#[test]
fn test_scope_structure_validation_invalid_parent() {
    let mut symbol_table = SymbolTable::new();

    // Manually create a scope with invalid parent reference
    let invalid_scope = Scope::new(
        999,
        Some(1000), // Non-existent parent
        ScopeType::Function,
        Position {
            start_line: 5,
            start_column: 1,
            end_line: 5,
            end_column: 1,
        },
        Position {
            start_line: 10,
            start_column: 1,
            end_line: 10,
            end_column: 1,
        },
    );

    symbol_table.scopes.scopes.insert(999, invalid_scope);

    let validator = ScopeValidator::new("rust".to_string());
    let errors = validator.validate_scope_structure(&symbol_table).unwrap();

    assert!(errors.len() > 0); // Should have validation errors
    assert!(errors
        .iter()
        .any(|e| e.message.contains("non-existent parent")));
}

#[test]
fn test_accessibility_validation() {
    let mut symbol_table = SymbolTable::new();

    // Create a function scope (non-module scope)
    let func_scope_id = symbol_table.scopes.create_scope(
        Some(0),
        ScopeType::Function,
        Position {
            start_line: 5,
            start_column: 1,
            end_line: 5,
            end_column: 1,
        },
        Position {
            start_line: 10,
            start_column: 1,
            end_line: 10,
            end_column: 1,
        },
    );

    // Add a private symbol to a function scope (should be invalid)
    let private_def = Definition::new_simple(
        "private_var".to_string(),
        DefinitionType::VariableDefinition,
        Position {
            start_line: 6,
            start_column: 5,
            end_line: 6,
            end_column: 5,
        },
    );

    symbol_table.add_symbol(
        "private_var".to_string(),
        private_def,
        func_scope_id,
        Accessibility::Private, // Private in function scope should be invalid
        false,
    );

    let validator = ScopeValidator::new("rust".to_string());
    let errors = validator.validate_scope_structure(&symbol_table).unwrap();

    // Should have accessibility validation errors
    assert!(errors
        .iter()
        .any(|e| e.message.contains("Private symbol") && e.message.contains("non-module scope")));
}

#[test]
fn test_valid_module_private_symbols() {
    let mut symbol_table = SymbolTable::new();

    // Create a module scope
    let module_scope_id = symbol_table.scopes.create_scope(
        Some(0),
        ScopeType::Module,
        Position {
            start_line: 5,
            start_column: 1,
            end_line: 5,
            end_column: 1,
        },
        Position {
            start_line: 15,
            start_column: 1,
            end_line: 15,
            end_column: 1,
        },
    );

    // Add private and public symbols to module scope
    let private_def = Definition::new_simple(
        "private_func".to_string(),
        DefinitionType::FunctionDefinition,
        Position {
            start_line: 6,
            start_column: 5,
            end_line: 6,
            end_column: 5,
        },
    );

    let public_def = Definition::new_simple(
        "public_func".to_string(),
        DefinitionType::FunctionDefinition,
        Position {
            start_line: 10,
            start_column: 5,
            end_line: 10,
            end_column: 5,
        },
    );

    symbol_table.add_symbol(
        "private_func".to_string(),
        private_def,
        module_scope_id,
        Accessibility::Private,
        false,
    );

    symbol_table.add_symbol(
        "public_func".to_string(),
        public_def,
        module_scope_id,
        Accessibility::Public,
        false,
    );

    let validator = ScopeValidator::new("rust".to_string());
    let errors = validator.validate_scope_structure(&symbol_table).unwrap();

    // Should have no errors for valid module accessibility
    let accessibility_errors: Vec<_> = errors
        .iter()
        .filter(|e| e.message.contains("Private symbol"))
        .collect();
    assert_eq!(accessibility_errors.len(), 0);
}

#[test]
fn test_variable_hoisting_validation_typescript() {
    let symbol_table = SymbolTable::new();

    let validator = ScopeValidator::new("typescript".to_string());
    let _errors = validator.validate_variable_hoisting(&symbol_table).unwrap();

    // For now, just ensure the method works without errors
    // More specific hoisting validation can be added later
}

#[test]
fn test_variable_hoisting_validation_rust() {
    let symbol_table = SymbolTable::new();

    let validator = ScopeValidator::new("rust".to_string());
    let errors = validator.validate_variable_hoisting(&symbol_table).unwrap();

    // Rust generally doesn't have hoisting issues, so should be empty
    assert_eq!(errors.len(), 0);
}

#[test]
fn test_scope_nesting_depth() {
    let mut symbol_table = SymbolTable::new();
    let mut current_scope = 0;

    // Create deeply nested scopes
    for i in 1..=10 {
        let scope_id = symbol_table.scopes.create_scope(
            Some(current_scope),
            ScopeType::Block,
            Position {
                start_line: i,
                start_column: 1,
                end_line: i,
                end_column: 1,
            },
            Position {
                start_line: i + 1,
                start_column: 1,
                end_line: i + 1,
                end_column: 1,
            },
        );
        current_scope = scope_id;
    }

    let validator = ScopeValidator::new("rust".to_string());
    let errors = validator.validate_scope_structure(&symbol_table).unwrap();

    // Deep nesting should be structurally valid
    assert_eq!(errors.len(), 0);

    // Verify that we can traverse the parent chain
    let parent_scopes = symbol_table.scopes.get_parent_scopes(current_scope);
    assert_eq!(parent_scopes.len(), 10); // Should have 10 ancestors (including global)
}

#[test]
fn test_symbol_shadowing_detection() {
    let mut symbol_table = SymbolTable::new();

    let func_scope_id = symbol_table.scopes.create_scope(
        Some(0),
        ScopeType::Function,
        Position {
            start_line: 2,
            start_column: 1,
            end_line: 2,
            end_column: 1,
        },
        Position {
            start_line: 10,
            start_column: 1,
            end_line: 10,
            end_column: 1,
        },
    );

    let block_scope_id = symbol_table.scopes.create_scope(
        Some(func_scope_id),
        ScopeType::Block,
        Position {
            start_line: 5,
            start_column: 5,
            end_line: 5,
            end_column: 5,
        },
        Position {
            start_line: 8,
            start_column: 5,
            end_line: 8,
            end_column: 5,
        },
    );

    // Add the same variable name in different scopes (shadowing)
    let outer_def = Definition::new_simple(
        "x".to_string(),
        DefinitionType::VariableDefinition,
        Position {
            start_line: 3,
            start_column: 5,
            end_line: 3,
            end_column: 5,
        },
    );

    let inner_def = Definition::new_simple(
        "x".to_string(),
        DefinitionType::VariableDefinition,
        Position {
            start_line: 6,
            start_column: 9,
            end_line: 6,
            end_column: 9,
        },
    );

    symbol_table.add_symbol(
        "x".to_string(),
        outer_def,
        func_scope_id,
        Accessibility::ScopeLocal,
        false,
    );
    symbol_table.add_symbol(
        "x".to_string(),
        inner_def,
        block_scope_id,
        Accessibility::ScopeLocal,
        false,
    );

    // Verify that lookup from inner scope finds both definitions
    let definitions = symbol_table.lookup_symbol_in_scope("x", block_scope_id);
    assert_eq!(definitions.len(), 2);

    // Verify that lookup from outer scope finds only outer definition
    let definitions = symbol_table.lookup_symbol_in_scope("x", func_scope_id);
    assert_eq!(definitions.len(), 1);
    assert_eq!(definitions[0].position.start_line, 3);
}
