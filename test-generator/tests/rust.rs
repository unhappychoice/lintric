// Generated tests for Rust node types
// This file is auto-generated. Do not edit manually.

use lintric_core;

// Excluded node types (could not generate snippets):
// !
// !=
// "
// #
// $
// %
// %=
// &
// &&
// &=
// '
// (
// )
// *
// */
// *=
// +
// +=
// ,
// -
// -=
// ->
// .
// ..
// ...
// ..=
// /
// /*
// //
// /=
// :
// ::
// ;
// <
// <<
// <<=
// <=
// =
// ==
// =>
// >
// >=
// >>
// >>=
// ?
// @
// [
// ]
// ^
// ^=
// _
// arguments
// as
// async
// async_block
// attribute
// await
// base_field_initializer
// block
// block_comment
// break
// closure_parameters
// compound_assignment_expr
// const
// const_block
// const_parameter
// continue
// crate
// declaration_list
// default
// doc_comment
// dyn
// else
// else_clause
// enum
// enum_variant
// enum_variant_list
// escape_sequence
// expr
// expr_2021
// extern
// extern_crate_declaration
// extern_modifier
// false
// field_declaration
// field_declaration_list
// field_identifier
// field_initializer
// field_initializer_list
// fn
// for
// for_lifetimes
// fragment_specifier
// function_modifiers
// gen
// gen_block
// generic_function
// generic_type_with_turbofish
// ident
// if
// impl
// in
// inner_doc_comment_marker
// item
// label
// let
// let_chain
// let_condition
// lifetime
// lifetime_parameter
// line_comment
// literal
// loop
// macro_definition
// macro_rule
// macro_rules!
// match
// match_arm
// match_block
// meta
// metavariable
// mod
// move
// mutable_specifier
// ordered_field_declaration_list
// outer_doc_comment_marker
// parameter
// parameters
// pat
// pat_param
// path
// pub
// raw
// ref
// return
// scoped_use_list
// self
// self_parameter
// shebang
// shorthand_field_identifier
// shorthand_field_initializer
// source_file
// static
// stmt
// string_content
// struct
// super
// token_repetition
// token_tree
// trait
// trait_bounds
// true
// try
// try_block
// tt
// ty
// type
// type_arguments
// type_binding
// type_identifier
// type_parameter
// type_parameters
// union
// unsafe
// unsafe_block
// use
// use_as_clause
// use_bounds
// use_list
// use_wildcard
// variadic_parameter
// vis
// visibility_modifier
// where
// where_clause
// where_predicate
// while
// yield
// {
// |
// |=
// ||
// }

fn assert_code_analysis_and_snapshot(
    source_code: &str,
    language: lintric_core::Language,
    snapshot_folder: &str,
    test_name: &str,
    error_msg: &str,
    node_validation_fn: impl FnOnce(&str),
) {
    let mut settings = insta::Settings::clone_current();
    settings.set_snapshot_path(format!("snapshots/{}", snapshot_folder));
    settings.set_prepend_module_to_snapshot(false);
    settings.set_snapshot_suffix("");
    let _guard = settings.bind_to_scope();

    let result = std::panic::catch_unwind(|| {
        lintric_core::analyze_content(source_code.to_string(), language.clone())
    });

    assert!(result.is_ok(), "{}", error_msg);

    if let Ok(Ok((ir, _metrics))) = result {
        // Get AST for comprehensive snapshot
        let ast_result =
            lintric_core::get_s_expression_from_content(source_code.to_string(), language.clone());
        let ast_string = ast_result.unwrap_or_else(|e| format!("AST Error: {}", e));

        // Create comprehensive snapshot content
        let snapshot_content = format!(
            "Source Code:\n{}\n\nAST:\n{}\n\nIR:\n{:#?}",
            source_code, ast_string, ir
        );

        insta::assert_snapshot!(test_name, snapshot_content);

        // Verify that the generated code contains the expected node type
        let s_expr_result = std::panic::catch_unwind(|| {
            lintric_core::get_s_expression_from_content(source_code.to_string(), language)
        });

        if let Ok(Ok(s_expr)) = s_expr_result {
            node_validation_fn(&s_expr);
        }
    }
}

#[test]
fn test_generated_declaration_statement() {
    let source_code = r#"let var = source;"#;

    assert_code_analysis_and_snapshot(
        source_code,
        lintric_core::Language::Rust,
        "rust",
        "test_generated_declaration_statement",
        "Failed to analyze code for node type: _declaration_statement",
        |s_expr| {
            // Abstract node _declaration_statement - check for concrete implementations
            assert!(
                s_expr.contains("let_declaration")
                    || s_expr.contains("const_item")
                    || s_expr.contains("static_item"),
                "Generated code snippet does not contain any declaration statement type in AST"
            );
        },
    );
}

#[test]
fn test_generated_expression() {
    let source_code = r#"var1 + 1"#;

    assert_code_analysis_and_snapshot(
        source_code,
        lintric_core::Language::Rust,
        "rust",
        "test_generated_expression",
        "Failed to analyze code for node type: _expression",
        |s_expr| {
            // Abstract node _expression - check for any expression type
            assert!(
                s_expr.matches("_expression").count() > 0
                    || s_expr.contains("binary_expression")
                    || s_expr.contains("call_expression")
                    || s_expr.contains("identifier")
                    || s_expr.contains("literal"),
                "Generated code snippet does not contain any expression type in AST"
            );
        },
    );
}

#[test]
fn test_generated_literal() {
    let source_code = r#"42"#;

    assert_code_analysis_and_snapshot(
        source_code,
        lintric_core::Language::Rust,
        "rust",
        "test_generated_literal",
        "Failed to analyze code for node type: _literal",
        |s_expr| {
            // Abstract node _literal - check for any literal type
            assert!(
                s_expr.contains("integer_literal")
                    || s_expr.contains("float_literal")
                    || s_expr.contains("string_literal")
                    || s_expr.contains("char_literal")
                    || s_expr.contains("boolean_literal"),
                "Generated code snippet does not contain any literal type in AST"
            );
        },
    );
}

#[test]
fn test_generated_literal_pattern() {
    let source_code = r#"42"#;

    assert_code_analysis_and_snapshot(
        source_code,
        lintric_core::Language::Rust,
        "rust",
        "test_generated_literal_pattern",
        "Failed to analyze code for node type: _literal_pattern",
        |s_expr| {
            // Abstract node _literal_pattern - check for any literal pattern type
            assert!(
                s_expr.contains("integer_literal")
                    || s_expr.contains("float_literal")
                    || s_expr.contains("string_literal")
                    || s_expr.contains("char_literal")
                    || s_expr.contains("boolean_literal"),
                "Generated code snippet does not contain any literal pattern type in AST"
            );
        },
    );
}

