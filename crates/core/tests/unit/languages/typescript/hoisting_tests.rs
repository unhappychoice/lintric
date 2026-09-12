use lintric_core::{analyze_content, Language};

#[test]
fn var_hoists_but_let_and_const_do_not() {
    let source = "var a = 1;\nlet b = 2;\nconst c = 3;\n";

    assert_eq!(hoisting(source, "a"), Some(true));
    assert_eq!(hoisting(source, "b"), Some(false));
    assert_eq!(hoisting(source, "c"), Some(false));
}

#[test]
fn declarations_that_hoist_survive_scope_assignment() {
    let source =
        "function f() {}\ninterface I { x: number }\nclass C {}\ntype T = number;\nenum E { V }\n";

    for name in ["f", "I", "C", "T", "E"] {
        assert_eq!(hoisting(source, name), Some(true), "{name} should hoist");
    }
}

#[test]
fn members_do_not_hoist() {
    let source = "class C {\n    field: number = 1;\n    method(): void {}\n}\n";

    assert_eq!(hoisting(source, "field"), Some(false));
    assert_eq!(hoisting(source, "method"), Some(false));
}

fn hoisting(source: &str, name: &str) -> Option<bool> {
    let (ir, _) = analyze_content(source.to_string(), Language::TypeScript).unwrap();

    ir.definitions
        .iter()
        .find(|definition| definition.name == name)
        .and_then(|definition| definition.is_hoisted())
}
