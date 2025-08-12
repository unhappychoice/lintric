use crate::collectors::common::dependency_collectors::DependencyCollector;
use crate::models::{Definition, Dependency, DependencyType};

use tree_sitter::Node;

pub struct TypescriptDependencyCollector;

impl DependencyCollector for TypescriptDependencyCollector {
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
            "property_identifier" => {
                self.handle_field_expression(
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
            "function_declaration" | "class_declaration" | "interface_declaration" | "module" => {
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
        let start_line = node.start_position().row + 1;
        let parent_kind = node.parent().map(|p| p.kind());
        let name = node
            .utf8_text(source_code.as_bytes())
            .unwrap()
            .trim()
            .to_string();

        let is_declaration_name = parent_kind == Some("function_declaration")
            && (node.parent().unwrap().child_by_field_name("name") == Some(node))
            || parent_kind == Some("class_declaration")
                && (node.parent().unwrap().child_by_field_name("name") == Some(node))
            || parent_kind == Some("interface_declaration")
                && (node.parent().unwrap().child_by_field_name("name") == Some(node));

        if parent_kind != Some("variable_declarator") && !is_declaration_name {
            if let Some(def) = find_definition_in_scope(definitions, &name, current_scope) {
                dependencies.push(Dependency {
                    source_line: start_line,
                    target_line: def.line_number,
                    symbol: name,
                    dependency_type: DependencyType::VariableUse,
                    context: parent_kind.map(|k| k.to_string()),
                });
            }
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
        let start_line = node.start_position().row + 1;
        if let Some(function_node) = node.child_by_field_name("function") {
            if function_node.kind() == "identifier" {
                let name = function_node
                    .utf8_text(source_code.as_bytes())
                    .unwrap()
                    .trim()
                    .to_string();
                if let Some(def) = find_definition_in_scope(definitions, &name, current_scope) {
                    dependencies.push(Dependency {
                        source_line: start_line,
                        target_line: def.line_number,
                        symbol: name,
                        dependency_type: DependencyType::FunctionCall,
                        context: Some("call_expression".to_string()),
                    });
                }
            }
        }
        if let Some(arguments_node) = node.child_by_field_name("arguments") {
            let mut arg_cursor = arguments_node.walk();
            for arg_child in arguments_node.children(&mut arg_cursor) {
                if arg_child.kind() == "identifier" {
                    let name = arg_child
                        .utf8_text(source_code.as_bytes())
                        .unwrap()
                        .trim()
                        .to_string();
                    if let Some(def) = find_definition_in_scope(definitions, &name, current_scope) {
                        dependencies.push(Dependency {
                            source_line: start_line,
                            target_line: def.line_number,
                            symbol: name,
                            dependency_type: DependencyType::VariableUse,
                            context: Some("variable_use".to_string()),
                        });
                    }
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
        let start_line = node.start_position().row + 1;
        if let Some(parent) = node.parent() {
            if parent.kind() == "member_expression" {
                if let Some(object_node) = parent.child_by_field_name("object") {
                    if object_node.kind() == "identifier" {
                        let name = object_node
                            .utf8_text(source_code.as_bytes())
                            .unwrap()
                            .trim()
                            .to_string();
                        if let Some(def) =
                            find_definition_in_scope(definitions, &name, current_scope)
                        {
                            dependencies.push(Dependency {
                                source_line: start_line,
                                target_line: def.line_number,
                                symbol: name,
                                dependency_type: DependencyType::StructFieldAccess,
                                context: Some("member_access".to_string()),
                            });
                        }
                    }
                }
            }
        }
    }

    fn handle_struct_expression<'a>(
        &self,
        _node: Node<'a>,
        _source_code: &'a str,
        _dependencies: &mut Vec<Dependency>,
        _definitions: &[Definition],
        _current_scope: &Option<String>,
    ) {
        // Empty implementation as there is no equivalent to struct_expression in TypeScript.
    }
}

fn find_definition_in_scope<'a>(
    definitions: &'a [Definition],
    name: &str,
    current_scope: &Option<String>,
) -> Option<&'a Definition> {
    if let Some(def) = definitions
        .iter()
        .find(|d| d.name == name && d.scope == *current_scope)
    {
        return Some(def);
    }

    if let Some(current_scope_str) = current_scope {
        let mut parts: Vec<&str> = current_scope_str.split('.').collect();
        while !parts.is_empty() {
            parts.pop(); // Remove the innermost scope
            let ancestor_scope = if parts.is_empty() {
                None
            } else {
                Some(parts.join("."))
            };
            if let Some(def) = definitions
                .iter()
                .find(|d| d.name == name && d.scope == ancestor_scope)
            {
                return Some(def);
            }
        }
    }

    definitions
        .iter()
        .find(|d| d.name == name && d.scope.is_none())
}
