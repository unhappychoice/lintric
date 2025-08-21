use crate::{
    language_plugin::{LanguageInfo, LanguagePlugin},
    GenerationContext,
};
use lintric_core::models::language::Language;

pub struct TypeScriptPlugin;

impl LanguagePlugin for TypeScriptPlugin {
    fn language(&self) -> Language {
        Language::TypeScript
    }

    fn language_info(&self) -> LanguageInfo {
        LanguageInfo {
            enum_variant: "lintric_core::Language::TypeScript",
            folder_name: "ts",
            display_name: "TypeScript",
        }
    }

    fn generate_snippet(&self, node_type: &str, context: &mut GenerationContext) -> Option<String> {
        self.generate_typescript_snippet(node_type, context)
    }

    fn generate_node_type_validation(&self, node_type: &str) -> String {
        // Handle abstract node types (starting with _) by checking for their concrete implementations
        if node_type.starts_with('_') {
            self.generate_abstract_node_validation(node_type)
        } else {
            // Handle specific TypeScript node types
            match node_type {
                "primary_expression" => r#"
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
            );"#
                .to_string(),
                "primary_type" => r#"
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
            );"#
                .to_string(),
                "pattern" => r#"
            // pattern is abstract - check for any subtype
            assert!(
                s_expr.contains("identifier")
                    || s_expr.contains("array_pattern")
                    || s_expr.contains("object_pattern")
                    || s_expr.contains("member_expression")
                    || s_expr.contains("non_null_expression"),
                "Generated code snippet does not contain any pattern subtype in AST"
            );"#
                .to_string(),
                "statement" => r#"
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
            );"#
                .to_string(),
                "expression" => r#"
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
            );"#
                .to_string(),
                "declaration" => r#"
            // declaration is abstract - check for any subtype
            assert!(
                s_expr.contains("lexical_declaration")
                    || s_expr.contains("function_declaration")
                    || s_expr.contains("class_declaration")
                    || s_expr.contains("interface_declaration")
                    || s_expr.contains("type_alias_declaration")
                    || s_expr.contains("enum_declaration"),
                "Generated code snippet does not contain any declaration subtype in AST"
            );"#
                .to_string(),
                _ => {
                    format!(
                        r#"
            assert!(
                s_expr.contains("{}"),
                "Generated code snippet does not contain expected node type '{}' in AST"
            );"#,
                        node_type, node_type
                    )
                }
            }
        }
    }
}

