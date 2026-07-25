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

#[test]
fn a_shorthand_property_reads_the_binding_it_names() {
    let source = "const x = 1;\nconst b = { x };\n";

    assert!(
        dependencies(&source).contains(&(2, 1, "x".to_string())),
        "{:?}",
        dependencies(&source)
    );
}

#[test]
fn a_shorthand_property_does_not_reference_a_member() {
    let source = format!("{POINT}\nconst x = 1;\nconst b = {{ x }};\n");

    assert!(!references_member(&source), "{:?}", dependencies(&source));
}

#[test]
fn two_interfaces_sharing_a_member_name_do_not_depend_on_each_other() {
    let source =
        "interface First {\n    id: string;\n}\n\ninterface Second {\n    id: number;\n}\n";
    let dependencies = dependencies(source);

    assert!(dependencies.is_empty(), "{dependencies:?}");
}

#[test]
fn a_method_signature_declaration_is_not_a_usage() {
    let source =
        "interface First {\n    run(): void;\n}\n\ninterface Second {\n    run(): void;\n}\n";
    let dependencies = dependencies(source);

    assert!(dependencies.is_empty(), "{dependencies:?}");
}

#[test]
fn a_declared_member_is_still_reachable_through_a_value() {
    let source =
        "interface A {\n    id: string;\n}\n\nfunction f(a: A): string {\n    return a.id;\n}\n";

    assert!(
        dependencies(source).contains(&(6, 2, "id".to_string())),
        "{:?}",
        dependencies(source)
    );
}
