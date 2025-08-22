use lintric_core::models::scope::{Accessibility, ScopeType, SymbolTable};
use lintric_core::models::{usage::UsageKind, Definition, DefinitionType, Position, Usage};
use lintric_core::scope_aware_symbol_lookup::ScopeAwareSymbolLookup;
use lintric_core::shadowing_resolver::{NameResolutionEngine, ShadowingResolver};

fn create_test_position(line: usize, column: usize) -> Position {
    Position {
        start_line: line,
        start_column: column,
        end_line: line,
        end_column: column + 1,
    }
}

fn create_test_definition(name: &str, line: usize) -> Definition {
    Definition {
        name: name.to_string(),
        definition_type: DefinitionType::VariableDefinition,
        position: create_test_position(line, 1),
    }
}

#[test]
fn test_basic_shadowing_resolution() {
    let mut symbol_table = SymbolTable::new();

    // Create function scope
    let func_scope_id = symbol_table.scopes.create_scope(
        Some(0),
        ScopeType::Function,
        create_test_position(5, 1),
        create_test_position(15, 1),
    );

    // Create block scope inside function
    let block_scope_id = symbol_table.scopes.create_scope(
        Some(func_scope_id),
        ScopeType::Block,
        create_test_position(8, 1),
        create_test_position(12, 1),
    );

    // Add outer x at global level
    let outer_x = create_test_definition("x", 1);
    symbol_table.add_symbol(
        "x".to_string(),
        outer_x.clone(),
        0,
        Accessibility::ScopeLocal,
        false,
    );

    // Add inner x at block level
    let inner_x = create_test_definition("x", 9);
    symbol_table.add_symbol(
        "x".to_string(),
        inner_x.clone(),
        block_scope_id,
        Accessibility::ScopeLocal,
        false,
    );

    let resolver = ShadowingResolver::from_symbol_table(symbol_table);

    // Usage inside block should resolve to inner x
    let usage_inside_block = Usage::new_simple(
        "x".to_string(),
        create_test_position(10, 5),
        UsageKind::Identifier,
    );

    let resolved = resolver.resolve_shadowed_symbol(&usage_inside_block);
    assert!(resolved.is_some());
    let resolved = resolved.unwrap();
    assert_eq!(resolved.name, "x");
    assert_eq!(resolved.position.start_line, 9); // Should be inner x

    // Usage outside block but inside function should resolve to outer x
    let usage_outside_block = Usage::new_simple(
        "x".to_string(),
        create_test_position(14, 5),
        UsageKind::Identifier,
    );

    let resolved = resolver.resolve_shadowed_symbol(&usage_outside_block);
    assert!(resolved.is_some());
    let resolved = resolved.unwrap();
    assert_eq!(resolved.name, "x");
    assert_eq!(resolved.position.start_line, 1); // Should be outer x
}

