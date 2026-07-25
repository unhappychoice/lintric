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
