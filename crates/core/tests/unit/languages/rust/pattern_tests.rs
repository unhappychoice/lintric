use lintric_core::models::DefinitionType;
use lintric_core::{analyze_content, Language};

const POINT: &str = "struct P {\n    x: i32,\n}\n";

#[test]
fn a_struct_pattern_registers_its_bindings() {
    let source = format!("{POINT}\nfn main() {{\n    let p = P {{ x: 1 }};\n    let P {{ x }} = p;\n    let y = x;\n}}\n");

    assert!(
        bindings(&source).contains(&(7, "x".to_string())),
        "{:?}",
        bindings(&source)
    );
}

#[test]
fn a_struct_pattern_references_the_field_it_names() {
    let source =
        format!("{POINT}\nfn main() {{\n    let p = P {{ x: 1 }};\n    let P {{ x }} = p;\n}}\n");

    assert!(
        dependencies(&source).contains(&(7, 2, "x".to_string())),
        "{:?}",
        dependencies(&source)
    );
}

#[test]
fn a_later_use_reads_the_pattern_binding_not_the_field() {
    let source = format!("{POINT}\nfn main() {{\n    let p = P {{ x: 1 }};\n    let P {{ x }} = p;\n    let y = x;\n}}\n");
    let dependencies = dependencies(&source);

    assert!(
        dependencies.contains(&(8, 7, "x".to_string())),
        "should read the binding: {dependencies:?}"
    );
    assert!(
        !dependencies.contains(&(8, 2, "x".to_string())),
        "should not read the field: {dependencies:?}"
    );
}

#[test]
fn a_renamed_field_pattern_separates_the_reference_from_the_binding() {
    let source = format!("{POINT}\nfn main() {{\n    let p = P {{ x: 1 }};\n    let P {{ x: renamed }} = p;\n    let y = renamed;\n}}\n");
    let dependencies = dependencies(&source);

    assert!(
        dependencies.contains(&(7, 2, "x".to_string())),
        "the field is referenced: {dependencies:?}"
    );
    assert!(
        dependencies.contains(&(8, 7, "renamed".to_string())),
        "the binding is read: {dependencies:?}"
    );
}

#[test]
fn a_tuple_struct_pattern_references_the_type_it_names() {
    let source = "struct Meters(i32);\n\nfn main() {\n    let m = Meters(1);\n    let Meters(value) = m;\n    let _ = value;\n}\n";
    let dependencies = dependencies(source);

    assert!(
        dependencies.contains(&(5, 1, "Meters".to_string())),
        "{dependencies:?}"
    );
    assert!(
        dependencies.contains(&(6, 5, "value".to_string())),
        "{dependencies:?}"
    );
}

#[test]
fn a_let_pattern_does_not_register_an_enum_path_as_a_binding() {
    let source = "enum S {\n    F(i32),\n}\n\nfn main() {\n    let s = S::F(1);\n    let S::F(v) = s else {\n        return;\n    };\n    let _ = v;\n}\n";
    let names: Vec<String> = bindings(source)
        .into_iter()
        .filter(|(line, _)| *line == 7)
        .map(|(_, name)| name)
        .collect();

    assert_eq!(names, vec!["v".to_string()]);
}

#[test]
fn a_let_pattern_references_the_enum_and_variant_it_names() {
    let source = "enum S {\n    F(i32),\n}\n\nfn main() {\n    let s = S::F(1);\n    let S::F(v) = s else {\n        return;\n    };\n    let _ = v;\n}\n";
    let dependencies = dependencies(source);

    assert!(
        dependencies.contains(&(7, 1, "S".to_string())),
        "{dependencies:?}"
    );
    assert!(
        dependencies.contains(&(7, 2, "F".to_string())),
        "{dependencies:?}"
    );
}

#[test]
fn an_earlier_reference_is_not_captured_by_a_later_let_pattern() {
    let source = "enum S {\n    F(i32),\n    E,\n}\n\nfn main() {\n    let s = S::E;\n    if let S::E = s {\n        return;\n    }\n    let S::F(v) = s else {\n        return;\n    };\n    let _ = v;\n}\n";
    let dependencies = dependencies(source);

    assert!(
        dependencies.contains(&(8, 1, "S".to_string())),
        "the enum, not a binding on line 11: {dependencies:?}"
    );
    assert!(
        !dependencies.contains(&(8, 11, "S".to_string())),
        "{dependencies:?}"
    );
}

#[test]
fn a_tuple_pattern_still_binds_each_element() {
    let source = "fn main() {\n    let pair = (1, 2);\n    let (first, second) = pair;\n    let _ = first + second;\n}\n";
    let dependencies = dependencies(source);

    assert!(
        dependencies.contains(&(4, 3, "first".to_string())),
        "{dependencies:?}"
    );
    assert!(
        dependencies.contains(&(4, 3, "second".to_string())),
        "{dependencies:?}"
    );
}

fn bindings(source: &str) -> Vec<(usize, String)> {
    let (ir, _) = analyze_content(source.to_string(), Language::Rust).unwrap();

    ir.definitions
        .iter()
        .filter(|definition| definition.definition_type == DefinitionType::VariableDefinition)
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