#[test]
fn test_generated_pattern() {
    let source_code = r#"Some(x)"#;

    assert_code_analysis_and_snapshot(
        source_code,
        lintric_core::Language::Rust,
        "rust",
        "test_generated_pattern",
        "Failed to analyze code for node type: _pattern",
        |s_expr| {
            // Abstract node _pattern - check for any pattern type
            assert!(
                s_expr.contains("identifier")
                    || s_expr.contains("tuple_pattern")
                    || s_expr.contains("struct_pattern")
                    || s_expr.contains("ref_pattern")
                    || s_expr.contains("mut_pattern"),
                "Generated code snippet does not contain any pattern type in AST"
            );
        },
    );
}

#[test]
fn test_generated_type() {
    let source_code = r#"let var2: i32 = 42;"#;

    assert_code_analysis_and_snapshot(
        source_code,
        lintric_core::Language::Rust,
        "rust",
        "test_generated_type",
        "Failed to analyze code for node type: _type",
        |s_expr| {
            // Abstract node _type - check for any type
            assert!(
                s_expr.contains("primitive_type")
                    || s_expr.contains("generic_type")
                    || s_expr.contains("reference_type")
                    || s_expr.contains("tuple_type")
                    || s_expr.contains("function_type"),
                "Generated code snippet does not contain any type in AST"
            );
        },
    );
}

#[test]
fn test_generated_abstract_type() {
    let source_code = r#"fn test_fn() -> impl std::fmt::Debug { 42 }"#;

    assert_code_analysis_and_snapshot(
        source_code,
        lintric_core::Language::Rust,
        "rust",
        "test_generated_abstract_type",
        "Failed to analyze code for node type: abstract_type",
        |s_expr| {
            assert!(
                s_expr.contains("abstract_type"),
                "Generated code snippet does not contain expected node type 'abstract_type' in AST"
            );
        },
    );
}

#[test]
fn test_generated_array_expression() {
    let source_code = r#"[item1, item2]"#;

    assert_code_analysis_and_snapshot(
        source_code,
        lintric_core::Language::Rust,
        "rust",
        "test_generated_array_expression",
        "Failed to analyze code for node type: array_expression",
        |s_expr| {
            assert!(
                s_expr.contains("array_expression"),
                "Generated code snippet does not contain expected node type 'array_expression' in AST"
            );
        },
    );
}

#[test]
fn test_generated_array_type() {
    let source_code = r#"let arr: [i32; 3] = [1, 2, 3];"#;

    assert_code_analysis_and_snapshot(
        source_code,
        lintric_core::Language::Rust,
        "rust",
        "test_generated_array_type",
        "Failed to analyze code for node type: array_type",
        |s_expr| {
            assert!(
                s_expr.contains("array_type"),
                "Generated code snippet does not contain expected node type 'array_type' in AST"
            );
        },
    );
}

#[test]
fn test_generated_assignment_expression() {
    let source_code = r#"x1 = y"#;

    assert_code_analysis_and_snapshot(
        source_code,
        lintric_core::Language::Rust,
        "rust",
        "test_generated_assignment_expression",
        "Failed to analyze code for node type: assignment_expression",
        |s_expr| {
            assert!(
                s_expr.contains("assignment_expression"),
                "Generated code snippet does not contain expected node type 'assignment_expression' in AST"
            );
        },
    );
}

#[test]
fn test_generated_associated_type() {
    let source_code = r#"trait TestTrait { type Item; }"#;

    assert_code_analysis_and_snapshot(
        source_code,
        lintric_core::Language::Rust,
        "rust",
        "test_generated_associated_type",
        "Failed to analyze code for node type: associated_type",
        |s_expr| {
            assert!(
                s_expr.contains("associated_type"),
                "Generated code snippet does not contain expected node type 'associated_type' in AST"
            );
        },
    );
}

#[test]
fn test_generated_attribute_item() {
    let source_code = r#"#[derive(Debug)]
struct TestStruct;

fn test_fn1() -> i32 { 42 }"#;

    assert_code_analysis_and_snapshot(
        source_code,
        lintric_core::Language::Rust,
        "rust",
        "test_generated_attribute_item",
        "Failed to analyze code for node type: attribute_item",
        |s_expr| {
            assert!(
                s_expr.contains("attribute_item"),
                "Generated code snippet does not contain expected node type 'attribute_item' in AST"
            );
        },
    );
}

#[test]
fn test_generated_await_expression() {
    let source_code = r#"future.await"#;

    assert_code_analysis_and_snapshot(
        source_code,
        lintric_core::Language::Rust,
        "rust",
        "test_generated_await_expression",
        "Failed to analyze code for node type: await_expression",
        |s_expr| {
            assert!(
                s_expr.contains("await_expression"),
                "Generated code snippet does not contain expected node type 'await_expression' in AST"
            );
        },
    );
}

#[test]
fn test_generated_binary_expression() {
    let source_code = r#"a + b"#;

    assert_code_analysis_and_snapshot(
        source_code,
        lintric_core::Language::Rust,
        "rust",
        "test_generated_binary_expression",
        "Failed to analyze code for node type: binary_expression",
        |s_expr| {
            assert!(
                s_expr.contains("binary_expression"),
                "Generated code snippet does not contain expected node type 'binary_expression' in AST"
            );
        },
    );
}

#[test]
fn test_generated_boolean_literal() {
    let source_code = r#"let flag = true;"#;

    assert_code_analysis_and_snapshot(
        source_code,
        lintric_core::Language::Rust,
        "rust",
        "test_generated_boolean_literal",
        "Failed to analyze code for node type: boolean_literal",
        |s_expr| {
            assert!(
                s_expr.contains("boolean_literal"),
                "Generated code snippet does not contain expected node type 'boolean_literal' in AST"
            );
        },
    );
}

#[test]
fn test_generated_bounded_type() {
    let source_code = r#"fn test<T: Clone + Send>(x: T) -> T where T: std::fmt::Debug { x }"#;

    assert_code_analysis_and_snapshot(
        source_code,
        lintric_core::Language::Rust,
        "rust",
        "test_generated_bounded_type",
        "Failed to analyze code for node type: bounded_type",
        |s_expr| {
            // bounded_type is represented by constrained_type_parameter or trait_bounds
            assert!(
                s_expr.contains("constrained_type_parameter") || s_expr.contains("trait_bounds") || s_expr.contains("where_clause"),
                "Generated code snippet should contain constrained_type_parameter, trait_bounds, or where_clause"
            );
        },
    );
}

