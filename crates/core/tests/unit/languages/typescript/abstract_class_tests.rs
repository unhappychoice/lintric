use lintric_core::models::DefinitionType;
use lintric_core::{analyze_content, Language};

#[test]
fn registers_an_abstract_class_as_a_class() {
    let source = "abstract class Shape {\n    abstract area(): number;\n}\n";

    assert_eq!(
        definition_type(source, "Shape"),
        Some(DefinitionType::ClassDefinition)
    );
}

#[test]
fn registers_an_abstract_method_as_a_declaration() {
    let source = "abstract class Shape {\n    abstract area(): number;\n}\n";

    assert_eq!(
        definition_type(source, "area"),
        Some(DefinitionType::MethodDefinition)
    );
}

#[test]
fn resolves_extending_an_abstract_class() {
    let source = "abstract class Shape {\n    abstract area(): number;\n}\n\nclass Square extends Shape {\n    area(): number {\n        return 1;\n    }\n}\n";

    assert!(
        dependencies(source).contains(&(5, 1, "Shape".to_string())),
        "{:?}",
        dependencies(source)
    );
}

#[test]
fn a_concrete_method_of_an_abstract_class_calls_the_declaration() {
    let source = "abstract class Shape {\n    abstract area(): number;\n\n    describe(): string {\n        return String(this.area());\n    }\n}\n\nclass Square extends Shape {\n    area(): number {\n        return 1;\n    }\n}\n";
    let dependencies = dependencies(source);

    assert!(
        dependencies.contains(&(5, 2, "area".to_string())),
        "the declaration, not a subclass: {dependencies:?}"
    );
    assert!(
        !dependencies.contains(&(5, 10, "area".to_string())),
        "{dependencies:?}"
    );
}

#[test]
fn an_abstract_property_is_still_a_property() {
    let source = "abstract class Shape {\n    abstract size: number;\n\n    describe(): number {\n        return this.size;\n    }\n}\n";

    assert!(
        dependencies(source).contains(&(5, 2, "size".to_string())),
        "{:?}",
        dependencies(source)
    );
}

fn definition_type(source: &str, name: &str) -> Option<DefinitionType> {
    let (ir, _) = analyze_content(source.to_string(), Language::TypeScript).unwrap();

    ir.definitions
        .iter()
        .find(|definition| definition.name == name)
        .map(|definition| definition.definition_type.clone())
}

fn dependencies(source: &str) -> Vec<(usize, usize, String)> {
    let (ir, _) = analyze_content(source.to_string(), Language::TypeScript).unwrap();

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
