use lintric_core::models::DependencyType;
use lintric_core::{analyze_content, Language};

#[test]
fn links_an_implementation_to_the_signature_it_satisfies() {
    let source = "trait Shape {\n    fn area(&self) -> i32;\n}\n\nstruct Square;\n\nimpl Shape for Square {\n    fn area(&self) -> i32 {\n        0\n    }\n}\n";

    assert!(
        trait_implementations(source).contains(&(8, 2, "area".to_string())),
        "{:?}",
        trait_implementations(source)
    );
}

#[test]
fn links_an_override_to_the_default_method_it_replaces() {
    let source = "trait Shape {\n    fn area(&self) -> i32 {\n        0\n    }\n}\n\nstruct Square;\n\nimpl Shape for Square {\n    fn area(&self) -> i32 {\n        1\n    }\n}\n";

    assert!(
        trait_implementations(source).contains(&(10, 2, "area".to_string())),
        "{:?}",
        trait_implementations(source)
    );
}

#[test]
fn links_every_method_of_the_implementation() {
    let source = "trait Shape {\n    fn area(&self) -> i32;\n    fn sides(&self) -> i32;\n}\n\nstruct Square;\n\nimpl Shape for Square {\n    fn area(&self) -> i32 {\n        0\n    }\n\n    fn sides(&self) -> i32 {\n        4\n    }\n}\n";
    let dependencies = trait_implementations(source);

    assert!(
        dependencies.contains(&(9, 2, "area".to_string())),
        "{dependencies:?}"
    );
    assert!(
        dependencies.contains(&(13, 3, "sides".to_string())),
        "{dependencies:?}"
    );
}

#[test]
fn resolves_a_shared_method_name_to_the_trait_being_implemented() {
    let source = "trait Left {\n    fn run(&self);\n}\n\ntrait Right {\n    fn run(&self);\n}\n\nstruct S;\n\nimpl Right for S {\n    fn run(&self) {}\n}\n";
    let dependencies = trait_implementations(source);

    assert_eq!(dependencies, vec![(12, 6, "run".to_string())]);
}

#[test]
fn ignores_an_inherent_impl_block() {
    let source =
        "struct Square;\n\nimpl Square {\n    fn area(&self) -> i32 {\n        0\n    }\n}\n";

    assert!(trait_implementations(source).is_empty());
}

#[test]
fn ignores_a_method_the_trait_does_not_declare() {
    let source = "trait Shape {\n    fn area(&self) -> i32;\n}\n\nstruct Square;\n\nimpl Shape for Square {\n    fn area(&self) -> i32 {\n        0\n    }\n\n    fn extra(&self) -> i32 {\n        0\n    }\n}\n";

    assert_eq!(
        trait_implementations(source),
        vec![(8, 2, "area".to_string())]
    );
}

fn trait_implementations(source: &str) -> Vec<(usize, usize, String)> {
    let (ir, _) = analyze_content(source.to_string(), Language::Rust).unwrap();

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
