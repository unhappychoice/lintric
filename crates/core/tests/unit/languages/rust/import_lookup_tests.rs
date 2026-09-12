use lintric_core::{analyze_content, Language};

#[test]
fn preserves_a_method_receiver() {
    let source = "use std::f64::consts::PI;\nfn run() {\n let _ = PI.abs();\n}\n";
    assert_eq!(targets(source, 3, "PI"), vec![1]);
}

#[test]
fn preserves_a_spaced_path_head() {
    let source = "use std::time::Duration;\nfn run() {\n let _ = Duration ::from_secs(1);\n}\n";
    assert_eq!(targets(source, 3, "Duration"), vec![1]);
}

#[test]
fn preserves_a_multiline_path_head() {
    let source = "use std::time::Duration;\nfn run() {\n let _ = Duration::\n from_secs(1);\n}\n";
    assert_eq!(targets(source, 3, "Duration"), vec![1]);
}

#[test]
fn preserves_receivers_and_arguments_inside_larger_expressions() {
    [
        "(PI).abs()",
        "PI.abs().abs()",
        "(PI + PI).abs()",
        "value.max(PI)",
    ]
    .into_iter()
    .for_each(|expression| {
        let source = format!(
            "use std::f64::consts::PI;\nfn run(value: f64) {{\n let _ = {expression};\n}}\n"
        );
        assert_eq!(targets(&source, 3, "PI"), vec![1], "{expression}");
    });
}

#[test]
fn does_not_treat_a_method_as_an_import() {
    let source = "use external::abs;\nfn run(value: f64) {\n let _ = value.abs();\n}\n";
    assert!(targets(source, 3, "abs").is_empty());
}

#[test]
fn does_not_treat_a_field_as_an_import() {
    let source = "use external::field;\nfn run(value: External) {\n let _ = value.field;\n}\n";
    assert!(targets(source, 3, "field").is_empty());
}

#[test]
fn preserves_path_heads_with_comments() {
    [
        "Duration /* gap */ ::from_secs(1)",
        "Duration\n ::from_secs(1)",
        "Duration :: from_secs(1)",
    ]
    .into_iter()
    .for_each(|expression| {
        let source = format!("use std::time::Duration;\nfn run() {{\n let _ = {expression};\n}}\n");
        assert_eq!(targets(&source, 3, "Duration"), vec![1], "{expression}");
    });
}

#[test]
fn preserves_generic_path_heads() {
    let source = "use std::vec::Vec;\nfn run() {\n let _ = Vec::<u8> ::new();\n}\n";
    assert_eq!(targets(source, 3, "Vec"), vec![1]);
}

#[test]
fn excludes_qualified_members_regardless_of_whitespace() {
    ["a :: f(1)", "a::\n f(1)", "a /* gap */ :: f(1)", "a::f::<i32>(1)"].into_iter().for_each(|expression| {
        let source = format!("mod a {{ pub fn f<T>(_: T) {{}} }}\nuse external::f;\nfn run() {{\n {expression};\n}}\n");
        let line = 4 + expression.matches('\n').count();
        assert_eq!(targets(&source, line, "f"), vec![1], "{expression}");
    });
}

#[test]
fn distinguishes_heads_and_members_in_type_paths() {
    let source = "use std::time;\ntype T = time :: Duration;\n";
    assert_eq!(targets(source, 2, "time"), vec![1]);

    let source = "use external::Duration;\ntype T = std::time :: Duration;\n";
    assert!(targets(source, 2, "Duration").is_empty());
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
