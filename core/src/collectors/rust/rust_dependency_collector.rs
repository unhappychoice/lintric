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
            "function_item" | "struct_item" | "enum_item" | "trait_item" => {
                node.child_by_field_name("name").map(|n| {
                    n.utf8_text(source_code.as_bytes())
                        .unwrap()
                        .trim()
                        .to_string()
                })
            }
            "impl_item" => node.child_by_field_name("type").map(|n| {
                n.utf8_text(source_code.as_bytes())
                    .unwrap()
                    .trim()
                    .to_string()
            }),
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
            } else if function_node.kind() == "scoped_identifier" {
                let path_node = function_node.child_by_field_name("path").unwrap();
                let name_node = function_node.child_by_field_name("name").unwrap();

                let path_text = path_node.utf8_text(source_code.as_bytes()).unwrap();
                let name_text = name_node.utf8_text(source_code.as_bytes()).unwrap();

                let f_definitions: Vec<&Definition> =
                    definitions.iter().filter(|d| d.name == name_text).collect();

                for def in f_definitions {
                    if let Some(scope) = &def.scope {
                        if scope.starts_with(path_text) {
                            let source_line = name_node.start_position().row + 1;
                            let target_line = def.line_number;
                            if source_line != target_line {
                                dependencies.push(Dependency {
                                    source_line,
                                    target_line,
                                    symbol: name_text.to_string(),
                                    dependency_type: DependencyType::FunctionCall,
                                    context: Some("call_expression".to_string()),
                                });
                            }
                        }
                    }
                }
            } else if function_node.kind() == "field_expression" {
                let method_node = function_node.child_by_field_name("field").unwrap();
                let method_text = method_node.utf8_text(source_code.as_bytes()).unwrap();
                let method_def = definitions.iter().find(|d| d.name == method_text);

                if let Some(m_def) = method_def {
                    let source_line = method_node.start_position().row + 1;
                    let target_line = m_def.line_number;
                    if source_line != target_line {
                        dependencies.push(Dependency {
                            source_line,
                            target_line,
                            symbol: method_text.to_string(),
                            dependency_type: DependencyType::FunctionCall,
                            context: Some("call_expression".to_string()),
                        });
                    }
                }
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
        if let Some(parent) = node.parent() {
            if parent.kind() == "call_expression"
                && parent.child_by_field_name("function") == Some(node)
            {
                // This field expression is the function part of a method call,
                // so we let handle_call_expression deal with it.
                return;
            }
        }

        if let Some(value_node) = node.child_by_field_name("value") {
            if value_node.kind() == "identifier" {
                self.add_dependency_if_needed(
                    dependencies,
                    value_node,
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
