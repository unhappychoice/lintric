use lintric_core::{analyze_content, Language};

const POINT: &str = "struct Point {\n    x: i32,\n    y: i32,\n}\n";

#[test]
fn resolves_a_named_field_to_its_declaration() {
    let source = format!("{POINT}\nfn main() {{\n    let p = Point {{ x: 1, y: 2 }};\n}}\n");
    let dependencies = dependencies(&source);

    assert!(
        dependencies.contains(&(7, 2, "x".to_string())),
        "{dependencies:?}"
    );
    assert!(
        dependencies.contains(&(7, 3, "y".to_string())),
        "{dependencies:?}"
    );
}

#[test]
fn resolves_a_shorthand_field_to_both_the_declaration_and_the_binding() {
    let source =
        format!("{POINT}\nfn main() {{\n    let x = 1;\n    let p = Point {{ x, y: 2 }};\n}}\n");
    let dependencies = dependencies(&source);

    assert!(
        dependencies.contains(&(8, 2, "x".to_string())),
        "field declaration: {dependencies:?}"
    );
    assert!(
        dependencies.contains(&(8, 7, "x".to_string())),
        "local binding: {dependencies:?}"
    );
}

#[test]
fn treats_a_functional_update_base_as_a_plain_binding() {
    let source = format!("{POINT}\nfn main() {{\n    let p = Point {{ x: 1, y: 2 }};\n    let q = Point {{ ..p }};\n}}\n");
    let dependencies = dependencies(&source);

    assert!(
        dependencies.contains(&(8, 7, "p".to_string())),
        "{dependencies:?}"
    );
    assert!(
        !dependencies
            .iter()
            .any(|(source_line, _, symbol)| *source_line == 8 && symbol == "x"),
        "a functional update names no field: {dependencies:?}"
    );
}

#[test]
fn resolves_a_field_of_an_enum_struct_variant() {
    let source = "enum Shape {\n    Rect { width: i32 },\n}\n\nfn main() {\n    let s = Shape::Rect { width: 1 };\n}\n";
    let dependencies = dependencies(source);

    assert!(
        dependencies.contains(&(6, 2, "width".to_string())),
        "{dependencies:?}"
    );
}

#[test]
fn prefers_the_field_declaration_over_a_method_of_the_same_name() {
    let source = "struct Point {\n    x: i32,\n}\n\nimpl Point {\n    fn x(&self) -> i32 {\n        0\n    }\n}\n\nfn main() {\n    let p = Point { x: 1 };\n}\n";
    let dependencies = dependencies(source);

    assert!(
        dependencies.contains(&(12, 2, "x".to_string())),
        "{dependencies:?}"
    );
    assert!(
        !dependencies.contains(&(12, 6, "x".to_string())),
        "a field initializer cannot name a method: {dependencies:?}"
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
