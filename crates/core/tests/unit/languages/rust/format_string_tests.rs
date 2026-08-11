use lintric_core::{analyze_content, Language};

#[test]
fn captures_an_inline_placeholder() {
    let dependencies =
        dependencies("fn main() {\n    let name = 1;\n    println!(\"{name}\");\n}\n");

    assert!(
        dependencies.contains(&(3, 2, "name".to_string())),
        "{dependencies:?}"
    );
}

#[test]
fn captures_every_placeholder_on_one_line() {
    let dependencies = dependencies(
        "fn main() {\n    let a = 1;\n    let b = 2;\n    println!(\"{a} {b}\");\n}\n",
    );

    assert!(
        dependencies.contains(&(4, 2, "a".to_string())),
        "{dependencies:?}"
    );
    assert!(
        dependencies.contains(&(4, 3, "b".to_string())),
        "{dependencies:?}"
    );
}

#[test]
fn captures_a_placeholder_carrying_a_format_spec() {
    let dependencies =
        dependencies("fn main() {\n    let name = 1;\n    println!(\"{name:?}\");\n}\n");

    assert!(
        dependencies.contains(&(3, 2, "name".to_string())),
        "{dependencies:?}"
    );
}

#[test]
fn captures_a_binding_referenced_by_a_width_spec() {
    let dependencies = dependencies(
        "fn main() {\n    let value = 1;\n    let width = 2;\n    println!(\"{value:>width$}\");\n}\n",
    );

    assert!(
        dependencies.contains(&(4, 2, "value".to_string())),
        "{dependencies:?}"
    );
    assert!(
        dependencies.contains(&(4, 3, "width".to_string())),
        "{dependencies:?}"
    );
}

#[test]
fn captures_from_a_format_string_that_is_not_the_first_argument() {
    let dependencies = dependencies(
        "fn main() {\n    let out = 1;\n    let name = 2;\n    write!(out, \"{name}\");\n}\n",
    );

    assert!(
        dependencies.contains(&(4, 3, "name".to_string())),
        "{dependencies:?}"
    );
}

#[test]
fn ignores_escaped_braces() {
    let usages = usage_names(
        "fn main() {\n    let name = 1;\n    println!(\"{{name}}\");\n}\n",
        3,
    );

    assert!(!usages.contains(&"name".to_string()), "{usages:?}");
}

#[test]
fn ignores_a_positional_placeholder() {
    let usages = usage_names(
        "fn main() {\n    let name = 1;\n    println!(\"{0}\", name);\n}\n",
        3,
    );

    assert_eq!(
        usages.iter().filter(|name| *name == "0").count(),
        0,
        "{usages:?}"
    );
}

#[test]
fn ignores_an_empty_placeholder() {
    let usages = usage_names(
        "fn main() {\n    let name = 1;\n    println!(\"{}\", name);\n}\n",
        3,
    );

    assert_eq!(
        usages.iter().filter(|name| *name == "name").count(),
        1,
        "the argument is the only usage: {usages:?}"
    );
}

#[test]
fn ignores_a_string_that_is_not_a_format_string() {
    let usages = usage_names(
        "fn main() {\n    let name = 1;\n    my_macro!(\"{name}\");\n}\n",
        3,
    );

    assert!(!usages.contains(&"name".to_string()), "{usages:?}");
}

#[test]
fn ignores_braces_in_a_later_string_argument() {
    let usages = usage_names(
        "fn main() {\n    let name = 1;\n    println!(\"{name}\", \"{other}\");\n}\n",
        3,
    );

    assert!(usages.contains(&"name".to_string()), "{usages:?}");
    assert!(!usages.contains(&"other".to_string()), "{usages:?}");
}

#[test]
fn points_a_capture_at_its_identifier_inside_the_string() {
    let source = "fn main() {\n    let name = 1;\n    println!(\"{name}\");\n}\n";
    let (ir, _) = analyze_content(source.to_string(), Language::Rust).unwrap();

    let capture = ir
        .usage
        .iter()
        .find(|usage| usage.name == "name" && usage.position.start_line == 3)
        .expect("capture usage not found");

    // `    println!("{name}");` places the identifier at column 16, 1-based.
    assert_eq!(capture.position.start_column, 16);
    assert_eq!(capture.position.end_column, 20);
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

fn usage_names(source: &str, line: usize) -> Vec<String> {
    let (ir, _) = analyze_content(source.to_string(), Language::Rust).unwrap();

    ir.usage
        .iter()
        .filter(|usage| usage.position.start_line == line)
        .map(|usage| usage.name.clone())
        .collect()
}
