// Generated tests for TSX node types
// This file is auto-generated. Do not edit manually.

use lintric_core;

// Excluded node types (could not generate snippets):
// !
// !=
// !==
// "
// ${
// %
// %=
// &
// &&
// &&=
// &=
// '
// (
// )
// *
// **
// **=
// *=
// +
// ++
// +=
// +?:
// ,
// -
// --
// -=
// -?:
// .
// ...
// /
// /=
// />
// :
// ;
// <
// </
// <<
// <<=
// <=
// =
// ==
// ===
// =>
// >
// >=
// >>
// >>=
// >>>
// >>>=
// ?
// ?.
// ?:
// ??
// ??=
// @
// [
// ]
// ^
// ^=
// `
// abstract
// abstract_method_signature
// accessibility_modifier
// accessor
// adding_type_annotation
// any
// arguments
// as
// assert
// asserts
// asserts_annotation
// async
// await
// boolean
// break
// call_signature
// case
// catch
// catch_clause
// class
// class_body
// class_heritage
// class_static_block
// comment
// computed_property_name
// const
// constraint
// construct_signature
// continue
// debugger
// declare
// decorator
// default
// delete
// do
// else
// else_clause
// enum
// enum_assignment
// enum_body
// escape_sequence
// export
// export_clause
// export_specifier
// extends
// extends_clause
// extends_type_clause
// finally
// finally_clause
// for
// formal_parameters
// from
// function
// function_signature
// generator_function
// get
// global
// hash_bang_line
// html_character_reference
// html_comment
// if
// implements
// implements_clause
// import
// import_alias
// import_attribute
// import_clause
// import_require_clause
// import_specifier
// in
// index_signature
// index_type_query
// infer
// instanceof
// instantiation_expression
// interface
// interface_body
// internal_module
// is
// jsx_namespace_name
// jsx_text
// keyof
// let
// mapped_type_clause
// meta
// meta_property
// method_definition
// method_signature
// module
// named_imports
// namespace
// namespace_export
// namespace_import
// nested_identifier
// nested_type_identifier
// never
// new
// null
// of
// omitting_type_annotation
// opting_type_annotation
// optional_chain
// optional_parameter
// optional_type
// override
// override_modifier
// pair
// private
// private_property_identifier
// program
// property_identifier
// property_signature
// protected
// public
// public_field_definition
// readonly
// regex
// regex_flags
// require
// required_parameter
// return
// satisfies
// set
// shorthand_property_identifier
// spread_element
// statement_block
// statement_identifier
// static
// string_fragment
// super
// switch
// switch_body
// switch_case
// switch_default
// symbol
// target
// template_string
// template_substitution
// this
// throw
// try
// type
// type_annotation
// type_arguments
// type_identifier
// type_parameter
// type_parameters
// type_predicate
// type_predicate_annotation
// type_query
// typeof
// undefined
// unique symbol
// unknown
// using
// var
// variable_declarator
// void
// while
// with
// yield
// {
// {|
// |
// |=
// ||
// ||=
// |}
// }
// ~

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
fn test_generated_declaration() {
    let source_code = r#"const declaration = value;"#;

    assert_code_analysis_and_snapshot(
        source_code,
        lintric_core::Language::TSX,
        "tsx",
        "test_generated_declaration",
        "Failed to analyze code for node type: declaration",
        |s_expr| {
            // declaration is abstract - check for any subtype
            assert!(
                s_expr.contains("lexical_declaration")
                    || s_expr.contains("function_declaration")
                    || s_expr.contains("class_declaration")
                    || s_expr.contains("interface_declaration")
                    || s_expr.contains("type_alias_declaration")
                    || s_expr.contains("enum_declaration"),
                "Generated code snippet does not contain any declaration subtype in AST"
            );
        },
    );
}

#[test]
fn test_generated_expression() {
    let source_code = r#"var + 1"#;

    assert_code_analysis_and_snapshot(
        source_code,
        lintric_core::Language::TSX,
        "tsx",
        "test_generated_expression",
        "Failed to analyze code for node type: expression",
        |s_expr| {
            // expression is abstract - check for any subtype
            assert!(
                s_expr.contains("identifier")
                    || s_expr.contains("binary_expression")
                    || s_expr.contains("call_expression")
                    || s_expr.contains("assignment_expression")
                    || s_expr.contains("member_expression")
                    || s_expr.contains("number")
                    || s_expr.contains("string")
                    || s_expr.contains("array")
                    || s_expr.contains("object"),
                "Generated code snippet does not contain any expression subtype in AST"
            );
        },
    );
}

#[test]
fn test_generated_pattern() {
    let source_code = r#"{ binding }"#;

    assert_code_analysis_and_snapshot(
        source_code,
        lintric_core::Language::TSX,
        "tsx",
        "test_generated_pattern",
        "Failed to analyze code for node type: pattern",
        |s_expr| {
            // pattern is abstract - check for any subtype
            assert!(
                s_expr.contains("identifier")
                    || s_expr.contains("array_pattern")
                    || s_expr.contains("object_pattern")
                    || s_expr.contains("member_expression")
                    || s_expr.contains("non_null_expression"),
                "Generated code snippet does not contain any pattern subtype in AST"
            );
        },
    );
}

#[test]
fn test_generated_primary_expression() {
    let source_code = r#"value1"#;

    assert_code_analysis_and_snapshot(
        source_code,
        lintric_core::Language::TSX,
        "tsx",
        "test_generated_primary_expression",
        "Failed to analyze code for node type: primary_expression",
        |s_expr| {
            // primary_expression is abstract - check for any subtype
            assert!(
                s_expr.contains("identifier")
                    || s_expr.contains("number")
                    || s_expr.contains("string")
                    || s_expr.contains("array")
                    || s_expr.contains("object")
                    || s_expr.contains("call_expression")
                    || s_expr.contains("member_expression")
                    || s_expr.contains("true")
                    || s_expr.contains("false"),
                "Generated code snippet does not contain any primary_expression subtype in AST"
            );
        },
    );
}

