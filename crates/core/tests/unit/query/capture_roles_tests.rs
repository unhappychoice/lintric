use lintric_core::query::capture_roles;
use lintric_core::{analyze_content, Language};

/// Roles are compared by identity in these tests, so a plain marker type is enough.
#[derive(Clone, Debug, PartialEq)]
enum Role {
    Type,
    Field,
}

#[test]
fn labels_a_captured_node_with_the_role_its_capture_name_maps_to() {
    let roles = roles_for(
        "struct P {\n    x: i32,\n}\n",
        "(struct_item name: (type_identifier) @kind.type)\n(field_declaration name: (field_identifier) @kind.field)",
        &[("kind.type", Role::Type), ("kind.field", Role::Field)],
    );

    assert_eq!(
        roles,
        vec![
            ("P".to_string(), Role::Type),
            ("x".to_string(), Role::Field)
        ]
    );
}

#[test]
fn ignores_a_capture_the_mapping_does_not_name() {
    let roles = roles_for(
        "struct P {\n    x: i32,\n}\n",
        "(struct_item name: (type_identifier) @kind.type)\n(field_declaration name: (field_identifier) @unmapped)",
        &[("kind.type", Role::Type)],
    );

    assert_eq!(roles, vec![("P".to_string(), Role::Type)]);
}

#[test]
fn labels_every_match_of_a_repeated_pattern() {
    let roles = roles_for(
        "struct A;\nstruct B;\n",
        "(struct_item name: (type_identifier) @kind.type)",
        &[("kind.type", Role::Type)],
    );

    assert_eq!(roles.len(), 2, "{roles:?}");
}

#[test]
fn reports_a_malformed_query_rather_than_silently_matching_nothing() {
    let source = "struct P;\n";
    let (_, _) = analyze_content(source.to_string(), Language::Rust).unwrap();
    let tree = parse(source);

    let error = capture_roles(
        "(no_such_node) @kind.type",
        source,
        tree.root_node(),
        &[("kind.type", Role::Type)],
    )
    .unwrap_err();

    assert!(error.contains("Failed to create query"), "{error}");
}

fn roles_for(source: &str, query: &str, mapping: &[(&str, Role)]) -> Vec<(String, Role)> {
    let tree = parse(source);
    let roles = capture_roles(query, source, tree.root_node(), mapping).unwrap();

    let mut named: Vec<(usize, String, Role)> = collect_named(tree.root_node(), source, &roles);
    named.sort_by_key(|(offset, _, _)| *offset);
    named
        .into_iter()
        .map(|(_, name, role)| (name, role))
        .collect()
}

fn collect_named(
    node: tree_sitter::Node,
    source: &str,
    roles: &std::collections::HashMap<usize, Role>,
) -> Vec<(usize, String, Role)> {
    let mut found = vec![];
    if let Some(role) = roles.get(&node.id()) {
        found.push((
            node.start_byte(),
            node.utf8_text(source.as_bytes()).unwrap().to_string(),
            role.clone(),
        ));
    }

    let mut cursor = node.walk();
    let children: Vec<tree_sitter::Node> = node.children(&mut cursor).collect();
    for child in children {
        found.extend(collect_named(child, source, roles));
    }
    found
}

fn parse(source: &str) -> tree_sitter::Tree {
    let mut parser = tree_sitter::Parser::new();
    parser
        .set_language(&tree_sitter_rust::LANGUAGE.into())
        .unwrap();
    parser.parse(source, None).unwrap()
}
