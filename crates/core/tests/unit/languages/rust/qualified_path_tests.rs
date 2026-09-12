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

#[test]
fn aliases_select_the_actual_member_owner_in_both_orders_near_and_far() {
    [0, 25].into_iter().for_each(|padding| {
        [false, true].into_iter().for_each(|reversed| {
            ["type A = B;", "type A = Alias;\ntype Alias = B;"]
                .into_iter()
                .for_each(|alias| {
                    [
                        ("fn f() {}", "A::f();", "f"),
                        ("fn f() {}", "let _ = A::f;", "f"),
                        ("const VALUE: i32 = 1;", "let _ = A::VALUE;", "VALUE"),
                        ("type Item = i32;", "let _: A::Item;", "Item"),
                    ]
                    .into_iter()
                    .for_each(|(member, access, symbol)| {
                        let owners = if reversed { ["B", "C"] } else { ["C", "B"] };
                        let blocks =
                            owners.map(|owner| format!("impl {owner} {{\n {member}\n}}\n"));
                        let source = format!(
                            "struct B;\n{alias}\n{}struct C;\n{}{}fn run() {{ {access} }}",
                            "\n".repeat(padding),
                            blocks[0],
                            blocks[1]
                        );
                        let usage_line = source.lines().count();
                        let member_line = usage_line - if reversed { 5 } else { 2 };
                        let actual: Vec<_> = dependencies(&source)
                            .into_iter()
                            .filter(|(line, _, name)| *line == usage_line && name == symbol)
                            .collect();
                        assert_eq!(
                            actual,
                            vec![(usage_line, member_line, symbol.to_string())],
                            "{source}"
                        );
                    });
                });
        });
    });
}

#[test]
fn unresolved_alias_owners_do_not_claim_unrelated_members() {
    [
        "type A = Missing;",
        "use external::Remote;\ntype A = Remote;",
        "type A = external::B;",
        "type A = Alias;\ntype Alias = A;",
        "type A = (B, B);",
    ]
    .into_iter()
    .for_each(|alias| {
        let source =
            format!("{alias}\nstruct B;\nimpl B {{ fn f() {{}} }}\nfn run() {{ A::f(); }}");
        let usage_line = source.lines().count();
        assert!(
            !dependencies(&source)
                .iter()
                .any(|(line, _, name)| *line == usage_line && name == "f"),
            "{source}"
        );
    });
}

#[test]
fn enum_alias_chains_select_variants_from_the_correct_enum() {
    [false, true].into_iter().for_each(|reversed| {
        let enums = if reversed {
            "enum B { V }\nenum C { V }"
        } else {
            "enum C { V }\nenum B { V }"
        };
        let source =
            format!("{enums}\ntype Alias = B;\ntype A = Alias;\nfn run() {{ let _ = A::V; }}");
        let actual: Vec<_> = dependencies(&source)
            .into_iter()
            .filter(|(line, _, name)| *line == 5 && name == "V")
            .collect();
        assert_eq!(
            actual,
            vec![(5, if reversed { 1 } else { 2 }, "V".to_string())]
        );
    });
}

#[test]
fn alias_targets_are_resolved_in_the_alias_declaration_scope() {
    let source = "struct B;\nimpl B { fn f() {} }\ntype A = B;\nfn run() {\n struct B;\n impl B { fn f() {} }\n A::f();\n B::f();\n}";
    let actual: Vec<_> = dependencies(source)
        .into_iter()
        .filter(|(line, _, name)| *line >= 7 && name == "f")
        .collect();
    assert_eq!(
        actual,
        vec![(7, 2, "f".to_string()), (8, 6, "f".to_string())]
    );
}

#[test]
fn local_module_paths_and_alias_targets_keep_their_owner_identity() {
    ["m::B::f();", "A::f();"].into_iter().for_each(|access| {
        let source = format!("struct B;\nimpl B {{ fn f() {{}} }}\nmod m {{\n pub struct B;\n impl B {{ pub fn f() {{}} }}\n}}\ntype A = crate::m::B;\nfn run() {{ {access} }}");
        let actual: Vec<_> = dependencies(&source).into_iter().filter(|(line, _, name)| *line == 8 && name == "f").collect();
        assert_eq!(actual, vec![(8, 5, "f".to_string())], "{source}");
    });
}

