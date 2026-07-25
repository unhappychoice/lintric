use lintric_core::languages::{rust, typescript};
use lintric_core::{analyze_content, Language};

/// A query that fails to compile yields no captures, and the extractors treat that as "nothing
/// declared here" — so a typo in a `.scm` file would silently empty the definition list rather than
/// fail. These assert the files compile against the grammar they are written for.
#[test]
fn the_rust_definition_query_compiles_and_matches() {
    let source = "struct P {\n    x: i32,\n}\n";
    let (ir, _) = analyze_content(source.to_string(), Language::Rust).unwrap();

    assert!(
        ir.definitions.iter().any(|d| d.name == "P"),
        "{:?}",
        ir.definitions.iter().map(|d| &d.name).collect::<Vec<_>>()
    );
    let _ = rust::definition_queries::declared_types;
}

#[test]
fn the_typescript_definition_query_compiles_and_matches() {
    let source = "class C {\n    m(): number {\n        return 1;\n    }\n}\n";
    let (ir, _) = analyze_content(source.to_string(), Language::TypeScript).unwrap();

    assert!(
        ir.definitions.iter().any(|d| d.name == "C"),
        "{:?}",
        ir.definitions.iter().map(|d| &d.name).collect::<Vec<_>>()
    );
    let _ = typescript::definition_queries::declared_types;
}

#[test]
fn the_typescript_definition_query_compiles_against_tsx_too() {
    let source = "class C {\n    m(): number {\n        return 1;\n    }\n}\n";
    let (ir, _) = analyze_content(source.to_string(), Language::TSX).unwrap();

    assert!(
        ir.definitions.iter().any(|d| d.name == "C"),
        "{:?}",
        ir.definitions.iter().map(|d| &d.name).collect::<Vec<_>>()
    );
}
