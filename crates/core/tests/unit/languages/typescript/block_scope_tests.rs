use lintric_core::{analyze_content, Language};

#[test]
fn a_block_binding_is_invisible_outside_the_block() {
    let source = "const a = 1;\n\nfunction f(): number {\n    {\n        const a = 2;\n        return a;\n    }\n}\n\nconst b = a;\n";
    let dependencies = dependencies(source);

    assert!(
        dependencies.contains(&(6, 5, "a".to_string())),
        "inside the block reads the block's binding: {dependencies:?}"
    );
    assert!(
        dependencies.contains(&(10, 1, "a".to_string())),
        "outside reads the outer binding: {dependencies:?}"
    );
    assert!(
        !dependencies.contains(&(10, 5, "a".to_string())),
        "outside must not reach into the block: {dependencies:?}"
    );
}

#[test]
fn a_binding_in_one_function_is_invisible_from_another() {
    let source = "function one(): number {\n    const v = 1;\n    return v;\n}\n\nfunction two(): number {\n    const v = 2;\n    return v;\n}\n";
    let dependencies = dependencies(source);

    assert!(
        dependencies.contains(&(3, 2, "v".to_string())),
        "{dependencies:?}"
    );
    assert!(
        dependencies.contains(&(8, 7, "v".to_string())),
        "{dependencies:?}"
    );
    assert!(
        !dependencies.contains(&(8, 2, "v".to_string())),
        "{dependencies:?}"
    );
}

#[test]
fn a_hoisted_declaration_is_reachable_before_its_own_line() {
    let source = "const n = helper();\n\nfunction helper(): number {\n    return 1;\n}\n";

    assert!(
        dependencies(source).contains(&(1, 3, "helper".to_string())),
        "{:?}",
        dependencies(source)
    );
}

#[test]
fn a_method_stays_reachable_through_a_receiver() {
    // A member is reached through a receiver, not by a name in scope, so the scope it sits in must
    // not restrict who can call it.
    let source = "class C {\n    run(): number {\n        return 1;\n    }\n}\n\nconst c = new C();\nconst v = c.run();\n";

    assert!(
        dependencies(source).contains(&(8, 2, "run".to_string())),
        "{:?}",
        dependencies(source)
    );
}

#[test]
fn a_loop_body_scopes_its_bindings() {
    let source = "const total = 0;\n\nfor (let i = 0; i < 3; i++) {\n    const total = i;\n    console.log(total);\n}\n\nconsole.log(total);\n";
    let dependencies = dependencies(source);

    assert!(
        dependencies.contains(&(5, 4, "total".to_string())),
        "{dependencies:?}"
    );
    assert!(
        dependencies.contains(&(8, 1, "total".to_string())),
        "{dependencies:?}"
    );
}

fn dependencies(source: &str) -> Vec<(usize, usize, String)> {
    let (ir, _) = analyze_content(source.to_string(), Language::TypeScript).unwrap();

    ir.dependencies
        .iter()
        .map(|d| (d.source_line, d.target_line, d.symbol.clone()))
        .collect()
}