#[test]
fn test_generated_primary_type() {
    let source_code = r#"const value2: Type = 42;"#;

    assert_code_analysis_and_snapshot(
        source_code,
        lintric_core::Language::TSX,
        "tsx",
        "test_generated_primary_type",
        "Failed to analyze code for node type: primary_type",
        |s_expr| {
            // primary_type is abstract - check for any subtype
            assert!(
                s_expr.contains("type_identifier")
                    || s_expr.contains("predefined_type")
                    || s_expr.contains("array_type")
                    || s_expr.contains("object_type")
                    || s_expr.contains("union_type")
                    || s_expr.contains("literal_type")
                    || s_expr.contains("generic_type")
                    || s_expr.contains("tuple_type"),
                "Generated code snippet does not contain any primary_type subtype in AST"
            );
        },
    );
}

#[test]
fn test_generated_statement() {
    let source_code = r#"const var1 = source;"#;

    assert_code_analysis_and_snapshot(
        source_code,
        lintric_core::Language::TSX,
        "tsx",
        "test_generated_statement",
        "Failed to analyze code for node type: statement",
        |s_expr| {
            // statement is abstract - check for any subtype
            assert!(
                s_expr.contains("expression_statement")
                    || s_expr.contains("lexical_declaration")
                    || s_expr.contains("if_statement")
                    || s_expr.contains("for_statement")
                    || s_expr.contains("while_statement")
                    || s_expr.contains("return_statement")
                    || s_expr.contains("break_statement")
                    || s_expr.contains("continue_statement"),
                "Generated code snippet does not contain any statement subtype in AST"
            );
        },
    );
}

#[test]
fn test_generated_abstract_class_declaration() {
    let source_code = r#"abstract class TestClass { abstract method(): void; }"#;

    assert_code_analysis_and_snapshot(
        source_code,
        lintric_core::Language::TSX,
        "tsx",
        "test_generated_abstract_class_declaration",
        "Failed to analyze code for node type: abstract_class_declaration",
        |s_expr| {
            assert!(
                s_expr.contains("abstract_class_declaration"),
                "Generated code snippet does not contain expected node type 'abstract_class_declaration' in AST"
            );
        },
    );
}

#[test]
fn test_generated_ambient_declaration() {
    let source_code = r#"declare const value3: string;"#;

    assert_code_analysis_and_snapshot(
        source_code,
        lintric_core::Language::TSX,
        "tsx",
        "test_generated_ambient_declaration",
        "Failed to analyze code for node type: ambient_declaration",
        |s_expr| {
            assert!(
                s_expr.contains("ambient_declaration"),
                "Generated code snippet does not contain expected node type 'ambient_declaration' in AST"
            );
        },
    );
}

#[test]
fn test_generated_array() {
    let source_code = r#"[item1, item2]"#;

    assert_code_analysis_and_snapshot(
        source_code,
        lintric_core::Language::TSX,
        "tsx",
        "test_generated_array",
        "Failed to analyze code for node type: array",
        |s_expr| {
            assert!(
                s_expr.contains("array"),
                "Generated code snippet does not contain expected node type 'array' in AST"
            );
        },
    );
}

#[test]
fn test_generated_array_pattern() {
    let source_code = r#"const [a, b] = array;"#;

    assert_code_analysis_and_snapshot(
        source_code,
        lintric_core::Language::TSX,
        "tsx",
        "test_generated_array_pattern",
        "Failed to analyze code for node type: array_pattern",
        |s_expr| {
            assert!(
                s_expr.contains("array_pattern"),
                "Generated code snippet does not contain expected node type 'array_pattern' in AST"
            );
        },
    );
}

