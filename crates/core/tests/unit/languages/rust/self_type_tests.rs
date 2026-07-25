use lintric_core::{analyze_content, Language};

#[test]
fn a_self_return_type_names_the_type_being_implemented() {
    let source = "struct T;\n\nimpl T {\n    fn make() -> Self {\n        T\n    }\n}\n";

    assert!(
        dependencies(source).contains(&(4, 1, "T".to_string())),
        "{:?}",
        dependencies(source)
    );
}

#[test]
fn a_self_struct_literal_names_the_type_being_implemented() {
    let source = "struct T {\n    v: i32,\n}\n\nimpl T {\n    fn make() -> T {\n        Self { v: 1 }\n    }\n}\n";

    assert!(
        dependencies(source).contains(&(7, 1, "T".to_string())),
        "{:?}",
        dependencies(source)
    );
}

#[test]
fn a_self_qualified_path_names_the_type_being_implemented() {
    let source = "struct T;\n\nimpl T {\n    const START: i32 = 0;\n\n    fn begin() -> i32 {\n        Self::START\n    }\n}\n";
    let dependencies = dependencies(source);

    assert!(
        dependencies.contains(&(7, 1, "T".to_string())),
        "{dependencies:?}"
    );
    assert!(
        dependencies.contains(&(7, 4, "START".to_string())),
        "{dependencies:?}"
    );
}

#[test]
fn self_is_the_type_not_the_trait_inside_a_trait_implementation() {
    let source = "trait Make {\n    fn make() -> Self;\n}\n\nstruct T;\n\nimpl Make for T {\n    fn make() -> Self {\n        T\n    }\n}\n";
    let dependencies = dependencies(source);

    assert!(
        dependencies.contains(&(8, 5, "T".to_string())),
        "the implementing type: {dependencies:?}"
    );
    assert!(
        !dependencies.contains(&(8, 1, "Make".to_string())),
        "not the trait: {dependencies:?}"
    );
}

#[test]
fn self_is_the_trait_inside_a_trait_declaration() {
    let source = "trait Make {\n    fn make() -> Self;\n}\n";

    assert!(
        dependencies(source).contains(&(2, 1, "Make".to_string())),
        "{:?}",
        dependencies(source)
    );
}

#[test]
fn a_generic_impl_resolves_self_to_the_bare_type_name() {
    let source = "struct Holder<T> {\n    v: T,\n}\n\nimpl<T> Holder<T> {\n    fn empty() -> Option<Self> {\n        None\n    }\n}\n";

    assert!(
        dependencies(source).contains(&(6, 1, "Holder".to_string())),
        "{:?}",
        dependencies(source)
    );
}

#[test]
fn the_lowercase_receiver_is_left_alone() {
    let source = "struct T {\n    v: i32,\n}\n\nimpl T {\n    fn get(&self) -> i32 {\n        self.v\n    }\n}\n";
    let dependencies = dependencies(source);

    assert!(
        dependencies.contains(&(7, 2, "v".to_string())),
        "the field still resolves: {dependencies:?}"
    );
    assert!(
        !dependencies
            .iter()
            .any(|(from, _, symbol)| *from == 7 && symbol == "T"),
        "`self` is not a type reference: {dependencies:?}"
    );
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

#[test]
fn a_type_parameter_declaration_does_not_depend_on_another_of_the_same_name() {
    // `trait B<T>` declares its own parameter. Left as a usage it resolved to A's, so one
    // declaration depended on another.
    let source =
        "trait A<T> {\n    fn a(&self, v: T);\n}\n\ntrait B<T> {\n    fn b(&self, v: T);\n}\n";
    let dependencies = dependencies(source);

    assert!(
        !dependencies
            .iter()
            .any(|(from, _, symbol)| *from == 5 && symbol == "T"),
        "line 5 declares T and references nothing: {dependencies:?}"
    );
}
