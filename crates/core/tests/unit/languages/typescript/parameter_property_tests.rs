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