#[test]
fn a_known_owner_without_the_member_does_not_claim_another_owners_member() {
    [0, 25].into_iter().for_each(|padding| {
        let source = format!("struct B;\ntype A = B;\n{}struct C;\nimpl C {{ fn f() {{}} }}\nfn run() {{ A::f(); B::f(); }}", "\n".repeat(padding));
        assert!(!dependencies(&source).iter().any(|(line, _, name)| *line == padding + 5 && name == "f"));
    });
}

#[test]
fn imported_path_heads_do_not_resolve_to_same_named_local_types_or_members() {
    ["use external as remote;", ""]
        .into_iter()
        .for_each(|import| {
            let source = format!(
                "{import}\nstruct B;\nimpl B {{ fn f() {{}} }}\nfn run() {{ remote::B::f(); }}"
            );
            assert!(!dependencies(&source)
                .iter()
                .any(|(line, _, name)| *line == 4 && (name == "B" || name == "f")));
        });
}

#[test]
fn nested_ufcs_associated_paths_preserve_existing_resolution() {
    ["<Self as T>::A", "Self::A"].into_iter().for_each(|qualifier| {
        let source = format!("trait T {{ type A; }}\nstruct A;\nimpl A {{\n fn f() {{}}\n}}\nstruct B;\nimpl T for B {{ type A = A; }}\nimpl B {{ fn run() {{ {qualifier}::f(); }} }}");
        assert!(dependencies(&source).contains(&(8, 4, "f".to_string())));
    });
}

#[test]
fn primitive_owners_reach_only_the_traits_they_implement() {
    [false, true].into_iter().for_each(|reversed| {
        let traits = if reversed {
            "trait T {\n fn f() {}\n}\ntrait Other { fn f() {} }"
        } else {
            "trait Other { fn f() {} }\ntrait T {\n fn f() {}\n}"
        };
        let source = format!(
            "{traits}\nimpl Other for u32 {{}}\nimpl T for i32 {{}}\nfn run() {{ i32::f(); }}"
        );
        let actual: Vec<_> = dependencies(&source)
            .into_iter()
            .filter(|(line, _, name)| *line == 7 && name == "f")
            .collect();
        assert_eq!(
            actual,
            vec![(7, if reversed { 2 } else { 3 }, "f".to_string())]
        );
    });
}

#[test]
fn primitive_default_method_preserves_existing_resolution() {
    let source = "trait T {\n fn f() {}\n}\nimpl T for i32 {}\nfn run() { i32::f(); }";
    assert!(dependencies(source).contains(&(5, 2, "f".to_string())));
}

#[test]
fn primitive_without_a_matching_impl_does_not_claim_a_local_member() {
    let source = "trait T { fn f() {} }\nstruct A;\nimpl T for A {}\nfn run() { i32::f(); }";
    assert!(!dependencies(source)
        .iter()
        .any(|(line, _, name)| *line == 4 && name == "f"));
}

#[test]
fn concrete_and_primitive_overrides_beat_trait_defaults_in_both_orders() {
    ["B", "i32"].into_iter().for_each(|owner| {
        [false, true].into_iter().for_each(|trait_first| {
            let declaration = "trait T {\n fn f() {}\n}\n";
            let implementation = format!("impl T for {owner} {{\n fn f() {{}}\n}}\n");
            let blocks = if trait_first {
                format!("{declaration}{implementation}")
            } else {
                format!("{implementation}{declaration}")
            };
            let source = format!("struct B;\n{blocks}fn run() {{ {owner}::f(); }}");
            let actual: Vec<_> = dependencies(&source)
                .into_iter()
                .filter(|(line, _, name)| *line == 8 && name == "f")
                .collect();
            assert_eq!(
                actual,
                vec![(8, if trait_first { 6 } else { 3 }, "f".to_string())],
                "{source}"
            );
        });
    });
}

#[test]
fn concrete_and_primitive_owners_fall_back_to_trait_defaults_in_both_orders() {
    ["B", "i32"].into_iter().for_each(|owner| {
        [false, true].into_iter().for_each(|trait_first| {
            let declaration = "trait T {\n fn f() {}\n}\n";
            let implementation = format!("impl T for {owner} {{}}\n");
            let blocks = if trait_first {
                format!("{declaration}{implementation}")
            } else {
                format!("{implementation}{declaration}")
            };
            let source = format!("struct B;\n{blocks}fn run() {{ {owner}::f(); }}");
            let actual: Vec<_> = dependencies(&source)
                .into_iter()
                .filter(|(line, _, name)| *line == 6 && name == "f")
                .collect();
            assert_eq!(
                actual,
                vec![(6, if trait_first { 3 } else { 4 }, "f".to_string())],
                "{source}"
            );
        });
    });
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
