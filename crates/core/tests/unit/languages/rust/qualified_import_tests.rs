use lintric_core::{analyze_content, Language};

#[test]
fn resolves_root_imports_through_self_and_crate() {
    ["self", "crate"].into_iter().for_each(|qualifier| {
        let source =
            format!("use std::f64::consts::PI;\nfn run() {{\n let _ = {qualifier}::PI;\n}}\n");
        assert_eq!(targets(&source, 3, "PI"), vec![1], "{qualifier}");
    });
}

#[test]
fn resolves_each_relative_module_instead_of_a_same_named_local_import() {
    let source = "use external::PI;\nmod outer {\n use other::PI;\n mod inner {\n  use third::PI;\n  fn run() {\n   use fourth::PI;\n   let _ = self::PI;\n   let _ = super::PI;\n   let _ = super::super::PI;\n   let _ = crate::PI;\n  }\n }\n}\n";
    [(8, 5), (9, 3), (10, 1), (11, 1)]
        .into_iter()
        .for_each(|(line, target)| {
            assert_eq!(targets(source, line, "PI"), vec![target]);
        });
}

#[test]
fn resolves_module_qualified_reexports() {
    let source = "mod unrelated { pub use other::PI; }\nmod constants { pub use std::f64::consts::PI; }\nfn run() {\n let _ = constants::PI;\n let _ = crate::constants::PI;\n let _ = self::constants::PI;\n}\n";
    [4, 5, 6].into_iter().for_each(|line| {
        assert_eq!(targets(source, line, "PI"), vec![2]);
    });
}

#[test]
fn a_module_path_does_not_reach_a_function_or_block_local_import() {
    let source = "fn elsewhere() { use external::PI; }\nfn run() {\n use other::PI;\n let _ = self::PI;\n let _ = crate::PI;\n}\n";
    [4, 5]
        .into_iter()
        .for_each(|line| assert!(targets(source, line, "PI").is_empty()));
}

#[test]
fn a_module_path_does_not_reach_an_unrelated_module_import() {
    let source = "mod other { pub use external::PI; }\nmod empty {}\nfn run() {\n let _ = self::PI;\n let _ = crate::PI;\n let _ = empty::PI;\n let _ = missing::PI;\n let _ = super::PI;\n}\n";
    (4..=8).for_each(|line| assert!(targets(source, line, "PI").is_empty()));
}

#[test]
fn resolves_super_from_a_module_nested_in_a_function() {
    let source = "use external::PI;\nfn outer() {\n use other::PI;\n mod child {\n  fn run() {\n   let _ = super::PI;\n  }\n }\n}\n";
    assert_eq!(targets(source, 6, "PI"), vec![1]);
}

#[test]
fn resolves_reexports_in_type_paths_and_across_whitespace() {
    let source =
        "mod types { pub use external::T; }\ntype Alias = crate :: types /* gap */ :: T;\n";
    assert_eq!(targets(source, 2, "T"), vec![1]);
}

#[test]
fn a_qualified_import_wins_over_unrelated_nonimport_definitions() {
    let source = "const PI: f64 = 0.0;\nmod constants { pub use std::f64::consts::PI; }\nfn run() {\n let _ = constants::PI;\n}\n";
    assert_eq!(targets(source, 4, "PI"), vec![2]);
}

#[test]
fn a_shadowed_module_does_not_supply_an_import() {
    ["use external::constants;", "type constants = External;", "struct constants;"].into_iter().for_each(|binding| {
        let source = format!("mod constants {{ pub use external::PI; }}\nfn run() {{\n {binding}\n let _ = constants::PI;\n}}\n");
        assert!(targets(&source, 4, "PI").is_empty(), "{binding}");
    });
}

#[test]
fn a_relative_module_path_does_not_search_past_its_module_boundary() {
    let source = "mod constants { pub use external::PI; }\nmod caller {\n fn run() {\n  let _ = constants::PI;\n  let _ = super::constants::PI;\n }\n}\n";
    assert!(targets(source, 4, "PI").is_empty());
    assert_eq!(targets(source, 5, "PI"), vec![1]);
}

#[test]
fn a_nearer_module_shadows_a_same_named_outer_module() {
    let source = "mod constants { pub use external::PI; }\nfn run() {\n mod constants { pub use other::PI; }\n let _ = constants::PI;\n let _ = crate::constants::PI;\n}\n";
    assert_eq!(targets(source, 4, "PI"), vec![3]);
    assert_eq!(targets(source, 5, "PI"), vec![1]);
}

#[test]
fn an_import_in_a_sibling_block_does_not_shadow_a_module() {
    let source = "mod constants { pub use external::PI; }\nfn run() {\n { use other::constants; }\n let _ = constants::PI;\n}\n";
    assert_eq!(targets(source, 4, "PI"), vec![1]);
}

fn targets(source: &str, from: usize, symbol: &str) -> Vec<usize> {
    let (ir, _) = analyze_content(source.to_string(), Language::Rust).unwrap();
    let mut lines: Vec<_> = ir
        .dependencies
        .iter()
        .filter(|dependency| dependency.source_line == from && dependency.symbol == symbol)
        .map(|dependency| dependency.target_line)
        .collect();
    lines.sort();
    lines.dedup();
    lines
}
