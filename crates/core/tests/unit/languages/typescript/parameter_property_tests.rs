use lintric_core::models::{DefinitionType, DependencyType};
use lintric_core::{analyze_content, Language};

#[test]
fn registers_a_parameter_with_an_accessibility_modifier_as_a_property() {
    let source = "class C {\n    constructor(public value: number) {}\n}\n";

    assert_eq!(
        definition_type(source, "value"),
        Some(DefinitionType::PropertyDefinition)
    );
}

#[test]
fn registers_a_readonly_parameter_as_a_property() {
    // `readonly` is an anonymous token, unlike the accessibility modifiers.
    let source = "class C {\n    constructor(readonly value: number) {}\n}\n";

    assert_eq!(
        definition_type(source, "value"),
        Some(DefinitionType::PropertyDefinition)
    );
}

#[test]
fn registers_a_parameter_with_combined_modifiers_as_a_property() {
    let source = "class C {\n    constructor(private readonly value: number) {}\n}\n";

    assert_eq!(
        definition_type(source, "value"),
        Some(DefinitionType::PropertyDefinition)
    );
}

#[test]
fn leaves_an_unmodified_constructor_parameter_a_variable() {
    let source = "class C {\n    constructor(value: number) {}\n}\n";

    assert_eq!(
        definition_type(source, "value"),
        Some(DefinitionType::VariableDefinition)
    );
}

#[test]
fn leaves_a_plain_function_parameter_a_variable() {
    let source = "function f(value: number): number {\n    return value;\n}\n";

    assert_eq!(
        definition_type(source, "value"),
        Some(DefinitionType::VariableDefinition)
    );
}

#[test]
fn leaves_a_method_parameter_a_variable() {
    // Only a constructor can declare properties through its parameters.
    let source = "class C {\n    set(value: number) {}\n}\n";

    assert_eq!(
        definition_type(source, "value"),
        Some(DefinitionType::VariableDefinition)
    );
}

#[test]
fn classifies_reading_a_parameter_property_as_a_field_access() {
    let source = "class C {\n    constructor(public value: number) {}\n    get(): number {\n        return this.value;\n    }\n}\n";
    let (ir, _) = analyze_content(source.to_string(), Language::TypeScript).unwrap();

    let dependency = ir
        .dependencies
        .iter()
        .find(|dependency| dependency.source_line == 4 && dependency.target_line == 2)
        .expect("this.value should depend on the parameter property");

    assert_eq!(
        dependency.dependency_type,
        DependencyType::StructFieldAccess
    );
}

fn definition_type(source: &str, name: &str) -> Option<DefinitionType> {
    let (ir, _) = analyze_content(source.to_string(), Language::TypeScript).unwrap();

    ir.definitions
        .iter()
        .find(|definition| definition.name == name)
        .map(|definition| definition.definition_type.clone())
}

#[test]
fn resolves_a_private_field_reference() {
    let source =
        "class C {\n    #n = 0;\n\n    get(): number {\n        return this.#n;\n    }\n}\n";
    let (ir, _) = analyze_content(source.to_string(), Language::TypeScript).unwrap();

    let dependency = ir
        .dependencies
        .iter()
        .find(|dependency| dependency.source_line == 5)
        .expect("this.#n should depend on its declaration");

    assert_eq!(dependency.target_line, 2);
    assert_eq!(dependency.symbol, "#n");
    assert_eq!(
        dependency.dependency_type,
        DependencyType::StructFieldAccess
    );
}

#[test]
fn resolves_a_private_field_written_to() {
    let source = "class C {\n    #n = 0;\n\n    set(v: number) {\n        this.#n = v;\n    }\n}\n";
    let (ir, _) = analyze_content(source.to_string(), Language::TypeScript).unwrap();

    assert!(
        ir.dependencies
            .iter()
            .any(|d| d.source_line == 5 && d.target_line == 2 && d.symbol == "#n"),
        "{:?}",
        ir.dependencies
            .iter()
            .map(|d| (d.source_line, d.target_line, &d.symbol))
            .collect::<Vec<_>>()
    );
}

#[test]
fn resolves_a_private_method_call() {
    let source = "class C {\n    #helper(): number {\n        return 1;\n    }\n\n    run(): number {\n        return this.#helper();\n    }\n}\n";
    let (ir, _) = analyze_content(source.to_string(), Language::TypeScript).unwrap();

    assert!(
        ir.dependencies
            .iter()
            .any(|d| d.source_line == 7 && d.target_line == 2 && d.symbol == "#helper"),
        "{:?}",
        ir.dependencies
            .iter()
            .map(|d| (d.source_line, d.target_line, &d.symbol))
            .collect::<Vec<_>>()
    );
}

#[test]
fn a_private_declaration_is_not_a_usage() {
    let source = "class C {\n    #n = 0;\n}\n";
    let (ir, _) = analyze_content(source.to_string(), Language::TypeScript).unwrap();

    assert!(
        !ir.usage.iter().any(|usage| usage.name == "#n"),
        "{:?}",
        ir.usage.iter().map(|u| &u.name).collect::<Vec<_>>()
    );
}
