use lintric_core::models::DependencyType;
use lintric_core::{analyze_content, Language};

#[test]
fn links_a_class_method_to_the_interface_method_it_implements() {
    let source = "interface Shape {\n    area(): number;\n}\n\nclass Square implements Shape {\n    area(): number {\n        return 1;\n    }\n}\n";

    assert_eq!(
        implementations(source, Language::TypeScript),
        vec![(6, 2, "area".to_string())]
    );
}

#[test]
fn links_an_optional_member_like_any_other() {
    let source = "interface Shape {\n    area?(): number;\n}\n\nclass Square implements Shape {\n    area(): number {\n        return 1;\n    }\n}\n";

    assert_eq!(
        implementations(source, Language::TypeScript),
        vec![(6, 2, "area".to_string())]
    );
}

#[test]
fn resolves_a_shared_method_name_to_the_interface_being_implemented() {
    let source = "interface Left {\n    run(): void;\n}\n\ninterface Right {\n    run(): void;\n}\n\nclass S implements Right {\n    run(): void {}\n}\n";

    assert_eq!(
        implementations(source, Language::TypeScript),
        vec![(10, 6, "run".to_string())]
    );
}

#[test]
fn links_each_method_to_the_interface_that_declares_it() {
    let source = "interface A {\n    one(): void;\n}\n\ninterface B {\n    two(): void;\n}\n\nclass S implements A, B {\n    one(): void {}\n\n    two(): void {}\n}\n";
    let dependencies = implementations(source, Language::TypeScript);

    assert!(
        dependencies.contains(&(10, 2, "one".to_string())),
        "{dependencies:?}"
    );
    assert!(
        dependencies.contains(&(12, 6, "two".to_string())),
        "{dependencies:?}"
    );
}

#[test]
fn ignores_a_class_that_implements_nothing() {
    let source = "class Square {\n    area(): number {\n        return 1;\n    }\n}\n";

    assert!(implementations(source, Language::TypeScript).is_empty());
}

#[test]
fn ignores_a_method_the_interface_does_not_declare() {
    let source = "interface Shape {\n    area(): number;\n}\n\nclass Square implements Shape {\n    area(): number {\n        return 1;\n    }\n\n    extra(): number {\n        return 2;\n    }\n}\n";

    assert_eq!(
        implementations(source, Language::TypeScript),
        vec![(6, 2, "area".to_string())]
    );
}

#[test]
fn works_in_tsx_as_well_as_typescript() {
    // The language comes from the parsed tree, so a dialect needs no separate entry point.
    let source = "interface I {\n    render(): JSX.Element;\n}\n\nclass C implements I {\n    render(): JSX.Element {\n        return <span />;\n    }\n}\n";

    assert_eq!(
        implementations(source, Language::TSX),
        vec![(6, 2, "render".to_string())]
    );
}

fn implementations(source: &str, language: Language) -> Vec<(usize, usize, String)> {
    let (ir, _) = analyze_content(source.to_string(), language).unwrap();

    ir.dependencies
        .iter()
        .filter(|dependency| dependency.dependency_type == DependencyType::TraitImplementation)
        .map(|dependency| {
            (
                dependency.source_line,
                dependency.target_line,
                dependency.symbol.clone(),
            )
        })
        .collect()
}

#[test]
fn links_a_class_method_to_a_declaration_inherited_from_a_parent_interface() {
    let source = "interface Base {\n    run(): void;\n}\n\ninterface Extended extends Base {}\n\nclass S implements Extended {\n    run(): void {}\n}\n";

    assert_eq!(
        implementations(source, Language::TypeScript),
        vec![(8, 2, "run".to_string())]
    );
}

#[test]
fn links_an_override_to_the_base_class_method() {
    let source = "class Base {\n    run(): void {}\n}\n\nclass Derived extends Base {\n    run(): void {}\n}\n";

    assert_eq!(
        implementations(source, Language::TypeScript),
        vec![(6, 2, "run".to_string())]
    );
}

#[test]
fn reaches_an_interface_declaration_through_a_base_class() {
    let source = "interface Shape {\n    area(): number;\n}\n\nclass Base implements Shape {\n    area(): number {\n        return 1;\n    }\n}\n\nclass Derived extends Base {\n    area(): number {\n        return 2;\n    }\n}\n";
    let dependencies = implementations(source, Language::TypeScript);

    assert!(
        dependencies.contains(&(12, 6, "area".to_string())),
        "the nearest declaration is the base class method: {dependencies:?}"
    );
}

#[test]
fn links_an_implementation_to_an_abstract_method_declaration() {
    let source = "abstract class Shape {\n    abstract area(): number;\n}\n\nclass Square extends Shape {\n    area(): number {\n        return 1;\n    }\n}\n";

    assert_eq!(
        implementations(source, Language::TypeScript),
        vec![(6, 2, "area".to_string())]
    );
}

#[test]
fn an_abstract_declaration_does_not_depend_on_its_implementation() {
    let source = "abstract class Shape {\n    abstract area(): number;\n}\n\nclass Square extends Shape {\n    area(): number {\n        return 1;\n    }\n}\n";
    let (ir, _) = analyze_content(source.to_string(), Language::TypeScript).unwrap();

    assert!(
        !ir.dependencies
            .iter()
            .any(|dependency| dependency.source_line == 2 && dependency.target_line == 6),
        "{:?}",
        ir.dependencies
            .iter()
            .map(|d| (d.source_line, d.target_line, &d.symbol))
            .collect::<Vec<_>>()
    );
}
