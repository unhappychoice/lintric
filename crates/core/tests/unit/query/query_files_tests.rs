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

#[test]
fn the_receiver_narrowing_queries_compile_for_both_typescript_grammars() {
    // These run on every file, so a node name absent from one grammar would fail all analysis of
    // that dialect rather than only the member access it was written for.
    let source = "interface R {\n    label: string;\n}\nfunction f(r: R): string {\n    return r.label;\n}\n";

    for language in [Language::TypeScript, Language::TSX] {
        let (ir, _) = analyze_content(source.to_string(), language.clone()).unwrap();

        assert!(
            ir.dependencies
                .iter()
                .any(|d| d.symbol == "label" && d.target_line == 2),
            "{language:?}: {:?}",
            ir.dependencies
        );
    }
}
