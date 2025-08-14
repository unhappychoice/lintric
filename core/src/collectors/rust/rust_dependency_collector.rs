use crate::collectors::common::dependency_collectors::DependencyCollector;
use crate::models::{Definition, Dependency, DependencyType};
use tree_sitter::Node;

pub struct RustDependencyCollector;

impl DependencyCollector for RustDependencyCollector {
    fn process_node<'a>(
        &self,
        node: Node<'a>,
        source_code: &'a str,
        dependencies: &mut Vec<Dependency>,
        definitions: &[Definition],
        current_scope: &Option<String>,
    ) {
        match node.kind() {
            "identifier" => {
                self.handle_identifier(node, source_code, dependencies, definitions, current_scope);
            }
            "call_expression" => {
                self.handle_call_expression(
                    node,
                    source_code,
                    dependencies,
                    definitions,
                    current_scope,
                );
            }
            "field_expression" => {
                self.handle_field_expression(
                    node,
                    source_code,
                    dependencies,
                    definitions,
                    current_scope,
                );
            }
            "struct_expression" => {
                self.handle_struct_expression(
                    node,
                    source_code,
                    dependencies,
                    definitions,
                    current_scope,
                );
            }
            _ => {}
        }
    }

    fn determine_scope<'a>(
        &self,
        node: &Node<'a>,
        source_code: &'a str,
        parent_scope: &Option<String>,
    ) -> Option<String> {
        let new_scope_name = match node.kind() {
            "function_item" | "struct_item" | "enum_item" | "trait_item" | "impl_item" => {
                node.child_by_field_name("name").map(|n| {
                    n.utf8_text(source_code.as_bytes())
                        .unwrap()
                        .trim()
                        .to_string()
                })
            }
            _ => None,
        };

        if let Some(name) = new_scope_name {
            Some(
                parent_scope
                    .as_ref()
                    .map_or(name.clone(), |p| format!("{p}.{name}")),
            )
        } else {
            parent_scope.clone()
        }
    }

    fn handle_identifier<'a>(
        &self,
        node: Node<'a>,
        source_code: &'a str,
        dependencies: &mut Vec<Dependency>,
        definitions: &[Definition],
        current_scope: &Option<String>,
    ) {
        let parent_kind = node.parent().map(|p| p.kind());

        let is_declaration_name = parent_kind == Some("function_item")
            && (node.parent().unwrap().child_by_field_name("name") == Some(node))
            || parent_kind == Some("struct_item")
                && (node.parent().unwrap().child_by_field_name("name") == Some(node))
            || parent_kind == Some("enum_item")
                && (node.parent().unwrap().child_by_field_name("name") == Some(node))
            || parent_kind == Some("trait_item")
                && (node.parent().unwrap().child_by_field_name("name") == Some(node))
            || parent_kind == Some("impl_item")
                && (node.parent().unwrap().child_by_field_name("name") == Some(node))
            || parent_kind == Some("type_alias")
                && (node.parent().unwrap().child_by_field_name("name") == Some(node));

        if parent_kind != Some("parameter")
            && parent_kind != Some("pattern")
            && !is_declaration_name
        {
            self.add_dependency_if_needed(
                dependencies,
                node,
                source_code,
                definitions,
                current_scope,
                DependencyType::VariableUse,
                parent_kind.map(|k| k.to_string()),
            );
        }
    }

    fn handle_call_expression<'a>(
        &self,
        node: Node<'a>,
        source_code: &'a str,
        dependencies: &mut Vec<Dependency>,
        definitions: &[Definition],
        current_scope: &Option<String>,
    ) {
        if let Some(function_node) = node.child_by_field_name("function") {
            if function_node.kind() == "identifier" {
                self.add_dependency_if_needed(
                    dependencies,
                    function_node,
                    source_code,
                    definitions,
                    current_scope,
                    DependencyType::FunctionCall,
                    Some("call_expression".to_string()),
                );
            }
        }

        if let Some(arguments_node) = node.child_by_field_name("arguments") {
            let mut args_cursor = arguments_node.walk();
            for arg_child in arguments_node.children(&mut args_cursor) {
                if arg_child.kind() == "identifier" {
                    self.add_dependency_if_needed(
                        dependencies,
                        arg_child,
                        source_code,
                        definitions,
                        current_scope,
                        DependencyType::VariableUse,
                        Some("arguments".to_string()),
                    );
                }
            }
        }
    }

    fn handle_field_expression<'a>(
        &self,
        node: Node<'a>,
        source_code: &'a str,
        dependencies: &mut Vec<Dependency>,
        definitions: &[Definition],
        current_scope: &Option<String>,
    ) {
        if let Some(operand_node) = node.child_by_field_name("operand") {
            if operand_node.kind() == "identifier" {
                self.add_dependency_if_needed(
                    dependencies,
                    operand_node,
                    source_code,
                    definitions,
                    current_scope,
                    DependencyType::StructFieldAccess,
                    Some("field_access".to_string()),
                );
            }
        }
        if let Some(type_node) = node.child_by_field_name("field") {
            self.add_dependency_if_needed(
                dependencies,
                type_node,
                source_code,
                definitions,
                current_scope,
                DependencyType::StructFieldAccess,
                Some("field_access".to_string()),
            );
        }
    }

    fn handle_struct_expression<'a>(
        &self,
        node: Node<'a>,
        source_code: &'a str,
        dependencies: &mut Vec<Dependency>,
        definitions: &[Definition],
        current_scope: &Option<String>,
    ) {
        if let Some(type_node) = node.child_by_field_name("type") {
            self.add_dependency_if_needed(
                dependencies,
                type_node,
                source_code,
                definitions,
                current_scope,
                DependencyType::TypeReference,
                Some("struct_instantiation".to_string()),
            );
        }
    }
}
