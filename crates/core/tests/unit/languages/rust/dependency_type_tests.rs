use lintric_core::models::DependencyType;
use lintric_core::{analyze_content, Language};

#[test]
fn classifies_a_method_call_as_a_call_not_a_field_access() {
    let source = "struct P {\n    x: i32,\n}\n\nimpl P {\n    fn norm(&self) -> i32 {\n        0\n    }\n}\n\nfn main() {\n    let p = P { x: 1 };\n    let n = p.norm();\n}\n";

    assert_eq!(
        dependency_type(source, 13, 6),
        Some(DependencyType::FunctionCall)
    );
}

#[test]
fn classifies_a_field_read_as_a_field_access() {
    let source = "struct P {\n    x: i32,\n}\n\nimpl P {\n    fn get(&self) -> i32 {\n        self.x\n    }\n}\n";

    assert_eq!(
        dependency_type(source, 7, 2),
        Some(DependencyType::StructFieldAccess)
    );
}

#[test]
fn classifies_a_field_initializer_as_a_field_access() {
    let source = "struct P {\n    x: i32,\n}\n\nfn main() {\n    let p = P { x: 1 };\n}\n";

    assert_eq!(
        dependency_type(source, 6, 2),
        Some(DependencyType::StructFieldAccess)
    );
}

#[test]
fn classifies_an_enum_variant_reference_as_such() {
    let source = "enum D {\n    Left,\n}\n\nfn main() {\n    let d = D::Left;\n}\n";

    assert_eq!(
        dependency_type(source, 6, 2),
        Some(DependencyType::EnumVariantReference)
    );
}

#[test]
fn classifies_a_type_reference_as_such() {
    let source = "struct P {\n    x: i32,\n}\n\nfn take(p: P) -> i32 {\n    0\n}\n";

    assert_eq!(
        dependency_type(source, 5, 1),
        Some(DependencyType::TypeReference)
    );
}

#[test]
fn classifies_an_invocation_of_a_local_macro_as_such() {
    let source = "macro_rules! shout {\n    () => {};\n}\n\nfn main() {\n    shout!();\n}\n";

    assert_eq!(
        dependency_type(source, 6, 1),
        Some(DependencyType::MacroInvocation)
    );
}

#[test]
fn keeps_call_syntax_a_call_even_when_it_resolves_to_a_binding() {
    // Calling a binding that holds a closure is still a call, so the definition must not win.
    let source = "fn main() {\n    let run = || 1;\n    let n = run();\n}\n";

    assert_eq!(
        dependency_type(source, 3, 2),
        Some(DependencyType::FunctionCall)
    );
}

fn dependency_type(source: &str, from: usize, to: usize) -> Option<DependencyType> {
    let (ir, _) = analyze_content(source.to_string(), Language::Rust).unwrap();

    ir.dependencies
        .iter()
        .find(|dependency| dependency.source_line == from && dependency.target_line == to)
        .map(|dependency| dependency.dependency_type.clone())
}
