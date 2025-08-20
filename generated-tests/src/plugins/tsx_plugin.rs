use crate::{
    language_plugin::{LanguageInfo, LanguagePlugin},
    GenerationContext,
};
use lintric_core::models::language::Language;

pub struct TsxPlugin;

impl LanguagePlugin for TsxPlugin {
    fn language(&self) -> Language {
        Language::TSX
    }

    fn language_info(&self) -> LanguageInfo {
        LanguageInfo {
            enum_variant: "lintric_core::Language::TSX",
            folder_name: "tsx",
            display_name: "TSX",
        }
    }

    fn generate_snippet(&self, node_type: &str, context: &mut GenerationContext) -> Option<String> {
        self.generate_tsx_snippet(node_type, context)
    }

    fn generate_node_type_validation(&self, node_type: &str) -> String {
        // Handle abstract node types (starting with _) by checking for their concrete implementations
        if node_type.starts_with('_') {
            self.generate_abstract_node_validation(node_type)
        } else {
            // Handle specific TSX node types (same as TypeScript for most cases)
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

impl TsxPlugin {
    fn generate_tsx_snippet(
        &self,
        node_type: &str,
        context: &mut GenerationContext,
    ) -> Option<String> {
        match node_type {
            // JSX-specific elements with dependencies
            "jsx_element" => {
                let component_name = context.get_unique_name("div");
                let content = context.get_unique_name("content");
                Some(format!(
                    "const element = <{}>{}</{}>;",
                    component_name, content, component_name
                ))
            }
            "jsx_self_closing_element" => {
                let component_name = context.get_unique_name("Component");
                let prop_name = context.get_unique_name("prop");
                let value_name = context.get_unique_name("value");
                Some(format!(
                    "const element = <{} {}={{{}}} />;",
                    component_name, prop_name, value_name
                ))
            }
            "jsx_fragment" => {
                let content = context.get_unique_name("content");
                Some(format!("const fragment = <>{}</>;", content))
            }
            "jsx_expression" => {
                let var_name = context.get_unique_name("value");
                Some(format!("const element = <div>{{{}}}</div>;", var_name))
            }
            "jsx_attribute" => {
                let attr_name = context.get_unique_name("prop");
                let value_name = context.get_unique_name("value");
                Some(format!(
                    "const element = <div {}={{{}}}>Content</div>;",
                    attr_name, value_name
                ))
            }
            "jsx_opening_element" => {
                let component_name = context.get_unique_name("Component");
                let prop_name = context.get_unique_name("prop");
                let value_name = context.get_unique_name("value");
                Some(format!(
                    "const element = <{} {}={{{}}}></{}>",
                    component_name, prop_name, value_name, component_name
                ))
            }
            "jsx_closing_element" => {
                let component_name = context.get_unique_name("Component");
                Some(format!(
                    "const element = <{}>Content</{}>",
                    component_name, component_name
                ))
            }

            // React component patterns with dependencies
            "jsx_component" => {
                let component_name = context.get_unique_name("Component");
                let prop_name = context.get_unique_name("prop");
                let value_name = context.get_unique_name("value");
                Some(format!(
                    "<{} {}={{{}}} />",
                    component_name, prop_name, value_name
                ))
            }
            "function_component" => {
                let component_name = context.get_unique_name("TestComponent");
                let prop_name = context.get_unique_name("prop");
                let local_var = context.get_unique_name("local");
                Some(format!(
                    "function {}({{ {} }}: {{ {}: string }}) {{ const {} = {}; return <div>{{{}}} content</div>; }}", 
                    component_name, prop_name, prop_name, local_var, prop_name, local_var
                ))
            }
            "arrow_function_component" => {
                let prop_name = context.get_unique_name("prop");
                let local_var = context.get_unique_name("local");
                Some(format!(
                    "({{ {} }}: {{ {}: string }}) => {{ const {} = {}; return <div>{{{}}} content</div>; }}", 
                    prop_name, prop_name, local_var, prop_name, local_var
                ))
            }

            // Fallback to TypeScript for non-JSX node types
            _ => {
                // Use the full TypeScript implementation from TypeScriptPlugin
                let typescript_plugin = super::typescript_plugin::TypeScriptPlugin;
                typescript_plugin.generate_snippet(node_type, context)
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
