use lintric_core::models::scope::{Accessibility, ScopeType, SymbolTable};
use lintric_core::models::{usage::UsageKind, Definition, DefinitionType, Position, Usage};
use lintric_core::scope_aware_symbol_lookup::ScopeAwareSymbolLookup;
use lintric_core::shadowing_resolver::{
    NameResolutionEngine, ResolutionCandidate, ShadowingResolver,
};

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
fn test_resolution_candidate_priority_calculation() {
    let definition = create_test_definition("test", 5);

    // Closer scope should have higher priority
    let candidate1 = ResolutionCandidate::new(definition.clone(), 0, 0, 1);
    let candidate2 = ResolutionCandidate::new(definition.clone(), 1, 0, 2);

    assert!(candidate1.priority_score > candidate2.priority_score);

    // Lower shadowing level should have higher priority
    let candidate3 = ResolutionCandidate::new(definition.clone(), 0, 0, 1);
    let candidate4 = ResolutionCandidate::new(definition, 0, 1, 1);

    assert!(candidate3.priority_score > candidate4.priority_score);
}

#[test]
fn test_shadowing_resolver_find_visible_definition() {
    let mut symbol_table = SymbolTable::new();

    // Create nested scopes
    let func_scope = symbol_table.scopes.create_scope(
        Some(0),
        ScopeType::Function,
        create_test_position(5, 1),
        create_test_position(20, 1),
    );

    let block_scope = symbol_table.scopes.create_scope(
        Some(func_scope),
        ScopeType::Block,
        create_test_position(10, 1),
        create_test_position(15, 1),
    );

    // Add definitions at different levels
    let global_def = create_test_definition("var", 1);
    let func_def = create_test_definition("var", 7);
    let block_def = create_test_definition("var", 12);

    symbol_table.add_symbol(
        "var".to_string(),
        global_def,
        0,
        Accessibility::ScopeLocal,
        false,
    );
    symbol_table.add_symbol(
        "var".to_string(),
        func_def,
        func_scope,
        Accessibility::ScopeLocal,
        false,
    );
    symbol_table.add_symbol(
        "var".to_string(),
        block_def.clone(),
        block_scope,
        Accessibility::ScopeLocal,
        false,
    );

    let resolver = ShadowingResolver::from_symbol_table(symbol_table);

    // From block scope, should find block definition
    let visible = resolver.find_visible_definition(block_scope, "var");
    assert!(visible.is_some());
    assert_eq!(visible.unwrap().position.start_line, 12);

    // From function scope, should find function definition
    let visible = resolver.find_visible_definition(func_scope, "var");
    assert!(visible.is_some());
    assert_eq!(visible.unwrap().position.start_line, 7);

    // From global scope, should find global definition
    let visible = resolver.find_visible_definition(0, "var");
    assert!(visible.is_some());
    assert_eq!(visible.unwrap().position.start_line, 1);

    // Non-existent variable should return None
    let visible = resolver.find_visible_definition(block_scope, "nonexistent");
    assert!(visible.is_none());
}

#[test]
fn test_shadowing_chain_analysis() {
    let mut symbol_table = SymbolTable::new();

    // Create 3-level nesting
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

    // Add same variable at each level
    let def0 = create_test_definition("chain_var", 1);
    let def1 = create_test_definition("chain_var", 7);
    let def2 = create_test_definition("chain_var", 11);
    let def3 = create_test_definition("chain_var", 13);

    symbol_table.add_symbol(
        "chain_var".to_string(),
        def0,
        0,
        Accessibility::ScopeLocal,
        false,
    );
    symbol_table.add_symbol(
        "chain_var".to_string(),
        def1,
        scope1,
        Accessibility::ScopeLocal,
        false,
    );
    symbol_table.add_symbol(
        "chain_var".to_string(),
        def2,
        scope2,
        Accessibility::ScopeLocal,
        false,
    );
    symbol_table.add_symbol(
        "chain_var".to_string(),
        def3,
        scope3,
        Accessibility::ScopeLocal,
        false,
    );

    let resolver = ShadowingResolver::from_symbol_table(symbol_table);

    // Get shadowing chain from deepest scope
    let chain = resolver.get_shadowing_chain(scope3, "chain_var");
    assert_eq!(chain.len(), 4); // All 4 levels

    // Chain should be in order from current scope to root
    assert_eq!(chain[0].1.position.start_line, 13); // scope3
    assert_eq!(chain[1].1.position.start_line, 11); // scope2
    assert_eq!(chain[2].1.position.start_line, 7); // scope1
    assert_eq!(chain[3].1.position.start_line, 1); // global
}

#[test]
fn test_is_shadowed_detection() {
    let mut symbol_table = SymbolTable::new();

    let func_scope = symbol_table.scopes.create_scope(
        Some(0),
        ScopeType::Function,
        create_test_position(5, 1),
        create_test_position(15, 1),
    );

    let outer_def = create_test_definition("shadowed_var", 1);
    let inner_def = create_test_definition("shadowed_var", 8);

    symbol_table.add_symbol(
        "shadowed_var".to_string(),
        outer_def.clone(),
        0,
        Accessibility::ScopeLocal,
        false,
    );
    symbol_table.add_symbol(
        "shadowed_var".to_string(),
        inner_def,
        func_scope,
        Accessibility::ScopeLocal,
        false,
    );

    let resolver = ShadowingResolver::from_symbol_table(symbol_table);

    // Outer definition should be shadowed when accessed from inner scope
    let is_shadowed = resolver.is_shadowed(&outer_def, func_scope);
    assert!(is_shadowed);

    // Same definition should not be shadowed by itself
    let is_self_shadowed = resolver.is_shadowed(&outer_def, 0);
    assert!(!is_self_shadowed);
}

