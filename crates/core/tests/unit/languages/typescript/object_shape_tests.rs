use lintric_core::{analyze_content, Language};

const POINT: &str = "interface Point {\n    x: number;\n}\n";

#[test]
fn an_object_literal_key_does_not_reference_a_member() {
    let source = format!("{POINT}\nconst bare = {{ x: 1 }};\n");

    assert!(!references_member(&source), "{:?}", dependencies(&source));
}

#[test]
fn an_annotated_object_literal_records_only_the_type_it_names() {
    // The coupling to the type is the annotation, not the field name.
    let source = format!("{POINT}\nconst annotated: Point = {{ x: 1 }};\n");
    let dependencies = dependencies(&source);

    assert!(
        dependencies.contains(&(5, 1, "Point".to_string())),
        "{dependencies:?}"
    );
    assert!(!references_member(&source), "{dependencies:?}");
}

#[test]
fn a_destructuring_pattern_key_does_not_reference_a_member() {
    let source = format!("{POINT}\nfunction f({{ x }}: Point): number {{\n    return x;\n}}\n");

    assert!(!references_member(&source), "{:?}", dependencies(&source));
}

#[test]
fn a_renaming_pattern_key_does_not_reference_a_member() {
    let source =
        format!("{POINT}\nconst origin: Point = {{ x: 1 }};\nconst {{ x: renamed }} = origin;\n");

    assert!(!references_member(&source), "{:?}", dependencies(&source));
}

#[test]
fn reading_a_member_through_a_value_still_resolves() {
    // Dropping the literal's key must not touch ordinary member access.
    let source = format!("{POINT}\nconst origin: Point = {{ x: 1 }};\nconst v = origin.x;\n");

    assert!(
        dependencies(&source).contains(&(6, 2, "x".to_string())),
        "{:?}",
        dependencies(&source)
    );
}

fn references_member(source: &str) -> bool {
    dependencies(source)
        .iter()
        .any(|(_, target, symbol)| *target == 2 && symbol == "x")
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
