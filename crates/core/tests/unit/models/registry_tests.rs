use lintric_core::models::{
    Definition, DefinitionRegistry, DefinitionType, Position, Usage, UsageKind, UsageRegistry,
};

#[test]
fn groups_definitions_that_share_a_name() {
    // Resolution asks for candidates by name, so two declarations of one name are both kept.
    let mut registry = DefinitionRegistry::new();
    registry.add_definition("value".to_string(), definition("value", 2));
    registry.add_definition("value".to_string(), definition("value", 9));
    registry.add_definition("other".to_string(), definition("other", 4));

    let all = registry.get_all_definitions();

    assert_eq!(all["value"].len(), 2);
    assert_eq!(all["other"].len(), 1);
}

#[test]
fn keeps_the_lines_a_name_was_declared_on() {
    let mut registry = DefinitionRegistry::new();
    registry.add_definition("value".to_string(), definition("value", 2));
    registry.add_definition("value".to_string(), definition("value", 9));

    let lines: Vec<usize> = registry.get_all_definitions()["value"]
        .iter()
        .map(|definition| definition.position.start_line)
        .collect();

    assert_eq!(lines, vec![2, 9]);
}

#[test]
fn a_registry_starts_empty() {
    assert!(DefinitionRegistry::default()
        .get_all_definitions()
        .is_empty());
    assert!(UsageRegistry::default().get_all_usages().is_empty());
}

#[test]
fn keeps_usages_in_the_order_they_were_found() {
    // A later usage of one name may resolve differently from an earlier one, so order is meaning.
    let mut registry = UsageRegistry::new();
    registry.add_usage(usage("value", 3));
    registry.add_usage(usage("value", 7));

    let lines: Vec<usize> = registry
        .get_all_usages()
        .iter()
        .map(|usage| usage.position.start_line)
        .collect();

    assert_eq!(lines, vec![3, 7]);
}

fn definition(name: &str, line: usize) -> Definition {
    Definition::new_simple(
        name.to_string(),
        DefinitionType::VariableDefinition,
        position(line, name),
    )
}

fn position(line: usize, name: &str) -> Position {
    Position {
        start_line: line,
        start_column: 1,
        end_line: line,
        end_column: 1 + name.len(),
    }
}

fn usage(name: &str, line: usize) -> Usage {
    Usage {
        name: name.to_string(),
        kind: UsageKind::Identifier,
        position: position(line, name),
        context: None,
        scope_id: None,
    }
}
