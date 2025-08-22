use lintric_core::{
    models::{
        Accessibility, Definition, DefinitionType, Position, ScopeType, SymbolTable, Usage,
        UsageKind,
    },
    scope_aware_resolver::{DefaultScopeAwareResolver, ScopeAwareDependencyResolver},
};

#[test]
fn test_scope_chain_resolution() {
    let mut symbol_table = SymbolTable::new();

    // Create nested scopes: global -> function -> block
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
            start_line: 15,
            start_column: 1,
            end_line: 15,
            end_column: 1,
        },
    );

    let block_scope_id = symbol_table.scopes.create_scope(
        Some(func_scope_id),
        ScopeType::Block,
        Position {
            start_line: 8,
            start_column: 5,
            end_line: 8,
            end_column: 5,
        },
        Position {
            start_line: 12,
            start_column: 5,
            end_line: 12,
            end_column: 5,
        },
    );

    // Add symbols at different scope levels
    let global_def = Definition::new_simple(
        "global_var".to_string(),
        DefinitionType::VariableDefinition,
        Position {
            start_line: 1,
            start_column: 1,
            end_line: 1,
            end_column: 1,
        },
    );

    let func_def = Definition::new_simple(
        "func_var".to_string(),
        DefinitionType::VariableDefinition,
        Position {
            start_line: 6,
            start_column: 5,
            end_line: 6,
            end_column: 5,
        },
    );

    let block_def = Definition::new_simple(
        "block_var".to_string(),
        DefinitionType::VariableDefinition,
        Position {
            start_line: 9,
            start_column: 9,
            end_line: 9,
            end_column: 9,
        },
    );

    let shadowing_def = Definition::new_simple(
        "global_var".to_string(), // Same name as global variable
        DefinitionType::VariableDefinition,
        Position {
            start_line: 10,
            start_column: 9,
            end_line: 10,
            end_column: 9,
        },
    );

    symbol_table.add_symbol(
        "global_var".to_string(),
        global_def,
        0,
        Accessibility::Public,
        false,
    );
    symbol_table.add_symbol(
        "func_var".to_string(),
        func_def,
        func_scope_id,
        Accessibility::ScopeLocal,
        false,
    );
    symbol_table.add_symbol(
        "block_var".to_string(),
        block_def,
        block_scope_id,
        Accessibility::ScopeLocal,
        false,
    );
    symbol_table.add_symbol(
        "global_var".to_string(),
        shadowing_def,
        block_scope_id,
        Accessibility::ScopeLocal,
        false,
    );

    let resolver = DefaultScopeAwareResolver::new("rust".to_string());

    // Test resolution from block scope
    let global_lookup =
        resolver.resolve_scope_chain_lookup("global_var", block_scope_id, &symbol_table);
    assert_eq!(global_lookup.len(), 2); // Both global and shadowing definitions

    let func_lookup =
        resolver.resolve_scope_chain_lookup("func_var", block_scope_id, &symbol_table);
    assert_eq!(func_lookup.len(), 1);
    assert_eq!(func_lookup[0].position.start_line, 6);

    let block_lookup =
        resolver.resolve_scope_chain_lookup("block_var", block_scope_id, &symbol_table);
    assert_eq!(block_lookup.len(), 1);
    assert_eq!(block_lookup[0].position.start_line, 9);

    // Test resolution from function scope (shouldn't see block variables)
    let block_lookup_from_func =
        resolver.resolve_scope_chain_lookup("block_var", func_scope_id, &symbol_table);
    assert_eq!(block_lookup_from_func.len(), 0);

    let global_lookup_from_func =
        resolver.resolve_scope_chain_lookup("global_var", func_scope_id, &symbol_table);
    assert_eq!(global_lookup_from_func.len(), 1); // Only global definition visible
}

