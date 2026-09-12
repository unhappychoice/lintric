use lintric_core::{analyze_content, Language};

#[test]
fn resolves_both_arguments_of_a_two_argument_generic() {
    let source = "struct A;\nstruct B;\n\nfn f() -> Result<A, B> {\n    todo!()\n}\n";
    let dependencies = dependencies(source);

    assert!(
        dependencies.contains(&(4, 1, "A".to_string())),
        "{dependencies:?}"
    );
    assert!(
        dependencies.contains(&(4, 2, "B".to_string())),
        "{dependencies:?}"
    );
}

#[test]
fn resolves_both_of_two_qualified_paths_on_one_line() {
    let source = "const V: i32 = 1;\nconst W: i32 = 2;\n\nmod m {\n    pub fn f() -> i32 {\n        crate::V + crate::W\n    }\n}\n";
    let dependencies = dependencies(source);

    assert!(
        dependencies.contains(&(6, 1, "V".to_string())),
        "{dependencies:?}"
    );
    assert!(
        dependencies.contains(&(6, 2, "W".to_string())),
        "{dependencies:?}"
    );
}

#[test]
fn resolves_every_bound_of_a_type_parameter() {
    let source = "trait One {}\ntrait Two {}\n\nfn f<T: One + Two>(t: T) -> i32 {\n    0\n}\n";
    let dependencies = dependencies(source);

    assert!(
        dependencies.contains(&(4, 1, "One".to_string())),
        "{dependencies:?}"
    );
    assert!(
        dependencies.contains(&(4, 2, "Two".to_string())),
        "{dependencies:?}"
    );
}

#[test]
fn resolves_every_name_of_an_import_group() {
    let source = "mod m {\n    pub fn one() {}\n    pub fn two() {}\n}\n\nuse m::{one, two};\n";
    let dependencies = dependencies(source);

    assert!(
        dependencies.contains(&(6, 2, "one".to_string())),
        "{dependencies:?}"
    );
    assert!(
        dependencies.contains(&(6, 3, "two".to_string())),
        "{dependencies:?}"
    );
}

#[test]
fn resolves_the_type_a_path_starts_from() {
    // `Holder` in `Holder::make()` is the type the associated function belongs to, not a qualifier
    // to be discarded.
    let source = "struct Holder;\n\nimpl Holder {\n    fn make() -> i32 {\n        1\n    }\n}\n\nfn main() {\n    let _ = Holder::make();\n}\n";
    let dependencies = dependencies(source);

    assert!(
        dependencies.contains(&(10, 1, "Holder".to_string())),
        "{dependencies:?}"
    );
}

#[test]
fn does_not_confuse_two_calls_on_one_line() {
    let source = "mod a {\n    pub fn one() -> i32 {\n        1\n    }\n}\n\nmod b {\n    pub fn two() -> i32 {\n        2\n    }\n}\n\nfn main() {\n    let _ = a::one() + b::two();\n}\n";
    let dependencies = dependencies(source);

    assert!(
        dependencies.contains(&(14, 2, "one".to_string())),
        "{dependencies:?}"
    );
    assert!(
        dependencies.contains(&(14, 8, "two".to_string())),
        "{dependencies:?}"
    );
}

#[test]
fn resolves_enum_variants_through_type_aliases() {
    [
        ("V", "V"),
        ("V(i32)", "V(1)"),
        ("V { value: i32 }", "V { value: 1 }"),
    ]
    .into_iter()
    .for_each(|(variant, value)| {
        let source = format!(
            "enum E {{ {variant} }}\ntype Alias = E;\nfn run() {{\n let _ = Alias::{value};\n}}\n"
        );
        let (ir, _) = analyze_content(source, Language::Rust).unwrap();
        let variants: Vec<_> = ir
            .dependencies
            .iter()
            .filter(|dependency| dependency.source_line == 4 && dependency.symbol == "V")
            .map(|dependency| (dependency.target_line, dependency.dependency_type.clone()))
            .collect();
        assert_eq!(
            variants,
            vec![(
                1,
                lintric_core::models::DependencyType::EnumVariantReference
            )]
        );
    });
}

#[test]
fn resolves_an_associated_function_reference_regardless_of_distance() {
    [0, 30].into_iter().for_each(|padding| {
        let source = format!("struct T;\ntype Alias = T;\n{}impl T {{ fn make() {{}} }}\nfn run() {{\n let _ = Alias::make;\n}}\n", "\n".repeat(padding));
        assert!(dependencies(&source).contains(&(padding + 5, padding + 3, "make".to_string())));
    });
}

#[test]
fn a_nearby_free_function_is_not_an_associated_function() {
    let source = "struct T;\ntype Alias = T;\nfn make() {}\nfn run() {\n let _ = Alias::make;\n}\n";
    assert!(!dependencies(source)
        .iter()
        .any(|(line, _, name)| *line == 5 && name == "make"));
}

#[test]
fn an_imported_type_does_not_claim_a_local_member() {
    let source = "use external::T;\nstruct Local;\nimpl Local { fn make() {} }\nfn run() {\n let _ = T::make;\n}\n";
    assert!(!dependencies(source)
        .iter()
        .any(|(line, _, name)| *line == 5 && name == "make"));
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
