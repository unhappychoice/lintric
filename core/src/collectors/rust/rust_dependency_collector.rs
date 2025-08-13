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
        let start_line = node.start_position().row + 1;
        let parent_kind = node.parent().map(|p| p.kind());
        let name = node
            .utf8_text(source_code.as_bytes())
            .unwrap()
            .trim()
            .to_string();

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
            let mut args_cursor = arguments_node.walk();
            for arg_child in arguments_node.children(&mut args_cursor) {
                if arg_child.kind() == "identifier" {
                    let arg_name = arg_child
                        .utf8_text(source_code.as_bytes())
                        .unwrap()
                        .trim()
                        .to_string();
                    if let Some(def) =
                        find_definition_in_scope(definitions, &arg_name, current_scope)
                    {
                        dependencies.push(Dependency {
                            source_line: start_line,
                            target_line: def.line_number,
                            symbol: arg_name,
                            dependency_type: DependencyType::VariableUse,
                            context: Some("arguments".to_string()),
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
        if let Some(operand_node) = node.child_by_field_name("operand") {
            if operand_node.kind() == "identifier" {
                let name = operand_node
                    .utf8_text(source_code.as_bytes())
                    .unwrap()
                    .trim()
                    .to_string();
                if let Some(def) = find_definition_in_scope(definitions, &name, current_scope) {
                    dependencies.push(Dependency {
                        source_line: start_line,
                        target_line: def.line_number,
                        symbol: name,
                        dependency_type: DependencyType::StructFieldAccess,
                        context: Some("field_access".to_string()),
                    });
                }
            }
        }
        if let Some(type_node) = node.child_by_field_name("field") {
            let type_name = type_node
                .utf8_text(source_code.as_bytes())
                .unwrap()
                .trim()
                .to_string();
            // Search for definition considering scope
            if let Some(def) = find_definition_in_scope(definitions, &type_name, current_scope) {
                dependencies.push(Dependency {
                    source_line: start_line,
                    target_line: def.line_number,
                    symbol: type_name,
                    dependency_type: DependencyType::StructFieldAccess,
                    context: Some("field_access".to_string()),
                });
            }
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
        let start_line = node.start_position().row + 1;
        if let Some(type_node) = node.child_by_field_name("type") {
            let type_name = type_node
                .utf8_text(source_code.as_bytes())
                .unwrap()
                .trim()
                .to_string();
            // Search for definition considering scope
            if let Some(def) = find_definition_in_scope(definitions, &type_name, current_scope) {
                dependencies.push(Dependency {
                    source_line: start_line,
                    target_line: def.line_number,
                    symbol: type_name,
                    dependency_type: DependencyType::TypeReference,
                    context: Some("struct_instantiation".to_string()),
                });
            }
        }
    }
}

// Helper function to find a definition considering scope
fn find_definition_in_scope<'a>(
    definitions: &'a [Definition],
    name: &str,
    current_scope: &Option<String>,
) -> Option<&'a Definition> {
    // First, try to find a definition that exactly matches the current scope
    if let Some(def) = definitions
        .iter()
        .find(|d| d.name == name && d.scope == *current_scope)
    {
        return Some(def);
    }

    // If current scope is Some, traverse up the ancestor scopes
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

    // If not found in any specific scope, look for global definitions (scope is None)
    definitions
        .iter()
        .find(|d| d.name == name && d.scope.is_none())
}
