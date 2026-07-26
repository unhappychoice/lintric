use lintric_core::models::{
    Accessibility, Definition, DefinitionType, Position, ScopeType, SymbolTable,
};

#[test]
fn a_symbol_lands_in_the_scope_it_was_added_to() {
    let mut table = SymbolTable::new();
    let scope = table
        .scopes
        .create_scope(Some(0), ScopeType::Function, span(2, 9));

    table.add_symbol(
        "value".to_string(),
        definition("value", 3),
        scope,
        Accessibility::ScopeLocal,
        false,
    );

    let held = table.scopes.get_scope(scope).unwrap().get_symbols("value");

    assert_eq!(held.map(Vec::len), Some(1));
}

#[test]
fn adding_a_symbol_records_the_scope_on_the_definition() {
    // The definition carries its own scope afterwards, which is what resolution reads.
    let mut table = SymbolTable::new();
    let scope = table
        .scopes
        .create_scope(Some(0), ScopeType::Function, span(2, 9));

    table.add_symbol(
        "value".to_string(),
        definition("value", 3),
        scope,
        Accessibility::ScopeLocal,
        true,
    );

    let held = &table
        .scopes
        .get_scope(scope)
        .unwrap()
        .get_symbols("value")
        .unwrap()[0];

    assert_eq!(held.get_scope_id(), Some(scope));
    assert_eq!(held.is_hoisted(), Some(true));
}

#[test]
fn an_enhanced_symbol_is_placed_by_the_scope_it_already_carries() {
    let mut table = SymbolTable::new();
    let scope = table
        .scopes
        .create_scope(Some(0), ScopeType::Block, span(4, 6));

    let mut prepared = definition("held", 5);
    prepared.set_context(scope, &Accessibility::ScopeLocal, false);
    table.add_enhanced_symbol("held".to_string(), prepared);

    assert!(table
        .scopes
        .get_scope(scope)
        .unwrap()
        .get_symbols("held")
        .is_some());
}

#[test]
#[should_panic(expected = "context information")]
fn an_enhanced_symbol_with_no_scope_is_refused() {
    // The name says the definition arrives already placed, and there is nowhere to put one that
    // does not. Panicking is the current contract rather than a considered choice; the test records
    // it so changing it is deliberate.
    let mut table = SymbolTable::new();
    table.add_enhanced_symbol("held".to_string(), definition("held", 5));
}

fn definition(name: &str, line: usize) -> Definition {
    Definition::new_simple(
        name.to_string(),
        DefinitionType::VariableDefinition,
        span(line, line),
    )
}

fn span(start_line: usize, end_line: usize) -> Position {
    Position {
        start_line,
        start_column: 1,
        end_line,
        end_column: 1,
    }
}
