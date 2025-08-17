use crate::collectors::common::dependency_collectors::DependencyCollector;
use crate::models::{Definition, Dependency, DependencyType};

use tree_sitter::Node;

pub struct TypescriptDependencyCollector<'a> {
    source_code: &'a str,
}

impl<'a> TypescriptDependencyCollector<'a> {
    pub fn new(source_code: &'a str) -> Self {
        Self { source_code }
    }
}

impl<'a> DependencyCollector<'a> for TypescriptDependencyCollector<'a> {
    fn process_node(
        &self,
        node: Node<'a>,
        definitions: &[Definition],
        current_scope: &Option<String>,
    ) -> Vec<Dependency> {
        let mut dependencies = Vec::new();
        match node.kind() {
            "identifier" => {
                self.handle_identifier(node, &mut dependencies, definitions, current_scope);
            }
            "call_expression" => {
                self.handle_call_expression(node, &mut dependencies, definitions, current_scope);
            }
            "property_identifier" => {
                self.handle_field_expression(node, &mut dependencies, definitions, current_scope);
            }
            _ => {}
        }
        dependencies
    }

    fn determine_scope(&self, node: &Node<'a>, parent_scope: &Option<String>) -> Option<String> {
        let new_scope_name = match node.kind() {
            "function_declaration" | "class_declaration" | "interface_declaration" | "module" => {
                node.child_by_field_name("name").map(|n| {
                    n.utf8_text(self.source_code.as_bytes())
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

    fn handle_identifier(
        &self,
        node: Node<'a>,
        dependencies: &mut Vec<Dependency>,
        definitions: &[Definition],
        current_scope: &Option<String>,
    ) {
        let parent_kind = node.parent().map(|p| p.kind());

        let is_declaration_name = parent_kind == Some("function_declaration")
            && (node.parent().unwrap().child_by_field_name("name") == Some(node))
            || parent_kind == Some("class_declaration")
                && (node.parent().unwrap().child_by_field_name("name") == Some(node))
            || parent_kind == Some("interface_declaration")
                && (node.parent().unwrap().child_by_field_name("name") == Some(node));

        if parent_kind != Some("variable_declarator") && !is_declaration_name {
            self.add_dependency_if_needed(
                dependencies,
                node,
                self.source_code,
                definitions,
                current_scope,
                DependencyType::VariableUse,
                parent_kind.map(|k| k.to_string()),
            );
        }
    }

    fn handle_call_expression(
        &self,
        node: Node<'a>,
        dependencies: &mut Vec<Dependency>,
        definitions: &[Definition],
        current_scope: &Option<String>,
    ) {
        if let Some(function_node) = node.child_by_field_name("function") {
            if function_node.kind() == "identifier" {
                self.add_dependency_if_needed(
                    dependencies,
                    function_node,
                    self.source_code,
                    definitions,
                    current_scope,
                    DependencyType::FunctionCall,
                    Some("call_expression".to_string()),
                );
            }
        }
        if let Some(arguments_node) = node.child_by_field_name("arguments") {
            let mut arg_cursor = arguments_node.walk();
            for arg_child in arguments_node.children(&mut arg_cursor) {
                if arg_child.kind() == "identifier" {
                    self.add_dependency_if_needed(
                        dependencies,
                        arg_child,
                        self.source_code,
                        definitions,
                        current_scope,
                        DependencyType::VariableUse,
                        Some("variable_use".to_string()),
                    );
                }
            }
        }
    }

    fn handle_field_expression(
        &self,
        node: Node<'a>,
        dependencies: &mut Vec<Dependency>,
        definitions: &[Definition],
        current_scope: &Option<String>,
    ) {
        if let Some(parent) = node.parent() {
            if parent.kind() == "member_expression" {
                if let Some(object_node) = parent.child_by_field_name("object") {
                    if object_node.kind() == "identifier" {
                        self.add_dependency_if_needed(
                            dependencies,
                            object_node,
                            self.source_code,
                            definitions,
                            current_scope,
                            DependencyType::StructFieldAccess,
                            Some("member_access".to_string()),
                        );
                    }
                }
            }
        }
    }

    fn handle_struct_expression(
        &self,
        _node: Node<'a>,
        _dependencies: &mut Vec<Dependency>,
        _definitions: &[Definition],
        _current_scope: &Option<String>,
    ) {
        // Empty implementation as there is no equivalent to struct_expression in TypeScript.
    }

    fn handle_metavariable(
        &self,
        _node: Node<'a>,
        _dependencies: &mut Vec<Dependency>,
        _definitions: &[Definition],
        _current_scope: &Option<String>,
    ) {
    }
}