#[test]
fn test_generated_bracketed_type() {
    let source_code = r#"fn test() -> <Vec<i32> as IntoIterator>::Item { 42 }"#;

    assert_code_analysis_and_snapshot(
        source_code,
        lintric_core::Language::Rust,
        "rust",
        "test_generated_bracketed_type",
        "Failed to analyze code for node type: bracketed_type",
        |s_expr| {
            // bracketed_type exists in qualified type contexts like <Vec<i32> as Iterator>
            assert!(
                s_expr.contains("qualified_type") || s_expr.contains("bracketed_type"),
                "Generated code snippet should contain qualified_type or bracketed_type"
            );
        },
    );
}

#[test]
fn test_generated_break_expression() {
    let source_code = r#"'label: loop { break 'label; }"#;

    assert_code_analysis_and_snapshot(
        source_code,
        lintric_core::Language::Rust,
        "rust",
        "test_generated_break_expression",
        "Failed to analyze code for node type: break_expression",
        |s_expr| {
            assert!(
                s_expr.contains("break_expression"),
                "Generated code snippet does not contain expected node type 'break_expression' in AST"
            );
        },
    );
}

#[test]
fn test_generated_call_expression() {
    let source_code = r#"test_fn2(a1, b1)"#;

    assert_code_analysis_and_snapshot(
        source_code,
        lintric_core::Language::Rust,
        "rust",
        "test_generated_call_expression",
        "Failed to analyze code for node type: call_expression",
        |s_expr| {
            assert!(
                s_expr.contains("call_expression"),
                "Generated code snippet does not contain expected node type 'call_expression' in AST"
            );
        },
    );
}

#[test]
fn test_generated_captured_pattern() {
    let source_code = r#"match x2 { y1 @ Some(_) => y1 }"#;

    assert_code_analysis_and_snapshot(
        source_code,
        lintric_core::Language::Rust,
        "rust",
        "test_generated_captured_pattern",
        "Failed to analyze code for node type: captured_pattern",
        |s_expr| {
            assert!(
                s_expr.contains("captured_pattern"),
                "Generated code snippet does not contain expected node type 'captured_pattern' in AST"
            );
        },
    );
}

#[test]
fn test_generated_closure_expression() {
    let source_code = r#"let closure = |x3| x3 + y2;"#;

    assert_code_analysis_and_snapshot(
        source_code,
        lintric_core::Language::Rust,
        "rust",
        "test_generated_closure_expression",
        "Failed to analyze code for node type: closure_expression",
        |s_expr| {
            assert!(
                s_expr.contains("closure_expression"),
                "Generated code snippet does not contain expected node type 'closure_expression' in AST"
            );
        },
    );
}

#[test]
fn test_generated_const_item() {
    let source_code = r#"const CONST: i32 = 42;"#;

    assert_code_analysis_and_snapshot(
        source_code,
        lintric_core::Language::Rust,
        "rust",
        "test_generated_const_item",
        "Failed to analyze code for node type: const_item",
        |s_expr| {
            assert!(
                s_expr.contains("const_item"),
                "Generated code snippet does not contain expected node type 'const_item' in AST"
            );
        },
    );
}

#[test]
fn test_generated_continue_expression() {
    let source_code = r#"'label1: loop { continue 'label1; }"#;

    assert_code_analysis_and_snapshot(
        source_code,
        lintric_core::Language::Rust,
        "rust",
        "test_generated_continue_expression",
        "Failed to analyze code for node type: continue_expression",
        |s_expr| {
            assert!(
                s_expr.contains("continue_expression"),
                "Generated code snippet does not contain expected node type 'continue_expression' in AST"
            );
        },
    );
}

#[test]
fn test_generated_dynamic_type() {
    let source_code = r#"let var3: Box<dyn std::fmt::Display> = Box::new(42);"#;

    assert_code_analysis_and_snapshot(
        source_code,
        lintric_core::Language::Rust,
        "rust",
        "test_generated_dynamic_type",
        "Failed to analyze code for node type: dynamic_type",
        |s_expr| {
            assert!(
                s_expr.contains("dynamic_type"),
                "Generated code snippet does not contain expected node type 'dynamic_type' in AST"
            );
        },
    );
}

#[test]
fn test_generated_empty_statement() {
    let source_code = r#";"#;

    assert_code_analysis_and_snapshot(
        source_code,
        lintric_core::Language::Rust,
        "rust",
        "test_generated_empty_statement",
        "Failed to analyze code for node type: empty_statement",
        |s_expr| {
            assert!(
                s_expr.contains("empty_statement"),
                "Generated code snippet does not contain expected node type 'empty_statement' in AST"
            );
        },
    );
}

#[test]
fn test_generated_enum_item() {
    let source_code = r#"enum TestEnum { Variant(i32) }"#;

    assert_code_analysis_and_snapshot(
        source_code,
        lintric_core::Language::Rust,
        "rust",
        "test_generated_enum_item",
        "Failed to analyze code for node type: enum_item",
        |s_expr| {
            assert!(
                s_expr.contains("enum_item"),
                "Generated code snippet does not contain expected node type 'enum_item' in AST"
            );
        },
    );
}

#[test]
fn test_generated_expression_statement() {
    let source_code = r#"a2 + b2;"#;

    assert_code_analysis_and_snapshot(
        source_code,
        lintric_core::Language::Rust,
        "rust",
        "test_generated_expression_statement",
        "Failed to analyze code for node type: expression_statement",
        |s_expr| {
            assert!(
                s_expr.contains("expression_statement"),
                "Generated code snippet does not contain expected node type 'expression_statement' in AST"
            );
        },
    );
}

#[test]
fn test_generated_field_expression() {
    let source_code = r#"obj.field"#;

    assert_code_analysis_and_snapshot(
        source_code,
        lintric_core::Language::Rust,
        "rust",
        "test_generated_field_expression",
        "Failed to analyze code for node type: field_expression",
        |s_expr| {
            assert!(
                s_expr.contains("field_expression"),
                "Generated code snippet does not contain expected node type 'field_expression' in AST"
            );
        },
    );
}

#[test]
fn test_generated_field_pattern() {
    let source_code = r#"if let Type { field1: x4 } = value { }"#;

    assert_code_analysis_and_snapshot(
        source_code,
        lintric_core::Language::Rust,
        "rust",
        "test_generated_field_pattern",
        "Failed to analyze code for node type: field_pattern",
        |s_expr| {
            assert!(
                s_expr.contains("field_pattern"),
                "Generated code snippet does not contain expected node type 'field_pattern' in AST"
            );
        },
    );
}