impl TypeScriptPlugin {
    fn generate_typescript_snippet(
        &self,
        node_type: &str,
        context: &mut GenerationContext,
    ) -> Option<String> {
        match node_type {
            // Complex declarations with dependencies
            "function_declaration" => {
                let fn_name = context.get_unique_name("testFn");
                let param_name = context.get_unique_name("param");
                let var_name = context.get_unique_name("local");
                Some(format!(
                    "function {}({}: number): number {{ const {} = {}; return {} + 1; }}",
                    fn_name, param_name, var_name, param_name, var_name
                ))
            }
            "class_declaration" => {
                let class_name = context.get_unique_name("TestClass");
                let field_name = context.get_unique_name("field");
                let method_name = context.get_unique_name("method");
                Some(format!(
                    "class {} {{ private {}: number = 42; public {}(): number {{ return this.{}; }} }}",
                    class_name, field_name, method_name, field_name
                ))
            }
            "abstract_class_declaration" => {
                let class_name = context.get_unique_name("TestClass");
                let method_name = context.get_unique_name("method");
                Some(format!(
                    "abstract class {} {{ abstract {}(): void; }}",
                    class_name, method_name
                ))
            }
            "ambient_declaration" => {
                let var_name = context.get_unique_name("value");
                Some(format!("declare const {}: string;", var_name))
            }
            "enum_declaration" => {
                let enum_name = context.get_unique_name("TestEnum");
                let member1 = context.get_unique_name("A");
                let member2 = context.get_unique_name("B");
                Some(format!("enum {} {{ {}, {} }}", enum_name, member1, member2))
            }
            "interface_declaration" => {
                let interface_name = context.get_unique_name("TestInterface");
                let prop_name = context.get_unique_name("prop");
                let method_name = context.get_unique_name("method");
                Some(format!(
                    "interface {} {{ {}: number; {}(): void; }}",
                    interface_name, prop_name, method_name
                ))
            }
            "variable_declaration" => {
                let var_name = context.get_unique_name("variable");
                let source_name = context.get_unique_name("source");
                Some(format!("var {} = {};", var_name, source_name))
            }

            // Import/export with dependencies
            "import_statement" => {
                let item_name = context.get_unique_name("Item");
                let module_name = context.get_unique_name("module");
                Some(format!(
                    "import {{ {} }} from './{}'",
                    item_name, module_name
                ))
            }
            "export_statement" => {
                let var_name = context.get_unique_name("exported");
                let value_name = context.get_unique_name("value");
                Some(format!("export const {} = {};", var_name, value_name))
            }

            // Complex expressions with dependencies
            "call_expression" => {
                let fn_name = context.get_unique_name("testFn");
                let arg1 = context.get_unique_name("a");
                let arg2 = context.get_unique_name("b");
                Some(format!("{}({}, {})", fn_name, arg1, arg2))
            }
            "arrow_function" => {
                let param = context.get_unique_name("x");
                let var_name = context.get_unique_name("y");
                Some(format!("({}: number) => {} + {}", param, param, var_name))
            }
            "function_expression" => {
                let param = context.get_unique_name("param");
                let local_var = context.get_unique_name("local");
                Some(format!(
                    "function({}: number) {{ const {} = {}; return {} + 1; }}",
                    param, local_var, param, local_var
                ))
            }
            "assignment_expression" => {
                let var_name = context.get_unique_name("x");
                let source_name = context.get_unique_name("y");
                Some(format!("{} = {}", var_name, source_name))
            }
            "binary_expression" => {
                let left = context.get_unique_name("a");
                let right = context.get_unique_name("b");
                Some(format!("{} + {}", left, right))
            }
            "member_expression" => {
                let obj_name = context.get_unique_name("obj");
                let prop_name = context.get_unique_name("prop");
                Some(format!("{}.{}", obj_name, prop_name))
            }
            "subscript_expression" => {
                let array_name = context.get_unique_name("arr");
                let index_name = context.get_unique_name("i");
                Some(format!("{}[{}]", array_name, index_name))
            }
            "new_expression" => {
                let class_name = context.get_unique_name("TestClass");
                let arg_name = context.get_unique_name("arg");
                Some(format!("new {}({})", class_name, arg_name))
            }
            "await_expression" => {
                let promise_name = context.get_unique_name("promise");
                Some(format!("await {}", promise_name))
            }
            "as_expression" => {
                let value_name = context.get_unique_name("value");
                let type_name = context.get_unique_name("Type");
                Some(format!("{} as {}", value_name, type_name))
            }
            "primary_expression" => {
                // primary_expression is abstract - generate code that contains identifier (subtype)
                let var_name = context.get_unique_name("value");
                Some(var_name.to_string())
            }
            "primary_type" => {
                // primary_type is abstract - generate code that contains type_identifier (subtype)
                let type_name = context.get_unique_name("Type");
                let var_name = context.get_unique_name("value");
                Some(format!("const {}: {} = 42;", var_name, type_name))
            }

            // Control flow with patterns
            "if_statement" => {
                let condition_var = context.get_unique_name("condition");
                let then_var = context.get_unique_name("result");
                Some(format!(
                    "if ({}) {{ const {} = 42; }}",
                    condition_var, then_var
                ))
            }
            "for_statement" => {
                let iterator = context.get_unique_name("i");
                let array_name = context.get_unique_name("arr");
                let item_name = context.get_unique_name("item");
                Some(format!(
                    "for (let {} = 0; {} < {}.length; {}++) {{ const {} = {}[{}]; }}",
                    iterator, iterator, array_name, iterator, item_name, array_name, iterator
                ))
            }
            "try_statement" => {
                let var_name = context.get_unique_name("result");
                let error_name = context.get_unique_name("error");
                Some(format!(
                    "try {{ const {} = riskyOperation(); }} catch ({}) {{ console.log({}); }}",
                    var_name, error_name, error_name
                ))
            }

            // Object and array patterns with destructuring
            "object" => {
                let prop1_name = context.get_unique_name("prop1");
                let prop2_name = context.get_unique_name("prop2");
                let value1 = context.get_unique_name("value1");
                let value2 = context.get_unique_name("value2");
                Some(format!(
                    "{{ {}: {}, {}: {} }}",
                    prop1_name, value1, prop2_name, value2
                ))
            }
            "array" => {
                let item1 = context.get_unique_name("item1");
                let item2 = context.get_unique_name("item2");
                Some(format!("[{}, {}]", item1, item2))
            }
            "object_pattern" => {
                let prop1 = context.get_unique_name("prop1");
                let prop2 = context.get_unique_name("prop2");
                let object_name = context.get_unique_name("object");
                Some(format!(
                    "const {{ {}, {} }} = {};",
                    prop1, prop2, object_name
                ))
            }
            "array_pattern" => {
                let item1 = context.get_unique_name("a");
                let item2 = context.get_unique_name("b");
                let array_name = context.get_unique_name("array");
                Some(format!("const [{}, {}] = {};", item1, item2, array_name))
            }

            // Simple elements
            "number" => Some("42".to_string()),
            "string" => Some("\"test\"".to_string()),
            "true" => Some("true".to_string()),
            "false" => Some("false".to_string()),
            "identifier" => {
                let name = context.get_unique_name("identifier");
                Some(name)
            }

            // Abstract types with dependencies
            "expression" => {
                let var_name = context.get_unique_name("var");
                Some(format!("{} + 1", var_name))
            }
            "statement" => {
                let var_name = context.get_unique_name("var");
                let source_name = context.get_unique_name("source");
                Some(format!("const {} = {};", var_name, source_name))
            }
            "declaration" => {
                let name = context.get_unique_name("declaration");
                let value = context.get_unique_name("value");
                Some(format!("const {} = {};", name, value))
            }
            "pattern" => {
                let binding = context.get_unique_name("binding");
                Some(format!("{{ {} }}", binding))
            }

            // Additional complex TypeScript patterns
            "array_type" => {
                let var_name = context.get_unique_name("arr");
                Some(format!("const {}: number[] = [1, 2, 3];", var_name))
            }
            "assignment_pattern" => {
                let param1 = context.get_unique_name("first");
                let param2 = context.get_unique_name("second");
                Some(format!(
                    "function test([{} = 1, {} = 2] = []) {{ return {} + {}; }}",
                    param1, param2, param1, param2
                ))
            }
            "augmented_assignment_expression" => {
                let var_name = context.get_unique_name("x");
                let value_name = context.get_unique_name("y");
                Some(format!("{} += {}", var_name, value_name))
            }
            "break_statement" => Some("for (let i = 0; i < 10; i++) { break; }".to_string()),
            "conditional_type" => {
                let param_type = context.get_unique_name("T");
                let var_name = context.get_unique_name("value");
                Some(format!("type Test<{}> = {} extends string ? number : boolean; const {}: Test<string> = 42;", param_type, param_type, var_name))
            }
            "constructor_type" => {
                let var_name = context.get_unique_name("ctor");
                Some(format!(
                    "const {}: new () => object = class {{}};",
                    var_name
                ))
            }
            "continue_statement" => Some("for (let i = 0; i < 10; i++) { continue; }".to_string()),
            "debugger_statement" => Some("debugger;".to_string()),
            "default_type" => {
                let param = context.get_unique_name("T");
                let var_name = context.get_unique_name("value");
                Some(format!(
                    "type Test<{} = string> = {}; const {}: Test = 'default';",
                    param, param, var_name
                ))
            }
            "do_statement" => {
                let var_name = context.get_unique_name("i");
                Some(format!(
                    "let {} = 0; do {{ {}++; }} while ({} < 5);",
                    var_name, var_name, var_name
                ))
            }
            "empty_statement" => Some(";".to_string()),
            "existential_type" => {
                let var_name = context.get_unique_name("val");
                Some(format!("const {}: * = 42;", var_name))
            }
            "expression_statement" => {
                let var_name = context.get_unique_name("value");
                Some(format!("{};", var_name))
            }
            "flow_maybe_type" => {
                let var_name = context.get_unique_name("val");
                Some(format!("const {}: ?string = null;", var_name))
            }
            "for_in_statement" => {
                let key = context.get_unique_name("key");
                let obj = context.get_unique_name("obj");
                Some(format!(
                    "for (const {} in {}) {{ console.log({}); }}",
                    key, obj, key
                ))
            }
            "function_type" => {
                let var_name = context.get_unique_name("fn");
                Some(format!(
                    "const {}: (x: number) => string = (x) => x.toString();",
                    var_name
                ))
            }
            "generator_function_declaration" => {
                let fn_name = context.get_unique_name("testFn");
                let value_name = context.get_unique_name("value");
                Some(format!(
                    "function* {}() {{ yield {}; }}",
                    fn_name, value_name
                ))
            }
            "generic_type" => {
                let var_name = context.get_unique_name("value");
                Some(format!("const {}: Array<number> = [1, 2, 3];", var_name))
            }
            "infer_type" => {
                let param = context.get_unique_name("T");
                let var_name = context.get_unique_name("value");
                Some(format!(
                    "type Test<{}> = {} extends infer U ? U : never; const {}: Test<string> = 'test';",
                    param, param, var_name
                ))
            }
            "instantiation_expression" => {
                // instantiation_expression may be handled as other constructs in parser
                context.mark_excluded(node_type);
                None
            }
            "intersection_type" => {
                let var_name = context.get_unique_name("value");
                Some(format!(
                    "const {}: {{ a: number }} & {{ b: string }} = {{ a: 42, b: 'test' }};",
                    var_name
                ))
            }
            "labeled_statement" => {
                let label = context.get_unique_name("label");
                Some(format!(
                    "{}: for (let i = 0; i < 10; i++) {{ break {}; }}",
                    label, label
                ))
            }
            "lexical_declaration" => {
                let var_name = context.get_unique_name("variable");
                let source_name = context.get_unique_name("source");
                Some(format!("const {} = {};", var_name, source_name))
            }
            "literal_type" => {
                let var_name = context.get_unique_name("value");
                Some(format!("const {}: 'hello' = 'hello';", var_name))
            }
            "lookup_type" => {
                let var_name = context.get_unique_name("value");
                Some(format!(
                    "type Obj = {{ key: string }}; const {}: Obj['key'] = 'test';",
                    var_name
                ))
            }
            "non_null_expression" => {
                let value = context.get_unique_name("value");
                Some(format!("{}!", value))
            }
            "object_assignment_pattern" => {
                let prop = context.get_unique_name("prop");
                let default_val = context.get_unique_name("defaultVal");
                let obj = context.get_unique_name("obj");
                Some(format!("const {{ {} = {} }} = {};", prop, default_val, obj))
            }
            "object_type" => {
                let var_name = context.get_unique_name("value");
                let prop = context.get_unique_name("prop");
                Some(format!(
                    "const {}: {{ {}: number }} = {{ {}: 42 }};",
                    var_name, prop, prop
                ))
            }
            "pair_pattern" => {
                let key = context.get_unique_name("key");
                let value = context.get_unique_name("value");
                let obj = context.get_unique_name("obj");
                Some(format!("const {{ {}: {} }} = {};", key, value, obj))
            }
            "parenthesized_expression" => {
                let value = context.get_unique_name("value");
                Some(format!("({})", value))
            }
            "parenthesized_type" => {
                let var_name = context.get_unique_name("value");
                Some(format!("const {}: (string | number) = 'test';", var_name))
            }
            "predefined_type" => {
                let var_name = context.get_unique_name("value");
                Some(format!("const {}: string = 'test';", var_name))
            }
            "readonly_type" => {
                let var_name = context.get_unique_name("value");
                Some(format!(
                    "const {}: readonly number[] = [1, 2, 3];",
                    var_name
                ))
            }
            "rest_pattern" => {
                let rest_var = context.get_unique_name("rest");
                let arr = context.get_unique_name("arr");
                Some(format!("const [first, ...{}] = {};", rest_var, arr))
            }
            "rest_type" => {
                let type_name = context.get_unique_name("RestTuple");
                Some(format!("type {} = [string, ...number[]];", type_name))
            }
            "return_statement" => {
                let value = context.get_unique_name("value");
                Some(format!("function test() {{ return {}; }}", value))
            }
            "satisfies_expression" => {
                let value = context.get_unique_name("value");
                Some(format!("{} satisfies string", value))
            }
            "sequence_expression" => {
                let var1 = context.get_unique_name("a");
                let var2 = context.get_unique_name("b");
                Some(format!("{}, {}", var1, var2))
            }
            "switch_statement" => {
                let value = context.get_unique_name("value");
                Some(format!(
                    "switch ({}) {{ case 1: break; default: break; }}",
                    value
                ))
            }
            "template_literal_type" => {
                let var_name = context.get_unique_name("value");
                let param = context.get_unique_name("T");
                Some(format!(
                    "type Test<{}> = `hello ${{{}}}!`; const {}: Test<'world'> = 'hello world!';",
                    param, param, var_name
                ))
            }
            "template_type" => {
                let var_name = context.get_unique_name("value");
                Some(format!(
                    "const {}: `hello ${{string}}` = 'hello world';",
                    var_name
                ))
            }
            "ternary_expression" => {
                let condition = context.get_unique_name("condition");
                let true_val = context.get_unique_name("trueVal");
                let false_val = context.get_unique_name("falseVal");
                Some(format!("{} ? {} : {}", condition, true_val, false_val))
            }
            "throw_statement" => {
                let error = context.get_unique_name("error");
                Some(format!("throw new Error({});", error))
            }
            "tuple_type" => {
                let var_name = context.get_unique_name("value");
                Some(format!(
                    "const {}: [string, number] = ['hello', 42];",
                    var_name
                ))
            }
            "type_alias_declaration" => {
                let type_name = context.get_unique_name("TestType");
                Some(format!("type {} = string;", type_name))
            }
            "unary_expression" => {
                let value = context.get_unique_name("value");
                Some(format!("!{}", value))
            }
            "union_type" => {
                let var_name = context.get_unique_name("value");
                Some(format!("const {}: string | number = 'test';", var_name))
            }
            "update_expression" => {
                let var_name = context.get_unique_name("counter");
                Some(format!("{}++", var_name))
            }
            "while_statement" => {
                let condition = context.get_unique_name("condition");
                Some(format!("while ({}) {{ break; }}", condition))
            }
            "with_statement" => {
                let obj = context.get_unique_name("obj");
                let prop = context.get_unique_name("prop");
                Some(format!("with ({}) {{ {}; }}", obj, prop))
            }
            "yield_expression" => {
                let value = context.get_unique_name("value");
                Some(format!("yield {}", value))
            }
            "regex_pattern" => {
                let var_name = context.get_unique_name("regex");
                Some(format!("const {} = /pattern/g;", var_name))
            }
            "shorthand_property_identifier_pattern" => {
                let prop = context.get_unique_name("prop");
                let obj = context.get_unique_name("obj");
                Some(format!("const {{ {} }} = {};", prop, obj))
            }
            "this_type" => {
                let class_name = context.get_unique_name("TestClass");
                let method_name = context.get_unique_name("method");
                Some(format!(
                    "class {} {{ {}(): this {{ return this; }} }}",
                    class_name, method_name
                ))
            }
            "number1" => Some("42".to_string()),
            "string1" => Some("\"test\"".to_string()),
            "object1" => {
                let prop_name = context.get_unique_name("prop");
                let value = context.get_unique_name("value");
                Some(format!("{{ {}: {} }}", prop_name, value))
            }
            "optional_type" => {
                // optional_type is handled as union_type | undefined or other constructs in current parser
                context.mark_excluded(node_type);
                None
            }

            // Pattern-based fallbacks
            _ => {
                if node_type.starts_with("primary_") {
                    panic!("No specific code generator implemented for primary node '{}'. Primary expressions may reference undefined variables.", node_type)
                } else if node_type.ends_with("_expression") {
                    panic!("No specific code generator implemented for expression node '{}'. Expressions need proper context and may reference undefined variables.", node_type)
                } else if node_type.ends_with("_statement") {
                    panic!("No specific code generator implemented for statement node '{}'. Statements need proper context and may reference undefined variables.", node_type)
                } else if node_type.ends_with("_declaration") {
                    // Declarations are generally safe as they define something
                    let name = context.get_unique_name("item");
                    let value = context.get_unique_name("value");
                    Some(format!("const {} = {};", name, value))
                } else if node_type.ends_with("_pattern") {
                    panic!("No specific code generator implemented for pattern node '{}'. Patterns are context-dependent and need proper implementation.", node_type)
                } else if node_type.ends_with("_type") {
                    panic!("Cannot generate valid standalone code for type node '{}'. Type nodes need context to be meaningful.", node_type)
                } else if node_type.ends_with("_literal") {
                    // Literals are generally safe standalone
                    Some("42".to_string())
                } else {
                    context.mark_excluded(node_type)
                }
            }
        }
    }

    fn generate_abstract_node_validation(&self, abstract_node: &str) -> String {
        format!(
            r#"
                // Unknown abstract node {} - basic validation
                assert!(
                    !s_expr.trim().is_empty(),
                    "Generated code snippet produced empty AST"
                );"#,
            abstract_node
        )
    }
}