#[test]
fn test_complex_shadowing_levels() {
    let mut symbol_table = SymbolTable::new();

    // Create nested scopes (4 levels deep)
    let scope1 = symbol_table.scopes.create_scope(
        Some(0),
        ScopeType::Function,
        create_test_position(5, 1),
        create_test_position(25, 1),
    );

    let scope2 = symbol_table.scopes.create_scope(
        Some(scope1),
        ScopeType::Block,
        create_test_position(10, 1),
        create_test_position(20, 1),
    );

    let scope3 = symbol_table.scopes.create_scope(
        Some(scope2),
        ScopeType::Block,
        create_test_position(12, 1),
        create_test_position(18, 1),
    );

    let scope4 = symbol_table.scopes.create_scope(
        Some(scope3),
        ScopeType::Block,
        create_test_position(14, 1),
        create_test_position(16, 1),
    );

    // Add variable at each level
    let def_level0 = create_test_definition("deep_var", 1);
    let def_level1 = create_test_definition("deep_var", 6);
    let def_level2 = create_test_definition("deep_var", 11);
    let def_level3 = create_test_definition("deep_var", 13);
    let def_level4 = create_test_definition("deep_var", 15);

    symbol_table.add_symbol(
        "deep_var".to_string(),
        def_level0,
        0,
        Accessibility::ScopeLocal,
        false,
    );
    symbol_table.add_symbol(
        "deep_var".to_string(),
        def_level1,
        scope1,
        Accessibility::ScopeLocal,
        false,
    );
    symbol_table.add_symbol(
        "deep_var".to_string(),
        def_level2,
        scope2,
        Accessibility::ScopeLocal,
        false,
    );
    symbol_table.add_symbol(
        "deep_var".to_string(),
        def_level3,
        scope3,
        Accessibility::ScopeLocal,
        false,
    );
    symbol_table.add_symbol(
        "deep_var".to_string(),
        def_level4,
        scope4,
        Accessibility::ScopeLocal,
        false,
    );

    let resolver = ShadowingResolver::from_symbol_table(symbol_table);

    // Usage at deepest level should resolve to deepest definition
    let usage_deepest = Usage::new_simple(
        "deep_var".to_string(),
        create_test_position(15, 5),
        UsageKind::Identifier,
    );

    let resolved = resolver.resolve_shadowed_symbol(&usage_deepest);
    assert!(resolved.is_some());
    assert_eq!(resolved.unwrap().position.start_line, 15);

    // Check shadowing chain
    let chain = resolver.get_shadowing_chain(scope4, "deep_var");
    assert_eq!(chain.len(), 5); // All 5 levels should be in the chain
}

#[test]
fn test_shadowing_warnings() {
    let mut symbol_table = SymbolTable::new();

    let func_scope_id = symbol_table.scopes.create_scope(
        Some(0),
        ScopeType::Function,
        create_test_position(5, 1),
        create_test_position(15, 1),
    );

    // Add variables that shadow each other
    let outer_var = create_test_definition("warning_var", 1);
    let inner_var = create_test_definition("warning_var", 8);

    symbol_table.add_symbol(
        "warning_var".to_string(),
        outer_var,
        0,
        Accessibility::ScopeLocal,
        false,
    );

    symbol_table.add_symbol(
        "warning_var".to_string(),
        inner_var,
        func_scope_id,
        Accessibility::ScopeLocal,
        false,
    );

    let resolver = ShadowingResolver::from_symbol_table(symbol_table);
    let warnings = resolver.check_shadowing_conflicts(func_scope_id);

    assert!(!warnings.is_empty());
    assert!(warnings[0].message.contains("shadows outer scope"));
    assert!(warnings[0].message.contains("warning_var"));
}

#[test]
fn test_name_resolution_engine_priority() {
    let mut symbol_table = SymbolTable::new();

    let func_scope_id = symbol_table.scopes.create_scope(
        Some(0),
        ScopeType::Function,
        create_test_position(5, 1),
        create_test_position(15, 1),
    );

    let outer_def = create_test_definition("priority_var", 1);
    let inner_def = create_test_definition("priority_var", 8);

    symbol_table.add_symbol(
        "priority_var".to_string(),
        outer_def,
        0,
        Accessibility::ScopeLocal,
        false,
    );

    symbol_table.add_symbol(
        "priority_var".to_string(),
        inner_def.clone(),
        func_scope_id,
        Accessibility::ScopeLocal,
        false,
    );

    let engine = NameResolutionEngine::new(symbol_table.scopes.clone(), symbol_table);

    let usage = Usage::new_simple(
        "priority_var".to_string(),
        create_test_position(10, 5),
        UsageKind::Identifier,
    );

    let candidates = engine.resolve_name(&usage);
    assert!(!candidates.is_empty());

    // Best candidate should be the inner definition due to priority
    let best = engine.select_best_candidate(&candidates);
    assert!(best.is_some());
    assert_eq!(best.unwrap().definition.position.start_line, 8);
}