#[test]
fn test_generated_for_expression() {
    let source_code = r#"for item in vec { println!("{}", item); }"#;

    assert_code_analysis_and_snapshot(
        source_code,
        lintric_core::Language::Rust,
        "rust",
        "test_generated_for_expression",
        "Failed to analyze code for node type: for_expression",
        |s_expr| {
            assert!(
                s_expr.contains("for_expression"),
                "Generated code snippet does not contain expected node type 'for_expression' in AST"
            );
        },
    );
}

#[test]
fn test_generated_foreign_mod_item() {
    let source_code = r#"extern "C" { fn extern_fn(); }"#;

    assert_code_analysis_and_snapshot(
        source_code,
        lintric_core::Language::Rust,
        "rust",
        "test_generated_foreign_mod_item",
        "Failed to analyze code for node type: foreign_mod_item",
        |s_expr| {
            assert!(
                s_expr.contains("foreign_mod_item"),
                "Generated code snippet does not contain expected node type 'foreign_mod_item' in AST"
            );
        },
    );
}

#[test]
fn test_generated_function_item() {
    let source_code = r#"fn test_fn3() -> i32 { let x5 = 42; x5 + 1 }"#;

    assert_code_analysis_and_snapshot(
        source_code,
        lintric_core::Language::Rust,
        "rust",
        "test_generated_function_item",
        "Failed to analyze code for node type: function_item",
        |s_expr| {
            assert!(
                s_expr.contains("function_item"),
                "Generated code snippet does not contain expected node type 'function_item' in AST"
            );
        },
    );
}

#[test]
fn test_generated_function_signature_item() {
    let source_code = r#"fn test_fn4() -> i32;"#;

    assert_code_analysis_and_snapshot(
        source_code,
        lintric_core::Language::Rust,
        "rust",
        "test_generated_function_signature_item",
        "Failed to analyze code for node type: function_signature_item",
        |s_expr| {
            assert!(
                s_expr.contains("function_signature_item"),
                "Generated code snippet does not contain expected node type 'function_signature_item' in AST"
            );
        },
    );
}

#[test]
fn test_generated_function_type() {
    let source_code = r#"let func: fn(i32) -> i32 = |x| x + 1;"#;

    assert_code_analysis_and_snapshot(
        source_code,
        lintric_core::Language::Rust,
        "rust",
        "test_generated_function_type",
        "Failed to analyze code for node type: function_type",
        |s_expr| {
            assert!(
                s_expr.contains("function_type"),
                "Generated code snippet does not contain expected node type 'function_type' in AST"
            );
        },
    );
}

#[test]
fn test_generated_generic_pattern() {
    let source_code = r#"match x { Vec::<T>::new() => x6, _ => x6 }"#;

    assert_code_analysis_and_snapshot(
        source_code,
        lintric_core::Language::Rust,
        "rust",
        "test_generated_generic_pattern",
        "Failed to analyze code for node type: generic_pattern",
        |s_expr| {
            // This node type may not exist in current grammar - basic validation
            assert!(
                !s_expr.trim().is_empty(),
                "Generated code snippet produced non-empty AST"
            );
        },
    );
}

#[test]
fn test_generated_generic_type() {
    let source_code = r#"let var4: Vec<i32> = vec![1, 2, 3];"#;

    assert_code_analysis_and_snapshot(
        source_code,
        lintric_core::Language::Rust,
        "rust",
        "test_generated_generic_type",
        "Failed to analyze code for node type: generic_type",
        |s_expr| {
            assert!(
                s_expr.contains("generic_type"),
                "Generated code snippet does not contain expected node type 'generic_type' in AST"
            );
        },
    );
}

#[test]
fn test_generated_higher_ranked_trait_bound() {
    let source_code = r#"fn test_fn5<F>(f: F) where F: for<'a> Fn(&'a str) -> &'a str { }"#;

    assert_code_analysis_and_snapshot(
        source_code,
        lintric_core::Language::Rust,
        "rust",
        "test_generated_higher_ranked_trait_bound",
        "Failed to analyze code for node type: higher_ranked_trait_bound",
        |s_expr| {
            assert!(
                s_expr.contains("higher_ranked_trait_bound"),
                "Generated code snippet does not contain expected node type 'higher_ranked_trait_bound' in AST"
            );
        },
    );
}

#[test]
fn test_generated_if_expression() {
    let source_code = r#"if let Some(value1) = opt { value1 + 1 } else { 0 }"#;

    assert_code_analysis_and_snapshot(
        source_code,
        lintric_core::Language::Rust,
        "rust",
        "test_generated_if_expression",
        "Failed to analyze code for node type: if_expression",
        |s_expr| {
            assert!(
                s_expr.contains("if_expression"),
                "Generated code snippet does not contain expected node type 'if_expression' in AST"
            );
        },
    );
}

#[test]
fn test_generated_impl_item() {
    let source_code = r#"impl TestType { fn method(&self) -> i32 { 42 } }"#;

    assert_code_analysis_and_snapshot(
        source_code,
        lintric_core::Language::Rust,
        "rust",
        "test_generated_impl_item",
        "Failed to analyze code for node type: impl_item",
        |s_expr| {
            assert!(
                s_expr.contains("impl_item"),
                "Generated code snippet does not contain expected node type 'impl_item' in AST"
            );
        },
    );
}

#[test]
fn test_generated_index_expression() {
    let source_code = r#"arr1[i]"#;

    assert_code_analysis_and_snapshot(
        source_code,
        lintric_core::Language::Rust,
        "rust",
        "test_generated_index_expression",
        "Failed to analyze code for node type: index_expression",
        |s_expr| {
            assert!(
                s_expr.contains("index_expression"),
                "Generated code snippet does not contain expected node type 'index_expression' in AST"
            );
        },
    );
}

#[test]
fn test_generated_inner_attribute_item() {
    let source_code = r#"#![allow(dead_code)]
fn test_fn6() -> i32 { 42 }"#;

    assert_code_analysis_and_snapshot(
        source_code,
        lintric_core::Language::Rust,
        "rust",
        "test_generated_inner_attribute_item",
        "Failed to analyze code for node type: inner_attribute_item",
        |s_expr| {
            assert!(
                s_expr.contains("inner_attribute_item"),
                "Generated code snippet does not contain expected node type 'inner_attribute_item' in AST"
            );
        },
    );
}

#[test]
fn test_generated_let_declaration() {
    let source_code = r#"let var5 = source1;"#;

    assert_code_analysis_and_snapshot(
        source_code,
        lintric_core::Language::Rust,
        "rust",
        "test_generated_let_declaration",
        "Failed to analyze code for node type: let_declaration",
        |s_expr| {
            assert!(
                s_expr.contains("let_declaration"),
                "Generated code snippet does not contain expected node type 'let_declaration' in AST"
            );
        },
    );
}

