use lintric_core::{analyze_content, Language};

#[test]
fn a_local_binding_shadows_a_top_level_function_of_the_same_name() {
    let source = "fn main() {\n    let x = 1;\n    let y = x;\n}\n\nfn x() -> i32 {\n    1\n}\n";

    assert_eq!(targets(source, 3, "x"), vec![2]);
}

#[test]
fn a_function_nested_in_another_function_is_out_of_reach() {
    let source = "fn main() {\n    let x = 1;\n    let y = x;\n}\n\nfn other() {\n    fn x() -> i32 {\n        1\n    }\n}\n";

    assert_eq!(targets(source, 3, "x"), vec![2]);
}

#[test]
fn the_latest_preceding_binding_wins_within_one_scope() {
    let source = "fn main() {\n    let a = 1;\n    let a = 2;\n    let b = a;\n}\n";

    assert_eq!(targets(source, 4, "a"), vec![3]);
}

#[test]
fn a_rebinding_reads_the_binding_it_shadows() {
    let source = "fn main() {\n    let a = 1;\n    let a = a + 1;\n}\n";

    assert_eq!(targets(source, 3, "a"), vec![2]);
}

#[test]
fn a_binding_shadowed_inside_a_block_is_readable_again_after_it() {
    let source = "fn main() {\n    let a = 1;\n    {\n        let a = 2;\n        let _ = a;\n    }\n    let _ = a;\n}\n";

    assert_eq!(targets(source, 5, "a"), vec![4]);
    assert_eq!(targets(source, 7, "a"), vec![2]);
}

#[test]
fn a_call_still_reaches_a_top_level_function() {
    let source = "fn helper() -> i32 {\n    1\n}\n\nfn main() {\n    let n = helper();\n}\n";

    assert_eq!(targets(source, 6, "helper"), vec![1]);
}

#[test]
fn a_qualified_path_still_reaches_a_function_inside_a_module() {
    let source = "mod inner {\n    pub fn unit() -> i32 {\n        1\n    }\n}\n\nfn main() {\n    let n = inner::unit();\n}\n";

    assert_eq!(targets(source, 8, "unit"), vec![2]);
}

#[test]
fn a_method_call_inside_a_macro_still_reaches_the_method() {
    // Inside a macro token tree `c.get()` is not parsed as a call, so the method name arrives as
    // a bare identifier. Lexical scope must not exclude it.
    let source = "struct C;\n\nimpl C {\n    fn get(&self) -> i32 {\n        1\n    }\n}\n\nfn main() {\n    let c = C;\n    println!(\"{}\", c.get());\n}\n";

    assert_eq!(targets(source, 11, "get"), vec![4]);
}

#[test]
fn a_qualified_call_does_not_prefer_an_unrelated_import() {
    ["fn elsewhere() { use crate::b::f; }", "use crate::b::f;"].into_iter().for_each(|import| {
        let source = format!("mod a {{ pub fn f() {{}} }}\nmod b {{ pub fn f() {{}} }}\n{import}\nfn run() {{\n a::f();\n}}\n");
        let (ir, _) = analyze_content(source, Language::Rust).unwrap();
        let calls: Vec<_> = ir.dependencies.iter()
            .filter(|dependency| dependency.source_line == 5 && dependency.symbol == "f")
            .map(|dependency| (dependency.target_line, dependency.dependency_type.clone()))
            .collect();
        assert_eq!(calls, vec![(1, lintric_core::models::DependencyType::FunctionCall)]);
    });
}

#[test]
fn an_import_is_visible_only_in_its_lexical_scope() {
    let source = "fn elsewhere() { use external::f; }\nfn run() {\n f();\n}\n";
    assert!(targets(source, 3, "f").is_empty());

    let source = "fn run() {\n { use external::f; }\n f();\n}\n";
    assert!(targets(source, 3, "f").is_empty());
}

#[test]
fn a_local_import_wins_in_any_function_and_its_nested_blocks() {
    ["main", "run"].into_iter().for_each(|name| {
        let source = format!(
            "mod m {{ pub fn f() {{}} }}\nfn {name}() {{\n use crate::m::f;\n {{\n f();\n }}\n}}\n"
        );
        assert_eq!(targets(&source, 5, "f"), vec![3]);
    });
}

#[test]
fn an_imported_path_head_still_reaches_its_binding() {
    let source = "mod m { pub struct T; }\nuse m::T;\nfn run() {\n T::new();\n}\n";
    assert_eq!(targets(source, 4, "T"), vec![2]);
}

fn targets(source: &str, from: usize, symbol: &str) -> Vec<usize> {
    let (ir, _) = analyze_content(source.to_string(), Language::Rust).unwrap();

    let mut lines: Vec<usize> = ir
        .dependencies
        .iter()
        .filter(|dependency| dependency.source_line == from && dependency.symbol == symbol)
        .map(|dependency| dependency.target_line)
        .collect();

    lines.sort();
    lines.dedup();
    lines
}