#[test]
fn test_scope_aware_symbol_lookup() {
    let mut symbol_table = SymbolTable::new();

    let func_scope_id = symbol_table.scopes.create_scope(
        Some(0),
        ScopeType::Function,
        create_test_position(5, 1),
        create_test_position(15, 1),
    );

    let block_scope_id = symbol_table.scopes.create_scope(
        Some(func_scope_id),
        ScopeType::Block,
        create_test_position(8, 1),
        create_test_position(12, 1),
    );

    // Add symbols to scopes
    let global_def = create_test_definition("lookup_var", 1);
    let func_def = create_test_definition("lookup_var", 6);
    let block_def = create_test_definition("lookup_var", 9);

    if let Some(global_scope) = symbol_table.scopes.get_scope_mut(0) {
        global_scope.add_symbol("lookup_var".to_string(), global_def);
    }

    if let Some(func_scope) = symbol_table.scopes.get_scope_mut(func_scope_id) {
        func_scope.add_symbol("lookup_var".to_string(), func_def);
    }

    if let Some(block_scope) = symbol_table.scopes.get_scope_mut(block_scope_id) {
        block_scope.add_symbol("lookup_var".to_string(), block_def);
    }

    let lookup = ScopeAwareSymbolLookup::new(symbol_table.scopes.clone());

    // Test lookup in scope chain
    let definitions = lookup.lookup_in_scope_chain(block_scope_id, "lookup_var");
    assert_eq!(definitions.len(), 3);

    // Test scope distance calculation
    let distance = lookup.calculate_scope_distance(block_scope_id, 0);
    assert_eq!(distance, Some(2));

    // Test scope depth
    let depth = lookup.get_scope_depth(block_scope_id);
    assert_eq!(depth, 2);

    // Test visible definitions at scope
    let visible = lookup.get_visible_definitions_at_scope(block_scope_id, "lookup_var");
    assert_eq!(visible.len(), 1);
    assert_eq!(visible[0].position.start_line, 9); // Should be the block definition
}

#[test]
fn test_parameter_shadowing() {
    let mut symbol_table = SymbolTable::new();

    let func_scope_id = symbol_table.scopes.create_scope(
        Some(0),
        ScopeType::Function,
        create_test_position(5, 1),
        create_test_position(15, 1),
    );

    let block_scope_id = symbol_table.scopes.create_scope(
        Some(func_scope_id),
        ScopeType::Block,
        create_test_position(8, 1),
        create_test_position(12, 1),
    );

    // Function parameter
    let param_def = Definition {
        name: "param".to_string(),
        definition_type: DefinitionType::VariableDefinition,
        position: create_test_position(5, 15), // parameter position
    };

    // Local variable shadowing parameter
    let local_def = create_test_definition("param", 7);

    // Block variable shadowing local variable
    let block_def = create_test_definition("param", 9);

    symbol_table.add_symbol(
        "param".to_string(),
        param_def,
        func_scope_id,
        Accessibility::ScopeLocal,
        false,
    );
    symbol_table.add_symbol(
        "param".to_string(),
        local_def,
        func_scope_id,
        Accessibility::ScopeLocal,
        false,
    );
    symbol_table.add_symbol(
        "param".to_string(),
        block_def.clone(),
        block_scope_id,
        Accessibility::ScopeLocal,
        false,
    );

    let resolver = ShadowingResolver::from_symbol_table(symbol_table);

    // Usage in block should resolve to block definition
    let usage_in_block = Usage::new_simple(
        "param".to_string(),
        create_test_position(10, 5),
        UsageKind::Identifier,
    );

    let resolved = resolver.resolve_shadowed_symbol(&usage_in_block);
    assert!(resolved.is_some());
    assert_eq!(resolved.unwrap().position.start_line, 9);

    // Check that there are shadowing warnings
    let warnings = resolver.check_shadowing_conflicts(block_scope_id);
    assert!(!warnings.is_empty());
}