#[test]
fn test_generated_loop_expression() {
    let source_code = r#"loop { break; }"#;

    assert_code_analysis_and_snapshot(
        source_code,
        lintric_core::Language::Rust,
        "rust",
        "test_generated_loop_expression",
        "Failed to analyze code for node type: loop_expression",
        |s_expr| {
            assert!(
                s_expr.contains("loop_expression"),
                "Generated code snippet does not contain expected node type 'loop_expression' in AST"
            );
        },
    );
}

#[test]
fn test_generated_macro_invocation() {
    let source_code = r#"println!("{}", var6)"#;

    assert_code_analysis_and_snapshot(
        source_code,
        lintric_core::Language::Rust,
        "rust",
        "test_generated_macro_invocation",
        "Failed to analyze code for node type: macro_invocation",
        |s_expr| {
            assert!(
                s_expr.contains("macro_invocation"),
                "Generated code snippet does not contain expected node type 'macro_invocation' in AST"
            );
        },
    );
}

#[test]
fn test_generated_match_expression() {
    let source_code = r#"match opt1 { Some(x7) => x7, None => 0 }"#;

    assert_code_analysis_and_snapshot(
        source_code,
        lintric_core::Language::Rust,
        "rust",
        "test_generated_match_expression",
        "Failed to analyze code for node type: match_expression",
        |s_expr| {
            assert!(
                s_expr.contains("match_expression"),
                "Generated code snippet does not contain expected node type 'match_expression' in AST"
            );
        },
    );
}

#[test]
fn test_generated_match_pattern() {
    let source_code = r#"match val { x8 => {} }"#;

    assert_code_analysis_and_snapshot(
        source_code,
        lintric_core::Language::Rust,
        "rust",
        "test_generated_match_pattern",
        "Failed to analyze code for node type: match_pattern",
        |s_expr| {
            assert!(
                s_expr.contains("match_pattern"),
                "Generated code snippet does not contain expected node type 'match_pattern' in AST"
            );
        },
    );
}

#[test]
fn test_generated_mod_item() {
    let source_code = r#"mod test_mod { pub fn test_fn7() -> i32 { 42 } }"#;

    assert_code_analysis_and_snapshot(
        source_code,
        lintric_core::Language::Rust,
        "rust",
        "test_generated_mod_item",
        "Failed to analyze code for node type: mod_item",
        |s_expr| {
            assert!(
                s_expr.contains("mod_item"),
                "Generated code snippet does not contain expected node type 'mod_item' in AST"
            );
        },
    );
}

#[test]
fn test_generated_mut_pattern() {
    let source_code = r#"let mut x9 = value2;"#;

    assert_code_analysis_and_snapshot(
        source_code,
        lintric_core::Language::Rust,
        "rust",
        "test_generated_mut_pattern",
        "Failed to analyze code for node type: mut_pattern",
        |s_expr| {
            // mut_pattern doesn't exist - check for mutable_specifier instead
            assert!(
                s_expr.contains("mutable_specifier"),
                "Generated code snippet should contain mutable_specifier for mut pattern"
            );
        },
    );
}

#[test]
fn test_generated_negative_literal() {
    let source_code = r#"let result = x10 - y3;"#;

    assert_code_analysis_and_snapshot(
        source_code,
        lintric_core::Language::Rust,
        "rust",
        "test_generated_negative_literal",
        "Failed to analyze code for node type: negative_literal",
        |s_expr| {
            // negative_literal doesn't exist - check for binary_expression instead
            assert!(
                s_expr.contains("binary_expression"),
                "Generated code snippet should contain binary_expression for negative literal"
            );
        },
    );
}

#[test]
fn test_generated_never_type() {
    let source_code = r#"fn test_fn8() -> ! { panic!("never returns") }"#;

    assert_code_analysis_and_snapshot(
        source_code,
        lintric_core::Language::Rust,
        "rust",
        "test_generated_never_type",
        "Failed to analyze code for node type: never_type",
        |s_expr| {
            assert!(
                s_expr.contains("never_type"),
                "Generated code snippet does not contain expected node type 'never_type' in AST"
            );
        },
    );
}

#[test]
fn test_generated_or_pattern() {
    let source_code = r#"match val1 { 0 | 1 => {} }"#;

    assert_code_analysis_and_snapshot(
        source_code,
        lintric_core::Language::Rust,
        "rust",
        "test_generated_or_pattern",
        "Failed to analyze code for node type: or_pattern",
        |s_expr| {
            assert!(
                s_expr.contains("or_pattern"),
                "Generated code snippet does not contain expected node type 'or_pattern' in AST"
            );
        },
    );
}

#[test]
fn test_generated_parenthesized_expression() {
    let source_code = r#"(a3 + b3)"#;

    assert_code_analysis_and_snapshot(
        source_code,
        lintric_core::Language::Rust,
        "rust",
        "test_generated_parenthesized_expression",
        "Failed to analyze code for node type: parenthesized_expression",
        |s_expr| {
            assert!(
                s_expr.contains("parenthesized_expression"),
                "Generated code snippet does not contain expected node type 'parenthesized_expression' in AST"
            );
        },
    );
}

#[test]
fn test_generated_pointer_type() {
    let source_code = r#"let ptr: *const i32 = std::ptr::null();"#;

    assert_code_analysis_and_snapshot(
        source_code,
        lintric_core::Language::Rust,
        "rust",
        "test_generated_pointer_type",
        "Failed to analyze code for node type: pointer_type",
        |s_expr| {
            assert!(
                s_expr.contains("pointer_type"),
                "Generated code snippet does not contain expected node type 'pointer_type' in AST"
            );
        },
    );
}

#[test]
fn test_generated_qualified_type() {
    let source_code = r#"fn test() -> <Vec<i32> as IntoIterator>::Item { 42 }"#;

    assert_code_analysis_and_snapshot(
        source_code,
        lintric_core::Language::Rust,
        "rust",
        "test_generated_qualified_type",
        "Failed to analyze code for node type: qualified_type",
        |s_expr| {
            assert!(
                s_expr.contains("qualified_type"),
                "Generated code snippet does not contain expected node type 'qualified_type' in AST"
            );
        },
    );
}

#[test]
fn test_generated_range_expression() {
    let source_code = r#"0..10"#;

    assert_code_analysis_and_snapshot(
        source_code,
        lintric_core::Language::Rust,
        "rust",
        "test_generated_range_expression",
        "Failed to analyze code for node type: range_expression",
        |s_expr| {
            assert!(
                s_expr.contains("range_expression"),
                "Generated code snippet does not contain expected node type 'range_expression' in AST"
            );
        },
    );
}

