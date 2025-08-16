use crate::collectors::common::definition_collectors::{
    find_identifiers_in_pattern, DefinitionCollector,
};
use crate::models::{Definition, DefinitionType};
use tree_sitter::{Node, Query, QueryCursor};

pub struct RustDefinitionCollector;

impl DefinitionCollector for RustDefinitionCollector {
    fn process_node<'a>(
        &self,
        node: Node<'a>,
        source_code: &'a str,
        definitions: &mut Vec<Definition>,
        current_scope: &Option<String>,
    ) {
        match node.kind() {
            "function_item" | "function_signature_item" => {
                self.collect_function_definitions(node, source_code, definitions, current_scope);
            }
            "let_declaration" | "for_expression" | "if_expression" | "while_expression" => {
                self.collect_variable_definitions(node, source_code, definitions, current_scope);
            }
            "struct_item" | "enum_item" | "type_item" | "trait_item" | "impl_item" | "mod_item" => {
                self.collect_type_definitions(node, source_code, definitions, current_scope);
            }
            "use_declaration" => {
                self.collect_import_definitions(node, source_code, definitions, current_scope);
            }
            "closure_expression" => {
                self.collect_closure_definitions(node, source_code, definitions, current_scope);
            }
            "const_item" | "static_item" => {
                self.collect_variable_definitions(node, source_code, definitions, current_scope);
            }
            "macro_definition" => {
                self.collect_macro_definitions(node, source_code, definitions, current_scope);
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
            "function_item" | "struct_item" | "enum_item" | "trait_item" | "mod_item" => {
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

    fn collect_variable_definitions<'a>(
        &self,
        node: Node<'a>,
        source_code: &'a str,
        definitions: &mut Vec<Definition>,
        current_scope: &Option<String>,
    ) {
        match node.kind() {
            "let_declaration" => {
                if let Some(pattern_node) = node.child_by_field_name("pattern") {
                    let identifiers = find_identifiers_in_pattern(pattern_node, source_code);
                    for (name, line) in identifiers {
                        definitions.push(Definition {
                            name,
                            line_number: line,
                            definition_type: DefinitionType::VariableDefinition,
                            scope: current_scope.clone(),
                        });
                    }
                }
            }
            "for_expression" => {
                if let Some(pattern_node) = node.child_by_field_name("pattern") {
                    let identifiers = find_identifiers_in_pattern(pattern_node, source_code);
                    for (name, line) in identifiers {
                        definitions.push(Definition {
                            name,
                            line_number: line,
                            definition_type: DefinitionType::VariableDefinition,
                            scope: current_scope.clone(),
                        });
                    }
                }
            }
            "if_expression" | "while_expression" => {
                let mut cursor = node.walk();
                for let_condition_node in node.children(&mut cursor) {
                    if let_condition_node.kind() == "let_condition" {
                        let mut let_cursor = let_condition_node.walk();
                        for destruct_pattern_node in let_condition_node.children(&mut let_cursor) {
                            if destruct_pattern_node.kind() == "tuple_struct_pattern" {
                                let mut identifiers =
                                    find_identifiers_in_pattern(destruct_pattern_node, source_code)
                                        .into_iter();
                                identifiers.next();
                                for (name, line) in identifiers {
                                    definitions.push(Definition {
                                        name,
                                        line_number: line,
                                        definition_type: DefinitionType::VariableDefinition,
                                        scope: current_scope.clone(),
                                    });
                                }
                            }
                        }
                    }
                }
            }
            "const_item" | "static_item" => {
                let start_line = node.start_position().row + 1;
                if let Some(name_node) = node.child_by_field_name("name") {
                    let name = name_node
                        .utf8_text(source_code.as_bytes())
                        .unwrap()
                        .trim()
                        .to_string();
                    definitions.push(Definition {
                        name,
                        line_number: start_line,
                        definition_type: DefinitionType::ConstDefinition,
                        scope: current_scope.clone(),
                    });
                }
            }
            _ => {}
        }
    }

    fn collect_function_definitions<'a>(
        &self,
        node: Node<'a>,
        source_code: &'a str,
        definitions: &mut Vec<Definition>,
        current_scope: &Option<String>,
    ) {
        let start_line = node.start_position().row + 1;
        if let Some(name_node) = node.child_by_field_name("name") {
            let name = name_node
                .utf8_text(source_code.as_bytes())
                .unwrap()
                .trim()
                .to_string();
            definitions.push(Definition {
                name,
                line_number: start_line,
                definition_type: DefinitionType::FunctionDefinition,
                scope: current_scope.clone(),
            });
        }
        if let Some(parameters_node) = node.child_by_field_name("parameters") {
            let mut param_cursor = parameters_node.walk();
            for param_child in parameters_node.children(&mut param_cursor) {
                if param_child.kind() == "parameter" {
                    if let Some(pattern_node) = param_child.child_by_field_name("pattern") {
                        let identifiers = find_identifiers_in_pattern(pattern_node, source_code);
                        for (name, line) in identifiers {
                            definitions.push(Definition {
                                name,
                                line_number: line,
                                definition_type: DefinitionType::VariableDefinition,
                                scope: current_scope.clone(),
                            });
                        }
                    }
                }
            }
        }
    }

    fn collect_type_definitions<'a>(
        &self,
        node: Node<'a>,
        source_code: &'a str,
        definitions: &mut Vec<Definition>,
        current_scope: &Option<String>,
    ) {
        let start_line = node.start_position().row + 1;
        if let Some(name_node) = node.child_by_field_name("name") {
            let name = name_node
                .utf8_text(source_code.as_bytes())
                .unwrap()
                .trim()
                .to_string();

            let def_type = match node.kind() {
                "struct_item" => DefinitionType::StructDefinition,
                "enum_item" => DefinitionType::EnumDefinition,
                "type_item" => DefinitionType::TypeDefinition,
                "mod_item" => DefinitionType::ModuleDefinition,
                _ => DefinitionType::Other(node.kind().to_string()),
            };

            let scope = if node.kind() == "mod_item" {
                if let Some(scope_str) = current_scope {
                    scope_str
                        .rfind('.')
                        .map(|last_dot| scope_str[..last_dot].to_string())
                } else {
                    None
                }
            } else {
                current_scope.clone()
            };

            definitions.push(Definition {
                name,
                line_number: start_line,
                definition_type: def_type,
                scope,
            });
        }
    }

    fn collect_import_definitions<'a>(
        &self,
        node: Node<'a>,
        source_code: &'a str,
        definitions: &mut Vec<Definition>,
        current_scope: &Option<String>,
    ) {
        let start_line = node.start_position().row + 1;
        let mut use_cursor = node.walk();
        for use_child in node.children(&mut use_cursor) {
            match use_child.kind() {
                "scoped_identifier" | "identifier" => {
                    let full_name = use_child
                        .utf8_text(source_code.as_bytes())
                        .unwrap()
                        .trim()
                        .to_string();
                    let name = full_name
                        .split("::")
                        .last()
                        .unwrap_or(&full_name)
                        .to_string();
                    definitions.push(Definition {
                        name,
                        line_number: start_line,
                        definition_type: DefinitionType::ModuleDefinition,
                        scope: current_scope.clone(),
                    });
                }
                "use_clause" => {
                    let mut clause_cursor = use_child.walk();
                    for clause_child_node in use_child.children(&mut clause_cursor) {
                        if clause_child_node.kind() == "identifier"
                            || clause_child_node.kind() == "scoped_identifier"
                        {
                            let full_name = clause_child_node
                                .utf8_text(source_code.as_bytes())
                                .unwrap()
                                .trim()
                                .to_string();
                            let name = full_name
                                .split("::")
                                .last()
                                .unwrap_or(&full_name)
                                .to_string();
                            definitions.push(Definition {
                                name,
                                line_number: start_line,
                                definition_type: DefinitionType::ModuleDefinition,
                                scope: current_scope.clone(),
                            });
                        } else if clause_child_node.kind() == "use_as_clause" {
                            if let Some(alias_node) = clause_child_node.child_by_field_name("alias")
                            {
                                let name = alias_node
                                    .utf8_text(source_code.as_bytes())
                                    .unwrap()
                                    .trim()
                                    .to_string();
                                definitions.push(Definition {
                                    name,
                                    line_number: start_line,
                                    definition_type: DefinitionType::ModuleDefinition,
                                    scope: current_scope.clone(),
                                });
                            }
                        }
                    }
                }
                _ => {}
            }
        }
    }

    fn collect_closure_definitions<'a>(
        &self,
        node: Node<'a>,
        source_code: &'a str,
        definitions: &mut Vec<Definition>,
        current_scope: &Option<String>,
    ) {
        if let Some(parameters_node) = node.child_by_field_name("parameters") {
            let mut param_cursor = parameters_node.walk();
            for param_child in parameters_node.children(&mut param_cursor) {
                let identifiers = find_identifiers_in_pattern(param_child, source_code);
                for (name, line) in identifiers {
                    definitions.push(Definition {
                        name,
                        line_number: line,
                        definition_type: DefinitionType::VariableDefinition,
                        scope: current_scope.clone(),
                    });
                }
            }
        }
    }

    fn collect_macro_definitions<'a>(
        &self,
        node: Node<'a>,
        source_code: &'a str,
        definitions: &mut Vec<Definition>,
        current_scope: &Option<String>,
    ) {
        let start_line = node.start_position().row + 1;
        if let Some(name_node) = node.child_by_field_name("name") {
            let name = name_node
                .utf8_text(source_code.as_bytes())
                .unwrap()
                .trim()
                .to_string();
            definitions.push(Definition {
                name,
                line_number: start_line,
                definition_type: DefinitionType::MacroDefinition,
                scope: current_scope.clone(),
            });
        }

        if let Some(macro_node) =
            run_query("(token_binding_pattern) @rule", node, source_code).first()
        {
            let nodes = run_query("(metavariable) @meta", *macro_node, source_code);

            for node in nodes {
                let name = node
                    .utf8_text(source_code.as_bytes())
                    .unwrap()
                    .trim()
                    .to_string();
                definitions.push(Definition {
                    name,
                    line_number: node.start_position().row + 1,
                    definition_type: DefinitionType::MacroVariableDefinition,
                    scope: current_scope.clone(),
                });
            }
        }
    }
}

fn run_query<'a>(query: &str, node: Node<'a>, source_code: &str) -> Vec<Node<'a>> {
    let mut result: Vec<Node<'a>> = vec![];

    let query = Query::new(&tree_sitter_rust::language(), query).unwrap();
    let mut query_cursor = QueryCursor::new();

    for m in query_cursor.matches(&query, node, source_code.as_bytes()) {
        for capture in m.captures {
            result.push(capture.node);
        }
    }

    result
}