#[test]
fn test_name_resolution_engine_complex_scenario() {
    let mut symbol_table = SymbolTable::new();

    // Create multiple nested scopes with same variable name
    let func_scope = symbol_table.scopes.create_scope(
        Some(0),
        ScopeType::Function,
        create_test_position(10, 1),
        create_test_position(50, 1),
    );
    let if_scope = symbol_table.scopes.create_scope(
        Some(func_scope),
        ScopeType::Block,
        create_test_position(15, 1),
        create_test_position(25, 1),
    );
    let inner_scope = symbol_table.scopes.create_scope(
        Some(if_scope),
        ScopeType::Block,
        create_test_position(18, 1),
        create_test_position(22, 1),
    );

    let global_def = create_test_definition("complex_var", 5);
    let func_def = create_test_definition("complex_var", 12);
    let if_def = create_test_definition("complex_var", 16);
    let inner_def = create_test_definition("complex_var", 19);

    symbol_table.add_symbol(
        "complex_var".to_string(),
        global_def,
        0,
        Accessibility::ScopeLocal,
        false,
    );
    symbol_table.add_symbol(
        "complex_var".to_string(),
        func_def,
        func_scope,
        Accessibility::ScopeLocal,
        false,
    );
    symbol_table.add_symbol(
        "complex_var".to_string(),
        if_def,
        if_scope,
        Accessibility::ScopeLocal,
        false,
    );
    symbol_table.add_symbol(
        "complex_var".to_string(),
        inner_def.clone(),
        inner_scope,
        Accessibility::ScopeLocal,
        false,
    );

    let engine = NameResolutionEngine::new(symbol_table.scopes.clone(), symbol_table);

    // Usage at innermost scope
    let usage = Usage::new_simple(
        "complex_var".to_string(),
        create_test_position(20, 5),
        UsageKind::Identifier,
    );

    let candidates = engine.resolve_name(&usage);
    assert_eq!(candidates.len(), 4); // All 4 definitions should be candidates

    // Best candidate should be the innermost one
    let best = engine.select_best_candidate(&candidates);
    assert!(best.is_some());
    assert_eq!(best.unwrap().definition.position.start_line, 19);

    // Test that candidates are sorted by priority
    assert!(candidates[0].priority_score >= candidates[1].priority_score);
    assert!(candidates[1].priority_score >= candidates[2].priority_score);
    assert!(candidates[2].priority_score >= candidates[3].priority_score);
}

#[test]
fn test_scope_aware_lookup_edge_cases() {
    let symbol_table = SymbolTable::new();

    let lookup = ScopeAwareSymbolLookup::new(symbol_table.scopes.clone());

    // Test with non-existent scope
    let definitions = lookup.lookup_in_scope_chain(999, "nonexistent");
    assert!(definitions.is_empty());

    // Test scope distance to same scope
    let distance = lookup.calculate_scope_distance(0, 0);
    assert_eq!(distance, Some(0));

    // Test scope distance to non-existent scope
    let distance = lookup.calculate_scope_distance(0, 999);
    assert_eq!(distance, None);

    // Test depth of global scope
    let depth = lookup.get_scope_depth(0);
    assert_eq!(depth, 0);

    // Test common scope of same scope
    let common = lookup.find_closest_common_scope(0, 0);
    assert_eq!(common, Some(0));
}

#[test]
fn test_shadowing_warnings_comprehensive() {
    let mut symbol_table = SymbolTable::new();

    let func_scope = symbol_table.scopes.create_scope(
        Some(0),
        ScopeType::Function,
        create_test_position(5, 1),
        create_test_position(20, 1),
    );

    // Multiple definitions in same scope (should generate warnings)
    let def1 = create_test_definition("multi_def", 8);
    let def2 = create_test_definition("multi_def", 10);

    symbol_table.add_symbol(
        "multi_def".to_string(),
        def1,
        func_scope,
        Accessibility::ScopeLocal,
        false,
    );
    symbol_table.add_symbol(
        "multi_def".to_string(),
        def2,
        func_scope,
        Accessibility::ScopeLocal,
        false,
    );

    // Definition that shadows outer scope
    let outer_def = create_test_definition("outer_shadow", 2);
    let shadow_def = create_test_definition("outer_shadow", 12);

    symbol_table.add_symbol(
        "outer_shadow".to_string(),
        outer_def,
        0,
        Accessibility::ScopeLocal,
        false,
    );
    symbol_table.add_symbol(
        "outer_shadow".to_string(),
        shadow_def,
        func_scope,
        Accessibility::ScopeLocal,
        false,
    );

    let resolver = ShadowingResolver::from_symbol_table(symbol_table);
    let warnings = resolver.check_shadowing_conflicts(func_scope);

    // Should have warnings for both scenarios
    assert!(warnings.len() >= 2);

    // Check warning messages
    let has_multi_def_warning = warnings.iter().any(|w| w.message.contains("multi_def"));
    let has_shadow_warning = warnings.iter().any(|w| w.message.contains("outer_shadow"));

    assert!(has_multi_def_warning);
    assert!(has_shadow_warning);
}