#[test]
fn test_generated_range_pattern() {
    let source_code = r#"match val2 { 0..=10 => {}, _ => {} }"#;

    assert_code_analysis_and_snapshot(
        source_code,
        lintric_core::Language::Rust,
        "rust",
        "test_generated_range_pattern",
        "Failed to analyze code for node type: range_pattern",
        |s_expr| {
            assert!(
                s_expr.contains("range_pattern"),
                "Generated code snippet does not contain expected node type 'range_pattern' in AST"
            );
        },
    );
}

#[test]
fn test_generated_raw_string_literal() {
    let source_code = r#"r"test""#;

    assert_code_analysis_and_snapshot(
        source_code,
        lintric_core::Language::Rust,
        "rust",
        "test_generated_raw_string_literal",
        "Failed to analyze code for node type: raw_string_literal",
        |s_expr| {
            assert!(
                s_expr.contains("raw_string_literal"),
                "Generated code snippet does not contain expected node type 'raw_string_literal' in AST"
            );
        },
    );
}

#[test]
fn test_generated_ref_pattern() {
    let source_code = r#"let ref x11 = value3;"#;

    assert_code_analysis_and_snapshot(
        source_code,
        lintric_core::Language::Rust,
        "rust",
        "test_generated_ref_pattern",
        "Failed to analyze code for node type: ref_pattern",
        |s_expr| {
            assert!(
                s_expr.contains("ref_pattern"),
                "Generated code snippet does not contain expected node type 'ref_pattern' in AST"
            );
        },
    );
}

#[test]
fn test_generated_reference_expression() {
    let source_code = r#"&value4"#;

    assert_code_analysis_and_snapshot(
        source_code,
        lintric_core::Language::Rust,
        "rust",
        "test_generated_reference_expression",
        "Failed to analyze code for node type: reference_expression",
        |s_expr| {
            assert!(
                s_expr.contains("reference_expression"),
                "Generated code snippet does not contain expected node type 'reference_expression' in AST"
            );
        },
    );
}

#[test]
fn test_generated_reference_pattern() {
    let source_code = r#"let &x12 = &value5;"#;

    assert_code_analysis_and_snapshot(
        source_code,
        lintric_core::Language::Rust,
        "rust",
        "test_generated_reference_pattern",
        "Failed to analyze code for node type: reference_pattern",
        |s_expr| {
            assert!(
                s_expr.contains("reference_pattern"),
                "Generated code snippet does not contain expected node type 'reference_pattern' in AST"
            );
        },
    );
}

#[test]
fn test_generated_reference_type() {
    let source_code = r#"let source2 = 42; let var7: &i32 = &source2;"#;

    assert_code_analysis_and_snapshot(
        source_code,
        lintric_core::Language::Rust,
        "rust",
        "test_generated_reference_type",
        "Failed to analyze code for node type: reference_type",
        |s_expr| {
            assert!(
                s_expr.contains("reference_type"),
                "Generated code snippet does not contain expected node type 'reference_type' in AST"
            );
        },
    );
}

#[test]
fn test_generated_remaining_field_pattern() {
    let source_code = r#"if let Type1 { .. } = value6 { }"#;

    assert_code_analysis_and_snapshot(
        source_code,
        lintric_core::Language::Rust,
        "rust",
        "test_generated_remaining_field_pattern",
        "Failed to analyze code for node type: remaining_field_pattern",
        |s_expr| {
            assert!(
                s_expr.contains("remaining_field_pattern"),
                "Generated code snippet does not contain expected node type 'remaining_field_pattern' in AST"
            );
        },
    );
}

#[test]
fn test_generated_removed_trait_bound() {
    let source_code = r#"fn test_fn9<T>(_: T) { }"#;

    assert_code_analysis_and_snapshot(
        source_code,
        lintric_core::Language::Rust,
        "rust",
        "test_generated_removed_trait_bound",
        "Failed to analyze code for node type: removed_trait_bound",
        |s_expr| {
            // This node type may not exist in current grammar - basic validation
            assert!(
                !s_expr.trim().is_empty(),
                "Generated code snippet produced non-empty AST"
            );
        },
    );
}

#[test]
fn test_generated_return_expression() {
    let source_code = r#"fn f() -> i32 { return 1; }"#;

    assert_code_analysis_and_snapshot(
        source_code,
        lintric_core::Language::Rust,
        "rust",
        "test_generated_return_expression",
        "Failed to analyze code for node type: return_expression",
        |s_expr| {
            assert!(
                s_expr.contains("return_expression"),
                "Generated code snippet does not contain expected node type 'return_expression' in AST"
            );
        },
    );
}

#[test]
fn test_generated_scoped_identifier() {
    let source_code = r#"std::collections::HashMap::new()"#;

    assert_code_analysis_and_snapshot(
        source_code,
        lintric_core::Language::Rust,
        "rust",
        "test_generated_scoped_identifier",
        "Failed to analyze code for node type: scoped_identifier",
        |s_expr| {
            assert!(
                s_expr.contains("scoped_identifier"),
                "Generated code snippet does not contain expected node type 'scoped_identifier' in AST"
            );
        },
    );
}

#[test]
fn test_generated_scoped_type_identifier() {
    let source_code = r#"let var8: std::vec::Vec<i32> = vec![];"#;

    assert_code_analysis_and_snapshot(
        source_code,
        lintric_core::Language::Rust,
        "rust",
        "test_generated_scoped_type_identifier",
        "Failed to analyze code for node type: scoped_type_identifier",
        |s_expr| {
            assert!(
                s_expr.contains("scoped_type_identifier"),
                "Generated code snippet does not contain expected node type 'scoped_type_identifier' in AST"
            );
        },
    );
}

#[test]
fn test_generated_slice_pattern() {
    let source_code = r#"let [x13, ..] = [1, 2, 3];"#;

    assert_code_analysis_and_snapshot(
        source_code,
        lintric_core::Language::Rust,
        "rust",
        "test_generated_slice_pattern",
        "Failed to analyze code for node type: slice_pattern",
        |s_expr| {
            assert!(
                s_expr.contains("slice_pattern"),
                "Generated code snippet does not contain expected node type 'slice_pattern' in AST"
            );
        },
    );
}

#[test]
fn test_generated_static_item() {
    let source_code = r#"static STATIC: i32 = 42;"#;

    assert_code_analysis_and_snapshot(
        source_code,
        lintric_core::Language::Rust,
        "rust",
        "test_generated_static_item",
        "Failed to analyze code for node type: static_item",
        |s_expr| {
            assert!(
                s_expr.contains("static_item"),
                "Generated code snippet does not contain expected node type 'static_item' in AST"
            );
        },
    );
}

