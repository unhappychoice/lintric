use lintric_core::models::DefinitionType;
use lintric_core::{analyze_content, Language};

#[test]
fn registers_each_variant_as_a_definition() {
    let source = "enum Direction {\n    Left,\n    Right,\n}\n";

    assert_eq!(
        variant_definitions(source),
        vec![(2, "Left".to_string()), (3, "Right".to_string())]
    );
}

#[test]
fn registers_a_variant_carrying_a_tuple_payload() {
    let source = "enum Value {\n    Number(i32),\n}\n";

    assert_eq!(variant_definitions(source), vec![(2, "Number".to_string())]);
}

#[test]
fn registers_a_variant_carrying_named_fields() {
    let source = "enum Shape {\n    Rect { width: i32 },\n}\n";
    let definitions = variant_definitions(source);

    assert_eq!(definitions, vec![(2, "Rect".to_string())]);
}

#[test]
fn does_not_treat_a_variant_declaration_as_a_usage() {
    let source = "enum Direction {\n    Left,\n}\n";
    let (ir, _) = analyze_content(source.to_string(), Language::Rust).unwrap();

    assert!(
        !ir.usage.iter().any(|usage| usage.name == "Left"),
        "{:?}",
        ir.usage.iter().map(|usage| &usage.name).collect::<Vec<_>>()
    );
}

#[test]
fn resolves_a_variant_reference_to_its_declaration() {
    let source = "enum Direction {\n    Left,\n    Right,\n}\n\nfn main() {\n    let d = Direction::Left;\n}\n";

    assert!(
        dependencies(source).contains(&(7, 2, "Left".to_string())),
        "{:?}",
        dependencies(source)
    );
}

#[test]
fn resolves_variant_patterns_in_match_arms() {
    let source = "enum Direction {\n    Left,\n    Right,\n}\n\nfn flip(d: Direction) -> Direction {\n    match d {\n        Direction::Left => Direction::Right,\n        Direction::Right => Direction::Left,\n    }\n}\n";
    let dependencies = dependencies(source);

    assert!(
        dependencies.contains(&(8, 2, "Left".to_string())),
        "{dependencies:?}"
    );
    assert!(
        dependencies.contains(&(8, 3, "Right".to_string())),
        "{dependencies:?}"
    );
    assert!(
        dependencies.contains(&(9, 3, "Right".to_string())),
        "{dependencies:?}"
    );
    assert!(
        dependencies.contains(&(9, 2, "Left".to_string())),
        "{dependencies:?}"
    );
}

fn variant_definitions(source: &str) -> Vec<(usize, String)> {
    let (ir, _) = analyze_content(source.to_string(), Language::Rust).unwrap();

    ir.definitions
        .iter()
        .filter(|definition| definition.definition_type == DefinitionType::EnumVariantDefinition)
        .map(|definition| (definition.position.start_line, definition.name.clone()))
        .collect()
}

fn dependencies(source: &str) -> Vec<(usize, usize, String)> {
    let (ir, _) = analyze_content(source.to_string(), Language::Rust).unwrap();

    ir.dependencies
        .iter()
        .map(|dependency| {
            (
                dependency.source_line,
                dependency.target_line,
                dependency.symbol.clone(),
            )
        })
        .collect()
}