#[test]
fn test_usage_to_definition_resolution() {
    let mut symbol_table = SymbolTable::new();

    let _func_scope_id = symbol_table.scopes.create_scope(
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

    let definition = Definition::new_simple(
        "test_var".to_string(),
        DefinitionType::VariableDefinition,
        Position {
            start_line: 3,
            start_column: 5,
            end_line: 3,
            end_column: 5,
        },
    );

    symbol_table.add_symbol(
        "test_var".to_string(),
        definition,
        0,
        Accessibility::Public,
        false,
    );

    let resolver = DefaultScopeAwareResolver::new("rust".to_string());

    let usage = Usage::new_simple(
        "test_var".to_string(),
        Position {
            start_line: 7,
            start_column: 10,
            end_line: 7,
            end_column: 10,
        }, // Inside function scope
        UsageKind::Read,
    );

    let found_def = resolver.find_definition_in_scope(&usage, &symbol_table);
    assert!(found_def.is_some());

    let def = found_def.unwrap();
    assert_eq!(def.name, "test_var");
    assert_eq!(def.position.start_line, 3);
}

#[test]
fn test_multiple_definitions_same_name() {
    let mut symbol_table = SymbolTable::new();

    let func1_scope_id = symbol_table.scopes.create_scope(
        Some(0),
        ScopeType::Function,
        Position {
            start_line: 2,
            start_column: 1,
            end_line: 2,
            end_column: 1,
        },
        Position {
            start_line: 6,
            start_column: 1,
            end_line: 6,
            end_column: 1,
        },
    );

    let func2_scope_id = symbol_table.scopes.create_scope(
        Some(0),
        ScopeType::Function,
        Position {
            start_line: 8,
            start_column: 1,
            end_line: 8,
            end_column: 1,
        },
        Position {
            start_line: 12,
            start_column: 1,
            end_line: 12,
            end_column: 1,
        },
    );

    let def1 = Definition::new_simple(
        "local_var".to_string(),
        DefinitionType::VariableDefinition,
        Position {
            start_line: 3,
            start_column: 5,
            end_line: 3,
            end_column: 5,
        },
    );

    let def2 = Definition::new_simple(
        "local_var".to_string(),
        DefinitionType::VariableDefinition,
        Position {
            start_line: 9,
            start_column: 5,
            end_line: 9,
            end_column: 5,
        },
    );

    symbol_table.add_symbol(
        "local_var".to_string(),
        def1,
        func1_scope_id,
        Accessibility::ScopeLocal,
        false,
    );
    symbol_table.add_symbol(
        "local_var".to_string(),
        def2,
        func2_scope_id,
        Accessibility::ScopeLocal,
        false,
    );

    let resolver = DefaultScopeAwareResolver::new("rust".to_string());

    // Usage in first function should resolve to first definition
    let usage1 = Usage::new_simple(
        "local_var".to_string(),
        Position {
            start_line: 4,
            start_column: 5,
            end_line: 4,
            end_column: 5,
        },
        UsageKind::Read,
    );

    let found_def1 = resolver.find_definition_in_scope(&usage1, &symbol_table);
    assert!(found_def1.is_some());
    assert_eq!(found_def1.unwrap().position.start_line, 3);

    // Usage in second function should resolve to second definition
    let usage2 = Usage::new_simple(
        "local_var".to_string(),
        Position {
            start_line: 10,
            start_column: 5,
            end_line: 10,
            end_column: 5,
        },
        UsageKind::Read,
    );

    let found_def2 = resolver.find_definition_in_scope(&usage2, &symbol_table);
    assert!(found_def2.is_some());
    assert_eq!(found_def2.unwrap().position.start_line, 9);
}

#[test]
fn test_dependency_resolution_with_scopes() {
    let mut symbol_table = SymbolTable::new();

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

    let global_def = Definition::new_simple(
        "global_var".to_string(),
        DefinitionType::VariableDefinition,
        Position {
            start_line: 2,
            start_column: 5,
            end_line: 2,
            end_column: 5,
        },
    );

    let local_def = Definition::new_simple(
        "local_var".to_string(),
        DefinitionType::VariableDefinition,
        Position {
            start_line: 6,
            start_column: 5,
            end_line: 6,
            end_column: 5,
        },
    );

    symbol_table.add_symbol(
        "global_var".to_string(),
        global_def,
        0,
        Accessibility::Public,
        false,
    );
    symbol_table.add_symbol(
        "local_var".to_string(),
        local_def,
        func_scope_id,
        Accessibility::ScopeLocal,
        false,
    );

    let usage_nodes = vec![
        Usage::new_simple(
            "global_var".to_string(),
            Position {
                start_line: 7,
                start_column: 10,
                end_line: 7,
                end_column: 10,
            },
            UsageKind::Read,
        ),
        Usage::new_simple(
            "local_var".to_string(),
            Position {
                start_line: 8,
                start_column: 5,
                end_line: 8,
                end_column: 5,
            },
            UsageKind::Read,
        ),
        Usage::new_simple(
            "undefined_var".to_string(),
            Position {
                start_line: 9,
                start_column: 5,
                end_line: 9,
                end_column: 5,
            },
            UsageKind::Read,
        ),
    ];

    let resolver = DefaultScopeAwareResolver::new("rust".to_string());

    // Create dummy root node and source code for the interface
    use tree_sitter::{Language, Parser};

    extern "C" {
        fn tree_sitter_rust() -> Language;
    }

    let mut parser = Parser::new();
    unsafe {
        parser.set_language(&tree_sitter_rust()).unwrap();
    }

    let source = "fn main() {}";
    let tree = parser.parse(source, None).unwrap();

    let dependencies = resolver
        .resolve_dependencies_with_scope(source, tree.root_node(), &usage_nodes, &symbol_table)
        .unwrap();

    // Should resolve global_var and local_var, but not undefined_var
    assert_eq!(dependencies.len(), 2);

    let global_dep = dependencies
        .iter()
        .find(|d| d.symbol == "global_var")
        .unwrap();
    assert_eq!(global_dep.target_line, 2);

    let local_dep = dependencies
        .iter()
        .find(|d| d.symbol == "local_var")
        .unwrap();
    assert_eq!(local_dep.target_line, 6);
}