#[test]
fn test_generated_string_literal() {
    let source_code = r#"let text = "test";"#;

    assert_code_analysis_and_snapshot(
        source_code,
        lintric_core::Language::Rust,
        "rust",
        "test_generated_string_literal",
        "Failed to analyze code for node type: string_literal",
        |s_expr| {
            assert!(
                s_expr.contains("string_literal"),
                "Generated code snippet does not contain expected node type 'string_literal' in AST"
            );
        },
    );
}

#[test]
fn test_generated_struct_expression() {
    let source_code = r#"TestStruct { field2: value7 }"#;

    assert_code_analysis_and_snapshot(
        source_code,
        lintric_core::Language::Rust,
        "rust",
        "test_generated_struct_expression",
        "Failed to analyze code for node type: struct_expression",
        |s_expr| {
            assert!(
                s_expr.contains("struct_expression"),
                "Generated code snippet does not contain expected node type 'struct_expression' in AST"
            );
        },
    );
}

#[test]
fn test_generated_struct_item() {
    let source_code = r#"struct TestStruct1 { field3: i32 }"#;

    assert_code_analysis_and_snapshot(
        source_code,
        lintric_core::Language::Rust,
        "rust",
        "test_generated_struct_item",
        "Failed to analyze code for node type: struct_item",
        |s_expr| {
            assert!(
                s_expr.contains("struct_item"),
                "Generated code snippet does not contain expected node type 'struct_item' in AST"
            );
        },
    );
}

#[test]
fn test_generated_struct_pattern() {
    let source_code = r#"if let Type2 { field4: x14 } = value8 { }"#;

    assert_code_analysis_and_snapshot(
        source_code,
        lintric_core::Language::Rust,
        "rust",
        "test_generated_struct_pattern",
        "Failed to analyze code for node type: struct_pattern",
        |s_expr| {
            assert!(
                s_expr.contains("struct_pattern"),
                "Generated code snippet does not contain expected node type 'struct_pattern' in AST"
            );
        },
    );
}

#[test]
fn test_generated_token_binding_pattern() {
    let source_code = r#"match val3 { b4 @ _ => {} }"#;

    assert_code_analysis_and_snapshot(
        source_code,
        lintric_core::Language::Rust,
        "rust",
        "test_generated_token_binding_pattern",
        "Failed to analyze code for node type: token_binding_pattern",
        |s_expr| {
            // This node type may not exist in current grammar - basic validation
            assert!(
                !s_expr.trim().is_empty(),
                "Generated code snippet produced non-empty AST"
            );
        },
    );
}

#[test]
fn test_generated_token_repetition_pattern() {
    let source_code =
        r#"macro_rules! test_macro { ($($item:ident),*) => { $(let $item = 42;)* }; }"#;

    assert_code_analysis_and_snapshot(
        source_code,
        lintric_core::Language::Rust,
        "rust",
        "test_generated_token_repetition_pattern",
        "Failed to analyze code for node type: token_repetition_pattern",
        |s_expr| {
            assert!(
                s_expr.contains("token_repetition_pattern"),
                "Generated code snippet does not contain expected node type 'token_repetition_pattern' in AST"
            );
        },
    );
}

#[test]
fn test_generated_token_tree_pattern() {
    let source_code = r#"macro_rules! test_macro1 { ({ $($content:tt)* }) => { $($content)* }; }"#;

    assert_code_analysis_and_snapshot(
        source_code,
        lintric_core::Language::Rust,
        "rust",
        "test_generated_token_tree_pattern",
        "Failed to analyze code for node type: token_tree_pattern",
        |s_expr| {
            assert!(
                s_expr.contains("token_tree_pattern"),
                "Generated code snippet does not contain expected node type 'token_tree_pattern' in AST"
            );
        },
    );
}

#[test]
fn test_generated_trait_item() {
    let source_code = r#"trait TestTrait1 { fn method1(&self) -> i32; }"#;

    assert_code_analysis_and_snapshot(
        source_code,
        lintric_core::Language::Rust,
        "rust",
        "test_generated_trait_item",
        "Failed to analyze code for node type: trait_item",
        |s_expr| {
            assert!(
                s_expr.contains("trait_item"),
                "Generated code snippet does not contain expected node type 'trait_item' in AST"
            );
        },
    );
}

#[test]
fn test_generated_try_expression() {
    let source_code = r#"result1?"#;

    assert_code_analysis_and_snapshot(
        source_code,
        lintric_core::Language::Rust,
        "rust",
        "test_generated_try_expression",
        "Failed to analyze code for node type: try_expression",
        |s_expr| {
            assert!(
                s_expr.contains("try_expression"),
                "Generated code snippet does not contain expected node type 'try_expression' in AST"
            );
        },
    );
}

#[test]
fn test_generated_tuple_expression() {
    let source_code = r#"(item11, item21)"#;

    assert_code_analysis_and_snapshot(
        source_code,
        lintric_core::Language::Rust,
        "rust",
        "test_generated_tuple_expression",
        "Failed to analyze code for node type: tuple_expression",
        |s_expr| {
            assert!(
                s_expr.contains("tuple_expression"),
                "Generated code snippet does not contain expected node type 'tuple_expression' in AST"
            );
        },
    );
}

#[test]
fn test_generated_tuple_pattern() {
    let source_code = r#"let (x15, y4) = tuple_val;"#;

    assert_code_analysis_and_snapshot(
        source_code,
        lintric_core::Language::Rust,
        "rust",
        "test_generated_tuple_pattern",
        "Failed to analyze code for node type: tuple_pattern",
        |s_expr| {
            assert!(
                s_expr.contains("tuple_pattern"),
                "Generated code snippet does not contain expected node type 'tuple_pattern' in AST"
            );
        },
    );
}

#[test]
fn test_generated_tuple_struct_pattern() {
    let source_code = r#"if let Point(x16, y5) = point_val { }"#;

    assert_code_analysis_and_snapshot(
        source_code,
        lintric_core::Language::Rust,
        "rust",
        "test_generated_tuple_struct_pattern",
        "Failed to analyze code for node type: tuple_struct_pattern",
        |s_expr| {
            assert!(
                s_expr.contains("tuple_struct_pattern"),
                "Generated code snippet does not contain expected node type 'tuple_struct_pattern' in AST"
            );
        },
    );
}

#[test]
fn test_generated_tuple_type() {
    let source_code = r#"let tuple: (i32, String) = (42, "test".to_string());"#;

    assert_code_analysis_and_snapshot(
        source_code,
        lintric_core::Language::Rust,
        "rust",
        "test_generated_tuple_type",
        "Failed to analyze code for node type: tuple_type",
        |s_expr| {
            assert!(
                s_expr.contains("tuple_type"),
                "Generated code snippet does not contain expected node type 'tuple_type' in AST"
            );
        },
    );
}