#[test]
fn test_generated_array_type() {
    let source_code = r#"const arr: number[] = [1, 2, 3];"#;

    assert_code_analysis_and_snapshot(
        source_code,
        lintric_core::Language::TSX,
        "tsx",
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
fn test_generated_arrow_function() {
    let source_code = r#"(x: number) => x + y"#;

    assert_code_analysis_and_snapshot(
        source_code,
        lintric_core::Language::TSX,
        "tsx",
        "test_generated_arrow_function",
        "Failed to analyze code for node type: arrow_function",
        |s_expr| {
            assert!(
                s_expr.contains("arrow_function"),
                "Generated code snippet does not contain expected node type 'arrow_function' in AST"
            );
        },
    );
}

#[test]
fn test_generated_as_expression() {
    let source_code = r#"value4 as Type1"#;

    assert_code_analysis_and_snapshot(
        source_code,
        lintric_core::Language::TSX,
        "tsx",
        "test_generated_as_expression",
        "Failed to analyze code for node type: as_expression",
        |s_expr| {
            assert!(
                s_expr.contains("as_expression"),
                "Generated code snippet does not contain expected node type 'as_expression' in AST"
            );
        },
    );
}

#[test]
fn test_generated_assignment_expression() {
    let source_code = r#"x1 = y1"#;

    assert_code_analysis_and_snapshot(
        source_code,
        lintric_core::Language::TSX,
        "tsx",
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
fn test_generated_assignment_pattern() {
    let source_code = r#"function test([first = 1, second = 2] = []) { return first + second; }"#;

    assert_code_analysis_and_snapshot(
        source_code,
        lintric_core::Language::TSX,
        "tsx",
        "test_generated_assignment_pattern",
        "Failed to analyze code for node type: assignment_pattern",
        |s_expr| {
            assert!(
                s_expr.contains("assignment_pattern"),
                "Generated code snippet does not contain expected node type 'assignment_pattern' in AST"
            );
        },
    );
}

#[test]
fn test_generated_augmented_assignment_expression() {
    let source_code = r#"x2 += y2"#;

    assert_code_analysis_and_snapshot(
        source_code,
        lintric_core::Language::TSX,
        "tsx",
        "test_generated_augmented_assignment_expression",
        "Failed to analyze code for node type: augmented_assignment_expression",
        |s_expr| {
            assert!(
                s_expr.contains("augmented_assignment_expression"),
                "Generated code snippet does not contain expected node type 'augmented_assignment_expression' in AST"
            );
        },
    );
}

#[test]
fn test_generated_await_expression() {
    let source_code = r#"await promise"#;

    assert_code_analysis_and_snapshot(
        source_code,
        lintric_core::Language::TSX,
        "tsx",
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
    let source_code = r#"a1 + b1"#;

    assert_code_analysis_and_snapshot(
        source_code,
        lintric_core::Language::TSX,
        "tsx",
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
fn test_generated_break_statement() {
    let source_code = r#"for (let i = 0; i < 10; i++) { break; }"#;

    assert_code_analysis_and_snapshot(
        source_code,
        lintric_core::Language::TSX,
        "tsx",
        "test_generated_break_statement",
        "Failed to analyze code for node type: break_statement",
        |s_expr| {
            assert!(
                s_expr.contains("break_statement"),
                "Generated code snippet does not contain expected node type 'break_statement' in AST"
            );
        },
    );
}

#[test]
fn test_generated_call_expression() {
    let source_code = r#"testFn(a2, b2)"#;

    assert_code_analysis_and_snapshot(
        source_code,
        lintric_core::Language::TSX,
        "tsx",
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
fn test_generated_class_declaration() {
    let source_code = r#"class TestClass1 { private field: number = 42; public method1(): number { return this.field; } }"#;

    assert_code_analysis_and_snapshot(
        source_code,
        lintric_core::Language::TSX,
        "tsx",
        "test_generated_class_declaration",
        "Failed to analyze code for node type: class_declaration",
        |s_expr| {
            assert!(
                s_expr.contains("class_declaration"),
                "Generated code snippet does not contain expected node type 'class_declaration' in AST"
            );
        },
    );
}

#[test]
fn test_generated_conditional_type() {
    let source_code =
        r#"type Test<T> = T extends string ? number : boolean; const value5: Test<string> = 42;"#;

    assert_code_analysis_and_snapshot(
        source_code,
        lintric_core::Language::TSX,
        "tsx",
        "test_generated_conditional_type",
        "Failed to analyze code for node type: conditional_type",
        |s_expr| {
            assert!(
                s_expr.contains("conditional_type"),
                "Generated code snippet does not contain expected node type 'conditional_type' in AST"
            );
        },
    );
}

#[test]
fn test_generated_constructor_type() {
    let source_code = r#"const ctor: new () => object = class {};"#;

    assert_code_analysis_and_snapshot(
        source_code,
        lintric_core::Language::TSX,
        "tsx",
        "test_generated_constructor_type",
        "Failed to analyze code for node type: constructor_type",
        |s_expr| {
            assert!(
                s_expr.contains("constructor_type"),
                "Generated code snippet does not contain expected node type 'constructor_type' in AST"
            );
        },
    );
}

#[test]
fn test_generated_continue_statement() {
    let source_code = r#"for (let i = 0; i < 10; i++) { continue; }"#;

    assert_code_analysis_and_snapshot(
        source_code,
        lintric_core::Language::TSX,
        "tsx",
        "test_generated_continue_statement",
        "Failed to analyze code for node type: continue_statement",
        |s_expr| {
            assert!(
                s_expr.contains("continue_statement"),
                "Generated code snippet does not contain expected node type 'continue_statement' in AST"
            );
        },
    );
}

#[test]
fn test_generated_debugger_statement() {
    let source_code = r#"debugger;"#;

    assert_code_analysis_and_snapshot(
        source_code,
        lintric_core::Language::TSX,
        "tsx",
        "test_generated_debugger_statement",
        "Failed to analyze code for node type: debugger_statement",
        |s_expr| {
            assert!(
                s_expr.contains("debugger_statement"),
                "Generated code snippet does not contain expected node type 'debugger_statement' in AST"
            );
        },
    );
}

#[test]
fn test_generated_default_type() {
    let source_code = r#"type Test<T1 = string> = T1; const value6: Test = 'default';"#;

    assert_code_analysis_and_snapshot(
        source_code,
        lintric_core::Language::TSX,
        "tsx",
        "test_generated_default_type",
        "Failed to analyze code for node type: default_type",
        |s_expr| {
            assert!(
                s_expr.contains("default_type"),
                "Generated code snippet does not contain expected node type 'default_type' in AST"
            );
        },
    );
}

#[test]
fn test_generated_do_statement() {
    let source_code = r#"let i = 0; do { i++; } while (i < 5);"#;

    assert_code_analysis_and_snapshot(
        source_code,
        lintric_core::Language::TSX,
        "tsx",
        "test_generated_do_statement",
        "Failed to analyze code for node type: do_statement",
        |s_expr| {
            assert!(
                s_expr.contains("do_statement"),
                "Generated code snippet does not contain expected node type 'do_statement' in AST"
            );
        },
    );
}

#[test]
fn test_generated_empty_statement() {
    let source_code = r#";"#;

    assert_code_analysis_and_snapshot(
        source_code,
        lintric_core::Language::TSX,
        "tsx",
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
fn test_generated_enum_declaration() {
    let source_code = r#"enum TestEnum { A, B }"#;

    assert_code_analysis_and_snapshot(
        source_code,
        lintric_core::Language::TSX,
        "tsx",
        "test_generated_enum_declaration",
        "Failed to analyze code for node type: enum_declaration",
        |s_expr| {
            assert!(
                s_expr.contains("enum_declaration"),
                "Generated code snippet does not contain expected node type 'enum_declaration' in AST"
            );
        },
    );
}

#[test]
fn test_generated_existential_type() {
    let source_code = r#"const val: * = 42;"#;

    assert_code_analysis_and_snapshot(
        source_code,
        lintric_core::Language::TSX,
        "tsx",
        "test_generated_existential_type",
        "Failed to analyze code for node type: existential_type",
        |s_expr| {
            assert!(
                s_expr.contains("existential_type"),
                "Generated code snippet does not contain expected node type 'existential_type' in AST"
            );
        },
    );
}

#[test]
fn test_generated_export_statement() {
    let source_code = r#"export const exported = value7;"#;

    assert_code_analysis_and_snapshot(
        source_code,
        lintric_core::Language::TSX,
        "tsx",
        "test_generated_export_statement",
        "Failed to analyze code for node type: export_statement",
        |s_expr| {
            assert!(
                s_expr.contains("export_statement"),
                "Generated code snippet does not contain expected node type 'export_statement' in AST"
            );
        },
    );
}

#[test]
fn test_generated_expression_statement() {
    let source_code = r#"value8;"#;

    assert_code_analysis_and_snapshot(
        source_code,
        lintric_core::Language::TSX,
        "tsx",
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
fn test_generated_flow_maybe_type() {
    let source_code = r#"const val1: ?string = null;"#;

    assert_code_analysis_and_snapshot(
        source_code,
        lintric_core::Language::TSX,
        "tsx",
        "test_generated_flow_maybe_type",
        "Failed to analyze code for node type: flow_maybe_type",
        |s_expr| {
            assert!(
                s_expr.contains("flow_maybe_type"),
                "Generated code snippet does not contain expected node type 'flow_maybe_type' in AST"
            );
        },
    );
}

#[test]
fn test_generated_for_in_statement() {
    let source_code = r#"for (const key in obj) { console.log(key); }"#;

    assert_code_analysis_and_snapshot(
        source_code,
        lintric_core::Language::TSX,
        "tsx",
        "test_generated_for_in_statement",
        "Failed to analyze code for node type: for_in_statement",
        |s_expr| {
            assert!(
                s_expr.contains("for_in_statement"),
                "Generated code snippet does not contain expected node type 'for_in_statement' in AST"
            );
        },
    );
}

#[test]
fn test_generated_for_statement() {
    let source_code = r#"for (let i1 = 0; i1 < arr1.length; i1++) { const item = arr1[i1]; }"#;

    assert_code_analysis_and_snapshot(
        source_code,
        lintric_core::Language::TSX,
        "tsx",
        "test_generated_for_statement",
        "Failed to analyze code for node type: for_statement",
        |s_expr| {
            assert!(
                s_expr.contains("for_statement"),
                "Generated code snippet does not contain expected node type 'for_statement' in AST"
            );
        },
    );
}

#[test]
fn test_generated_function_declaration() {
    let source_code =
        r#"function testFn1(param: number): number { const local = param; return local + 1; }"#;

    assert_code_analysis_and_snapshot(
        source_code,
        lintric_core::Language::TSX,
        "tsx",
        "test_generated_function_declaration",
        "Failed to analyze code for node type: function_declaration",
        |s_expr| {
            assert!(
                s_expr.contains("function_declaration"),
                "Generated code snippet does not contain expected node type 'function_declaration' in AST"
            );
        },
    );
}

#[test]
fn test_generated_function_expression() {
    let source_code = r#"function(param1: number) { const local1 = param1; return local1 + 1; }"#;

    assert_code_analysis_and_snapshot(
        source_code,
        lintric_core::Language::TSX,
        "tsx",
        "test_generated_function_expression",
        "Failed to analyze code for node type: function_expression",
        |s_expr| {
            assert!(
                s_expr.contains("function_expression"),
                "Generated code snippet does not contain expected node type 'function_expression' in AST"
            );
        },
    );
}

#[test]
fn test_generated_function_type() {
    let source_code = r#"const fn: (x: number) => string = (x) => x.toString();"#;

    assert_code_analysis_and_snapshot(
        source_code,
        lintric_core::Language::TSX,
        "tsx",
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
fn test_generated_generator_function_declaration() {
    let source_code = r#"function* testFn2() { yield value9; }"#;

    assert_code_analysis_and_snapshot(
        source_code,
        lintric_core::Language::TSX,
        "tsx",
        "test_generated_generator_function_declaration",
        "Failed to analyze code for node type: generator_function_declaration",
        |s_expr| {
            assert!(
                s_expr.contains("generator_function_declaration"),
                "Generated code snippet does not contain expected node type 'generator_function_declaration' in AST"
            );
        },
    );
}

#[test]
fn test_generated_generic_type() {
    let source_code = r#"const value10: Array<number> = [1, 2, 3];"#;

    assert_code_analysis_and_snapshot(
        source_code,
        lintric_core::Language::TSX,
        "tsx",
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
fn test_generated_identifier() {
    let source_code = r#"identifier"#;

    assert_code_analysis_and_snapshot(
        source_code,
        lintric_core::Language::TSX,
        "tsx",
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
fn test_generated_if_statement() {
    let source_code = r#"if (condition) { const result = 42; }"#;

    assert_code_analysis_and_snapshot(
        source_code,
        lintric_core::Language::TSX,
        "tsx",
        "test_generated_if_statement",
        "Failed to analyze code for node type: if_statement",
        |s_expr| {
            assert!(
                s_expr.contains("if_statement"),
                "Generated code snippet does not contain expected node type 'if_statement' in AST"
            );
        },
    );
}

#[test]
fn test_generated_import_statement() {
    let source_code = r#"import { Item } from './module'"#;

    assert_code_analysis_and_snapshot(
        source_code,
        lintric_core::Language::TSX,
        "tsx",
        "test_generated_import_statement",
        "Failed to analyze code for node type: import_statement",
        |s_expr| {
            assert!(
                s_expr.contains("import_statement"),
                "Generated code snippet does not contain expected node type 'import_statement' in AST"
            );
        },
    );
}

#[test]
fn test_generated_infer_type() {
    let source_code =
        r#"type Test<T2> = T2 extends infer U ? U : never; const value11: Test<string> = 'test';"#;

    assert_code_analysis_and_snapshot(
        source_code,
        lintric_core::Language::TSX,
        "tsx",
        "test_generated_infer_type",
        "Failed to analyze code for node type: infer_type",
        |s_expr| {
            assert!(
                s_expr.contains("infer_type"),
                "Generated code snippet does not contain expected node type 'infer_type' in AST"
            );
        },
    );
}

#[test]
fn test_generated_interface_declaration() {
    let source_code = r#"interface TestInterface { prop: number; method2(): void; }"#;

    assert_code_analysis_and_snapshot(
        source_code,
        lintric_core::Language::TSX,
        "tsx",
        "test_generated_interface_declaration",
        "Failed to analyze code for node type: interface_declaration",
        |s_expr| {
            assert!(
                s_expr.contains("interface_declaration"),
                "Generated code snippet does not contain expected node type 'interface_declaration' in AST"
            );
        },
    );
}

#[test]
fn test_generated_intersection_type() {
    let source_code = r#"const value12: { a: number } & { b: string } = { a: 42, b: 'test' };"#;

    assert_code_analysis_and_snapshot(
        source_code,
        lintric_core::Language::TSX,
        "tsx",
        "test_generated_intersection_type",
        "Failed to analyze code for node type: intersection_type",
        |s_expr| {
            assert!(
                s_expr.contains("intersection_type"),
                "Generated code snippet does not contain expected node type 'intersection_type' in AST"
            );
        },
    );
}

#[test]
fn test_generated_jsx_attribute() {
    let source_code = r#"const element = <div prop1={value13}>Content</div>;"#;

    assert_code_analysis_and_snapshot(
        source_code,
        lintric_core::Language::TSX,
        "tsx",
        "test_generated_jsx_attribute",
        "Failed to analyze code for node type: jsx_attribute",
        |s_expr| {
            assert!(
                s_expr.contains("jsx_attribute"),
                "Generated code snippet does not contain expected node type 'jsx_attribute' in AST"
            );
        },
    );
}

#[test]
fn test_generated_jsx_closing_element() {
    let source_code = r#"const element = <Component>Content</Component>"#;

    assert_code_analysis_and_snapshot(
        source_code,
        lintric_core::Language::TSX,
        "tsx",
        "test_generated_jsx_closing_element",
        "Failed to analyze code for node type: jsx_closing_element",
        |s_expr| {
            assert!(
                s_expr.contains("jsx_closing_element"),
                "Generated code snippet does not contain expected node type 'jsx_closing_element' in AST"
            );
        },
    );
}

#[test]
fn test_generated_jsx_element() {
    let source_code = r#"const element = <div>content</div>;"#;

    assert_code_analysis_and_snapshot(
        source_code,
        lintric_core::Language::TSX,
        "tsx",
        "test_generated_jsx_element",
        "Failed to analyze code for node type: jsx_element",
        |s_expr| {
            assert!(
                s_expr.contains("jsx_element"),
                "Generated code snippet does not contain expected node type 'jsx_element' in AST"
            );
        },
    );
}

#[test]
fn test_generated_jsx_expression() {
    let source_code = r#"const element = <div>{value14}</div>;"#;

    assert_code_analysis_and_snapshot(
        source_code,
        lintric_core::Language::TSX,
        "tsx",
        "test_generated_jsx_expression",
        "Failed to analyze code for node type: jsx_expression",
        |s_expr| {
            assert!(
                s_expr.contains("jsx_expression"),
                "Generated code snippet does not contain expected node type 'jsx_expression' in AST"
            );
        },
    );
}

#[test]
fn test_generated_jsx_opening_element() {
    let source_code = r#"const element = <Component1 prop2={value15}></Component1>"#;

    assert_code_analysis_and_snapshot(
        source_code,
        lintric_core::Language::TSX,
        "tsx",
        "test_generated_jsx_opening_element",
        "Failed to analyze code for node type: jsx_opening_element",
        |s_expr| {
            assert!(
                s_expr.contains("jsx_opening_element"),
                "Generated code snippet does not contain expected node type 'jsx_opening_element' in AST"
            );
        },
    );
}

#[test]
fn test_generated_jsx_self_closing_element() {
    let source_code = r#"const element = <Component2 prop3={value16} />;"#;

    assert_code_analysis_and_snapshot(
        source_code,
        lintric_core::Language::TSX,
        "tsx",
        "test_generated_jsx_self_closing_element",
        "Failed to analyze code for node type: jsx_self_closing_element",
        |s_expr| {
            assert!(
                s_expr.contains("jsx_self_closing_element"),
                "Generated code snippet does not contain expected node type 'jsx_self_closing_element' in AST"
            );
        },
    );
}

#[test]
fn test_generated_labeled_statement() {
    let source_code = r#"label: for (let i = 0; i < 10; i++) { break label; }"#;

    assert_code_analysis_and_snapshot(
        source_code,
        lintric_core::Language::TSX,
        "tsx",
        "test_generated_labeled_statement",
        "Failed to analyze code for node type: labeled_statement",
        |s_expr| {
            assert!(
                s_expr.contains("labeled_statement"),
                "Generated code snippet does not contain expected node type 'labeled_statement' in AST"
            );
        },
    );
}

#[test]
fn test_generated_lexical_declaration() {
    let source_code = r#"const variable = source1;"#;

    assert_code_analysis_and_snapshot(
        source_code,
        lintric_core::Language::TSX,
        "tsx",
        "test_generated_lexical_declaration",
        "Failed to analyze code for node type: lexical_declaration",
        |s_expr| {
            assert!(
                s_expr.contains("lexical_declaration"),
                "Generated code snippet does not contain expected node type 'lexical_declaration' in AST"
            );
        },
    );
}

#[test]
fn test_generated_literal_type() {
    let source_code = r#"const value17: 'hello' = 'hello';"#;

    assert_code_analysis_and_snapshot(
        source_code,
        lintric_core::Language::TSX,
        "tsx",
        "test_generated_literal_type",
        "Failed to analyze code for node type: literal_type",
        |s_expr| {
            assert!(
                s_expr.contains("literal_type"),
                "Generated code snippet does not contain expected node type 'literal_type' in AST"
            );
        },
    );
}

#[test]
fn test_generated_lookup_type() {
    let source_code = r#"type Obj = { key: string }; const value18: Obj['key'] = 'test';"#;

    assert_code_analysis_and_snapshot(
        source_code,
        lintric_core::Language::TSX,
        "tsx",
        "test_generated_lookup_type",
        "Failed to analyze code for node type: lookup_type",
        |s_expr| {
            assert!(
                s_expr.contains("lookup_type"),
                "Generated code snippet does not contain expected node type 'lookup_type' in AST"
            );
        },
    );
}

#[test]
fn test_generated_member_expression() {
    let source_code = r#"obj1.prop4"#;

    assert_code_analysis_and_snapshot(
        source_code,
        lintric_core::Language::TSX,
        "tsx",
        "test_generated_member_expression",
        "Failed to analyze code for node type: member_expression",
        |s_expr| {
            assert!(
                s_expr.contains("member_expression"),
                "Generated code snippet does not contain expected node type 'member_expression' in AST"
            );
        },
    );
}

#[test]
fn test_generated_new_expression() {
    let source_code = r#"new TestClass2(arg)"#;

    assert_code_analysis_and_snapshot(
        source_code,
        lintric_core::Language::TSX,
        "tsx",
        "test_generated_new_expression",
        "Failed to analyze code for node type: new_expression",
        |s_expr| {
            assert!(
                s_expr.contains("new_expression"),
                "Generated code snippet does not contain expected node type 'new_expression' in AST"
            );
        },
    );
}

#[test]
fn test_generated_non_null_expression() {
    let source_code = r#"value19!"#;

    assert_code_analysis_and_snapshot(
        source_code,
        lintric_core::Language::TSX,
        "tsx",
        "test_generated_non_null_expression",
        "Failed to analyze code for node type: non_null_expression",
        |s_expr| {
            assert!(
                s_expr.contains("non_null_expression"),
                "Generated code snippet does not contain expected node type 'non_null_expression' in AST"
            );
        },
    );
}

#[test]
fn test_generated_object() {
    let source_code = r#"{ prop11: value110, prop21: value21 }"#;

    assert_code_analysis_and_snapshot(
        source_code,
        lintric_core::Language::TSX,
        "tsx",
        "test_generated_object",
        "Failed to analyze code for node type: object",
        |s_expr| {
            assert!(
                s_expr.contains("object"),
                "Generated code snippet does not contain expected node type 'object' in AST"
            );
        },
    );
}

#[test]
fn test_generated_object_assignment_pattern() {
    let source_code = r#"const { prop5 = defaultVal } = obj2;"#;

    assert_code_analysis_and_snapshot(
        source_code,
        lintric_core::Language::TSX,
        "tsx",
        "test_generated_object_assignment_pattern",
        "Failed to analyze code for node type: object_assignment_pattern",
        |s_expr| {
            assert!(
                s_expr.contains("object_assignment_pattern"),
                "Generated code snippet does not contain expected node type 'object_assignment_pattern' in AST"
            );
        },
    );
}

#[test]
fn test_generated_object_pattern() {
    let source_code = r#"const { prop12, prop22 } = object;"#;

    assert_code_analysis_and_snapshot(
        source_code,
        lintric_core::Language::TSX,
        "tsx",
        "test_generated_object_pattern",
        "Failed to analyze code for node type: object_pattern",
        |s_expr| {
            assert!(
                s_expr.contains("object_pattern"),
                "Generated code snippet does not contain expected node type 'object_pattern' in AST"
            );
        },
    );
}

#[test]
fn test_generated_object_type() {
    let source_code = r#"const value20: { prop6: number } = { prop6: 42 };"#;

    assert_code_analysis_and_snapshot(
        source_code,
        lintric_core::Language::TSX,
        "tsx",
        "test_generated_object_type",
        "Failed to analyze code for node type: object_type",
        |s_expr| {
            assert!(
                s_expr.contains("object_type"),
                "Generated code snippet does not contain expected node type 'object_type' in AST"
            );
        },
    );
}

#[test]
fn test_generated_pair_pattern() {
    let source_code = r#"const { key1: value22 } = obj3;"#;

    assert_code_analysis_and_snapshot(
        source_code,
        lintric_core::Language::TSX,
        "tsx",
        "test_generated_pair_pattern",
        "Failed to analyze code for node type: pair_pattern",
        |s_expr| {
            assert!(
                s_expr.contains("pair_pattern"),
                "Generated code snippet does not contain expected node type 'pair_pattern' in AST"
            );
        },
    );
}

#[test]
fn test_generated_parenthesized_expression() {
    let source_code = r#"(value23)"#;

    assert_code_analysis_and_snapshot(
        source_code,
        lintric_core::Language::TSX,
        "tsx",
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
fn test_generated_parenthesized_type() {
    let source_code = r#"const value24: (string | number) = 'test';"#;

    assert_code_analysis_and_snapshot(
        source_code,
        lintric_core::Language::TSX,
        "tsx",
        "test_generated_parenthesized_type",
        "Failed to analyze code for node type: parenthesized_type",
        |s_expr| {
            assert!(
                s_expr.contains("parenthesized_type"),
                "Generated code snippet does not contain expected node type 'parenthesized_type' in AST"
            );
        },
    );
}

#[test]
fn test_generated_predefined_type() {
    let source_code = r#"const value25: string = 'test';"#;

    assert_code_analysis_and_snapshot(
        source_code,
        lintric_core::Language::TSX,
        "tsx",
        "test_generated_predefined_type",
        "Failed to analyze code for node type: predefined_type",
        |s_expr| {
            assert!(
                s_expr.contains("predefined_type"),
                "Generated code snippet does not contain expected node type 'predefined_type' in AST"
            );
        },
    );
}

#[test]
fn test_generated_readonly_type() {
    let source_code = r#"const value26: readonly number[] = [1, 2, 3];"#;

    assert_code_analysis_and_snapshot(
        source_code,
        lintric_core::Language::TSX,
        "tsx",
        "test_generated_readonly_type",
        "Failed to analyze code for node type: readonly_type",
        |s_expr| {
            assert!(
                s_expr.contains("readonly_type"),
                "Generated code snippet does not contain expected node type 'readonly_type' in AST"
            );
        },
    );
}

#[test]
fn test_generated_rest_pattern() {
    let source_code = r#"const [first, ...rest] = arr2;"#;

    assert_code_analysis_and_snapshot(
        source_code,
        lintric_core::Language::TSX,
        "tsx",
        "test_generated_rest_pattern",
        "Failed to analyze code for node type: rest_pattern",
        |s_expr| {
            assert!(
                s_expr.contains("rest_pattern"),
                "Generated code snippet does not contain expected node type 'rest_pattern' in AST"
            );
        },
    );
}

#[test]
fn test_generated_rest_type() {
    let source_code = r#"type RestTuple = [string, ...number[]];"#;

    assert_code_analysis_and_snapshot(
        source_code,
        lintric_core::Language::TSX,
        "tsx",
        "test_generated_rest_type",
        "Failed to analyze code for node type: rest_type",
        |s_expr| {
            assert!(
                s_expr.contains("rest_type"),
                "Generated code snippet does not contain expected node type 'rest_type' in AST"
            );
        },
    );
}

#[test]
fn test_generated_return_statement() {
    let source_code = r#"function test() { return value27; }"#;

    assert_code_analysis_and_snapshot(
        source_code,
        lintric_core::Language::TSX,
        "tsx",
        "test_generated_return_statement",
        "Failed to analyze code for node type: return_statement",
        |s_expr| {
            assert!(
                s_expr.contains("return_statement"),
                "Generated code snippet does not contain expected node type 'return_statement' in AST"
            );
        },
    );
}

#[test]
fn test_generated_satisfies_expression() {
    let source_code = r#"value28 satisfies string"#;

    assert_code_analysis_and_snapshot(
        source_code,
        lintric_core::Language::TSX,
        "tsx",
        "test_generated_satisfies_expression",
        "Failed to analyze code for node type: satisfies_expression",
        |s_expr| {
            assert!(
                s_expr.contains("satisfies_expression"),
                "Generated code snippet does not contain expected node type 'satisfies_expression' in AST"
            );
        },
    );
}

#[test]
fn test_generated_sequence_expression() {
    let source_code = r#"a3, b3"#;

    assert_code_analysis_and_snapshot(
        source_code,
        lintric_core::Language::TSX,
        "tsx",
        "test_generated_sequence_expression",
        "Failed to analyze code for node type: sequence_expression",
        |s_expr| {
            assert!(
                s_expr.contains("sequence_expression"),
                "Generated code snippet does not contain expected node type 'sequence_expression' in AST"
            );
        },
    );
}

#[test]
fn test_generated_string() {
    let source_code = r#""test""#;

    assert_code_analysis_and_snapshot(
        source_code,
        lintric_core::Language::TSX,
        "tsx",
        "test_generated_string",
        "Failed to analyze code for node type: string",
        |s_expr| {
            assert!(
                s_expr.contains("string"),
                "Generated code snippet does not contain expected node type 'string' in AST"
            );
        },
    );
}

#[test]
fn test_generated_subscript_expression() {
    let source_code = r#"arr3[i2]"#;

    assert_code_analysis_and_snapshot(
        source_code,
        lintric_core::Language::TSX,
        "tsx",
        "test_generated_subscript_expression",
        "Failed to analyze code for node type: subscript_expression",
        |s_expr| {
            assert!(
                s_expr.contains("subscript_expression"),
                "Generated code snippet does not contain expected node type 'subscript_expression' in AST"
            );
        },
    );
}

#[test]
fn test_generated_switch_statement() {
    let source_code = r#"switch (value29) { case 1: break; default: break; }"#;

    assert_code_analysis_and_snapshot(
        source_code,
        lintric_core::Language::TSX,
        "tsx",
        "test_generated_switch_statement",
        "Failed to analyze code for node type: switch_statement",
        |s_expr| {
            assert!(
                s_expr.contains("switch_statement"),
                "Generated code snippet does not contain expected node type 'switch_statement' in AST"
            );
        },
    );
}

#[test]
fn test_generated_template_literal_type() {
    let source_code =
        r#"type Test<T3> = `hello ${T3}!`; const value30: Test<'world'> = 'hello world!';"#;

    assert_code_analysis_and_snapshot(
        source_code,
        lintric_core::Language::TSX,
        "tsx",
        "test_generated_template_literal_type",
        "Failed to analyze code for node type: template_literal_type",
        |s_expr| {
            assert!(
                s_expr.contains("template_literal_type"),
                "Generated code snippet does not contain expected node type 'template_literal_type' in AST"
            );
        },
    );
}

#[test]
fn test_generated_template_type() {
    let source_code = r#"const value31: `hello ${string}` = 'hello world';"#;

    assert_code_analysis_and_snapshot(
        source_code,
        lintric_core::Language::TSX,
        "tsx",
        "test_generated_template_type",
        "Failed to analyze code for node type: template_type",
        |s_expr| {
            assert!(
                s_expr.contains("template_type"),
                "Generated code snippet does not contain expected node type 'template_type' in AST"
            );
        },
    );
}

#[test]
fn test_generated_ternary_expression() {
    let source_code = r#"condition1 ? trueVal : falseVal"#;

    assert_code_analysis_and_snapshot(
        source_code,
        lintric_core::Language::TSX,
        "tsx",
        "test_generated_ternary_expression",
        "Failed to analyze code for node type: ternary_expression",
        |s_expr| {
            assert!(
                s_expr.contains("ternary_expression"),
                "Generated code snippet does not contain expected node type 'ternary_expression' in AST"
            );
        },
    );
}

#[test]
fn test_generated_throw_statement() {
    let source_code = r#"throw new Error(error);"#;

    assert_code_analysis_and_snapshot(
        source_code,
        lintric_core::Language::TSX,
        "tsx",
        "test_generated_throw_statement",
        "Failed to analyze code for node type: throw_statement",
        |s_expr| {
            assert!(
                s_expr.contains("throw_statement"),
                "Generated code snippet does not contain expected node type 'throw_statement' in AST"
            );
        },
    );
}

#[test]
fn test_generated_try_statement() {
    let source_code =
        r#"try { const result1 = riskyOperation(); } catch (error1) { console.log(error1); }"#;

    assert_code_analysis_and_snapshot(
        source_code,
        lintric_core::Language::TSX,
        "tsx",
        "test_generated_try_statement",
        "Failed to analyze code for node type: try_statement",
        |s_expr| {
            assert!(
                s_expr.contains("try_statement"),
                "Generated code snippet does not contain expected node type 'try_statement' in AST"
            );
        },
    );
}

#[test]
fn test_generated_tuple_type() {
    let source_code = r#"const value32: [string, number] = ['hello', 42];"#;

    assert_code_analysis_and_snapshot(
        source_code,
        lintric_core::Language::TSX,
        "tsx",
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
fn test_generated_type_alias_declaration() {
    let source_code = r#"type TestType = string;"#;

    assert_code_analysis_and_snapshot(
        source_code,
        lintric_core::Language::TSX,
        "tsx",
        "test_generated_type_alias_declaration",
        "Failed to analyze code for node type: type_alias_declaration",
        |s_expr| {
            assert!(
                s_expr.contains("type_alias_declaration"),
                "Generated code snippet does not contain expected node type 'type_alias_declaration' in AST"
            );
        },
    );
}

#[test]
fn test_generated_unary_expression() {
    let source_code = r#"!value33"#;

    assert_code_analysis_and_snapshot(
        source_code,
        lintric_core::Language::TSX,
        "tsx",
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
fn test_generated_union_type() {
    let source_code = r#"const value34: string | number = 'test';"#;

    assert_code_analysis_and_snapshot(
        source_code,
        lintric_core::Language::TSX,
        "tsx",
        "test_generated_union_type",
        "Failed to analyze code for node type: union_type",
        |s_expr| {
            assert!(
                s_expr.contains("union_type"),
                "Generated code snippet does not contain expected node type 'union_type' in AST"
            );
        },
    );
}

#[test]
fn test_generated_update_expression() {
    let source_code = r#"counter++"#;

    assert_code_analysis_and_snapshot(
        source_code,
        lintric_core::Language::TSX,
        "tsx",
        "test_generated_update_expression",
        "Failed to analyze code for node type: update_expression",
        |s_expr| {
            assert!(
                s_expr.contains("update_expression"),
                "Generated code snippet does not contain expected node type 'update_expression' in AST"
            );
        },
    );
}

#[test]
fn test_generated_variable_declaration() {
    let source_code = r#"var variable1 = source2;"#;

    assert_code_analysis_and_snapshot(
        source_code,
        lintric_core::Language::TSX,
        "tsx",
        "test_generated_variable_declaration",
        "Failed to analyze code for node type: variable_declaration",
        |s_expr| {
            assert!(
                s_expr.contains("variable_declaration"),
                "Generated code snippet does not contain expected node type 'variable_declaration' in AST"
            );
        },
    );
}

#[test]
fn test_generated_while_statement() {
    let source_code = r#"while (condition2) { break; }"#;

    assert_code_analysis_and_snapshot(
        source_code,
        lintric_core::Language::TSX,
        "tsx",
        "test_generated_while_statement",
        "Failed to analyze code for node type: while_statement",
        |s_expr| {
            assert!(
                s_expr.contains("while_statement"),
                "Generated code snippet does not contain expected node type 'while_statement' in AST"
            );
        },
    );
}

#[test]
fn test_generated_with_statement() {
    let source_code = r#"with (obj4) { prop7; }"#;

    assert_code_analysis_and_snapshot(
        source_code,
        lintric_core::Language::TSX,
        "tsx",
        "test_generated_with_statement",
        "Failed to analyze code for node type: with_statement",
        |s_expr| {
            assert!(
                s_expr.contains("with_statement"),
                "Generated code snippet does not contain expected node type 'with_statement' in AST"
            );
        },
    );
}

#[test]
fn test_generated_yield_expression() {
    let source_code = r#"yield value35"#;

    assert_code_analysis_and_snapshot(
        source_code,
        lintric_core::Language::TSX,
        "tsx",
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
fn test_generated_false() {
    let source_code = r#"false"#;

    assert_code_analysis_and_snapshot(
        source_code,
        lintric_core::Language::TSX,
        "tsx",
        "test_generated_false",
        "Failed to analyze code for node type: false",
        |s_expr| {
            assert!(
                s_expr.contains("false"),
                "Generated code snippet does not contain expected node type 'false' in AST"
            );
        },
    );
}

#[test]
fn test_generated_number() {
    let source_code = r#"42"#;

    assert_code_analysis_and_snapshot(
        source_code,
        lintric_core::Language::TSX,
        "tsx",
        "test_generated_number",
        "Failed to analyze code for node type: number",
        |s_expr| {
            assert!(
                s_expr.contains("number"),
                "Generated code snippet does not contain expected node type 'number' in AST"
            );
        },
    );
}

#[test]
fn test_generated_number1() {
    let source_code = r#"42"#;

    assert_code_analysis_and_snapshot(
        source_code,
        lintric_core::Language::TSX,
        "tsx",
        "test_generated_number1",
        "Failed to analyze code for node type: number",
        |s_expr| {
            assert!(
                s_expr.contains("number"),
                "Generated code snippet does not contain expected node type 'number' in AST"
            );
        },
    );
}

#[test]
fn test_generated_object1() {
    let source_code = r#"{ prop13: value111, prop23: value210 }"#;

    assert_code_analysis_and_snapshot(
        source_code,
        lintric_core::Language::TSX,
        "tsx",
        "test_generated_object1",
        "Failed to analyze code for node type: object",
        |s_expr| {
            assert!(
                s_expr.contains("object"),
                "Generated code snippet does not contain expected node type 'object' in AST"
            );
        },
    );
}

#[test]
fn test_generated_regex_pattern() {
    let source_code = r#"const regex = /pattern/g;"#;

    assert_code_analysis_and_snapshot(
        source_code,
        lintric_core::Language::TSX,
        "tsx",
        "test_generated_regex_pattern",
        "Failed to analyze code for node type: regex_pattern",
        |s_expr| {
            assert!(
                s_expr.contains("regex_pattern"),
                "Generated code snippet does not contain expected node type 'regex_pattern' in AST"
            );
        },
    );
}

#[test]
fn test_generated_shorthand_property_identifier_pattern() {
    let source_code = r#"const { prop8 } = obj5;"#;

    assert_code_analysis_and_snapshot(
        source_code,
        lintric_core::Language::TSX,
        "tsx",
        "test_generated_shorthand_property_identifier_pattern",
        "Failed to analyze code for node type: shorthand_property_identifier_pattern",
        |s_expr| {
            assert!(
                s_expr.contains("shorthand_property_identifier_pattern"),
                "Generated code snippet does not contain expected node type 'shorthand_property_identifier_pattern' in AST"
            );
        },
    );
}

#[test]
fn test_generated_string1() {
    let source_code = r#""test""#;

    assert_code_analysis_and_snapshot(
        source_code,
        lintric_core::Language::TSX,
        "tsx",
        "test_generated_string1",
        "Failed to analyze code for node type: string",
        |s_expr| {
            assert!(
                s_expr.contains("string"),
                "Generated code snippet does not contain expected node type 'string' in AST"
            );
        },
    );
}

#[test]
fn test_generated_this_type() {
    let source_code = r#"class TestClass3 { method3(): this { return this; } }"#;

    assert_code_analysis_and_snapshot(
        source_code,
        lintric_core::Language::TSX,
        "tsx",
        "test_generated_this_type",
        "Failed to analyze code for node type: this_type",
        |s_expr| {
            assert!(
                s_expr.contains("this_type"),
                "Generated code snippet does not contain expected node type 'this_type' in AST"
            );
        },
    );
}

#[test]
fn test_generated_true() {
    let source_code = r#"true"#;

    assert_code_analysis_and_snapshot(
        source_code,
        lintric_core::Language::TSX,
        "tsx",
        "test_generated_true",
        "Failed to analyze code for node type: true",
        |s_expr| {
            assert!(
                s_expr.contains("true"),
                "Generated code snippet does not contain expected node type 'true' in AST"
            );
        },
    );
}