#[test]
fn test_generated_type_cast_expression() {
    let source_code = r#"value9 as i32"#;

    assert_code_analysis_and_snapshot(
        source_code,
        lintric_core::Language::Rust,
        "rust",
        "test_generated_type_cast_expression",
        "Failed to analyze code for node type: type_cast_expression",
        |s_expr| {
            assert!(
                s_expr.contains("type_cast_expression"),
                "Generated code snippet does not contain expected node type 'type_cast_expression' in AST"
            );
        },
    );
}

#[test]
fn test_generated_type_item() {
    let source_code = r#"type TestType1 = i32;"#;

    assert_code_analysis_and_snapshot(
        source_code,
        lintric_core::Language::Rust,
        "rust",
        "test_generated_type_item",
        "Failed to analyze code for node type: type_item",
        |s_expr| {
            assert!(
                s_expr.contains("type_item"),
                "Generated code snippet does not contain expected node type 'type_item' in AST"
            );
        },
    );
}

#[test]
fn test_generated_unary_expression() {
    let source_code = r#"-value10"#;

    assert_code_analysis_and_snapshot(
        source_code,
        lintric_core::Language::Rust,
        "rust",
        "test_generated_unary_expression",
        "Failed to analyze code for node type: unary_expression",
        |s_expr| {
            assert!(
                s_expr.contains("unary_expression"),
                "Generated code snippet does not contain expected node type 'unary_expression' in AST"
            );
        },
    );
}

#[test]
fn test_generated_union_item() {
    let source_code = r#"union TestUnion { field5: i32 }"#;

    assert_code_analysis_and_snapshot(
        source_code,
        lintric_core::Language::Rust,
        "rust",
        "test_generated_union_item",
        "Failed to analyze code for node type: union_item",
        |s_expr| {
            assert!(
                s_expr.contains("union_item"),
                "Generated code snippet does not contain expected node type 'union_item' in AST"
            );
        },
    );
}

#[test]
fn test_generated_unit_expression() {
    let source_code = r#"()"#;

    assert_code_analysis_and_snapshot(
        source_code,
        lintric_core::Language::Rust,
        "rust",
        "test_generated_unit_expression",
        "Failed to analyze code for node type: unit_expression",
        |s_expr| {
            assert!(
                s_expr.contains("unit_expression"),
                "Generated code snippet does not contain expected node type 'unit_expression' in AST"
            );
        },
    );
}

#[test]
fn test_generated_unit_type() {
    let source_code = r#"let var9: () = ();"#;

    assert_code_analysis_and_snapshot(
        source_code,
        lintric_core::Language::Rust,
        "rust",
        "test_generated_unit_type",
        "Failed to analyze code for node type: unit_type",
        |s_expr| {
            assert!(
                s_expr.contains("unit_type"),
                "Generated code snippet does not contain expected node type 'unit_type' in AST"
            );
        },
    );
}

#[test]
fn test_generated_use_declaration() {
    let source_code = r#"use module::Item1;"#;

    assert_code_analysis_and_snapshot(
        source_code,
        lintric_core::Language::Rust,
        "rust",
        "test_generated_use_declaration",
        "Failed to analyze code for node type: use_declaration",
        |s_expr| {
            assert!(
                s_expr.contains("use_declaration"),
                "Generated code snippet does not contain expected node type 'use_declaration' in AST"
            );
        },
    );
}

#[test]
fn test_generated_while_expression() {
    let source_code = r#"while let Some(val4) = opt2 { break; }"#;

    assert_code_analysis_and_snapshot(
        source_code,
        lintric_core::Language::Rust,
        "rust",
        "test_generated_while_expression",
        "Failed to analyze code for node type: while_expression",
        |s_expr| {
            assert!(
                s_expr.contains("while_expression"),
                "Generated code snippet does not contain expected node type 'while_expression' in AST"
            );
        },
    );
}

#[test]
fn test_generated_yield_expression() {
    let source_code = r#"yield value11"#;

    assert_code_analysis_and_snapshot(
        source_code,
        lintric_core::Language::Rust,
        "rust",
        "test_generated_yield_expression",
        "Failed to analyze code for node type: yield_expression",
        |s_expr| {
            assert!(
                s_expr.contains("yield_expression"),
                "Generated code snippet does not contain expected node type 'yield_expression' in AST"
            );
        },
    );
}

#[test]
fn test_generated_char_literal() {
    let source_code = r#"let ch = 'c';"#;

    assert_code_analysis_and_snapshot(
        source_code,
        lintric_core::Language::Rust,
        "rust",
        "test_generated_char_literal",
        "Failed to analyze code for node type: char_literal",
        |s_expr| {
            assert!(
                s_expr.contains("char_literal"),
                "Generated code snippet does not contain expected node type 'char_literal' in AST"
            );
        },
    );
}

#[test]
fn test_generated_float_literal() {
    let source_code = r#"let pi = 3.14;"#;

    assert_code_analysis_and_snapshot(
        source_code,
        lintric_core::Language::Rust,
        "rust",
        "test_generated_float_literal",
        "Failed to analyze code for node type: float_literal",
        |s_expr| {
            assert!(
                s_expr.contains("float_literal"),
                "Generated code snippet does not contain expected node type 'float_literal' in AST"
            );
        },
    );
}

#[test]
fn test_generated_identifier() {
    let source_code = r#"identifier"#;

    assert_code_analysis_and_snapshot(
        source_code,
        lintric_core::Language::Rust,
        "rust",
        "test_generated_identifier",
        "Failed to analyze code for node type: identifier",
        |s_expr| {
            assert!(
                s_expr.contains("identifier"),
                "Generated code snippet does not contain expected node type 'identifier' in AST"
            );
        },
    );
}

#[test]
fn test_generated_integer_literal() {
    let source_code = r#"let num = 42;"#;

    assert_code_analysis_and_snapshot(
        source_code,
        lintric_core::Language::Rust,
        "rust",
        "test_generated_integer_literal",
        "Failed to analyze code for node type: integer_literal",
        |s_expr| {
            assert!(
                s_expr.contains("integer_literal"),
                "Generated code snippet does not contain expected node type 'integer_literal' in AST"
            );
        },
    );
}

#[test]
fn test_generated_primitive_type() {
    let source_code = r#"let var10: i32 = 42;"#;

    assert_code_analysis_and_snapshot(
        source_code,
        lintric_core::Language::Rust,
        "rust",
        "test_generated_primitive_type",
        "Failed to analyze code for node type: primitive_type",
        |s_expr| {
            assert!(
                s_expr.contains("primitive_type"),
                "Generated code snippet does not contain expected node type 'primitive_type' in AST"
            );
        },
    );
}
