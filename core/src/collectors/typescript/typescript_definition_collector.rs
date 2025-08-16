use crate::collectors::common::definition_collectors::{
    find_identifiers_in_pattern, DefinitionCollector,
};
use crate::models::{Definition, DefinitionType};
use tree_sitter::Node;

pub struct TypescriptDefinitionCollector;

impl DefinitionCollector for TypescriptDefinitionCollector {
    fn process_node<'a>(
        &self,
        node: Node<'a>,
        source_code: &'a str,
        definitions: &mut Vec<Definition>,
        current_scope: &Option<String>,
    ) {
        match node.kind() {
            "function_declaration" | "method_definition" | "arrow_function" => {
                self.collect_function_definitions(node, source_code, definitions, current_scope);
            }
            "variable_declarator" => {
                self.collect_variable_definitions(node, source_code, definitions, current_scope);
            }
            "class_declaration" | "interface_declaration" | "type_alias_declaration" => {
                self.collect_type_definitions(node, source_code, definitions, current_scope);
            }
            "import_statement" | "export_statement" => {
                self.collect_import_definitions(node, source_code, definitions, current_scope);
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

    fn collect_variable_definitions<'a>(
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
                definition_type: DefinitionType::VariableDefinition,
                scope: current_scope.clone(),
            });
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
                if param_child.kind() == "required_parameter"
                    || param_child.kind() == "optional_parameter"
                {
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
                "class_declaration" => DefinitionType::ClassDefinition,
                "interface_declaration" => DefinitionType::InterfaceDefinition,
                "type_alias_declaration" => DefinitionType::TypeDefinition,
                _ => DefinitionType::Other(node.kind().to_string()),
            };

            definitions.push(Definition {
                name,
                line_number: start_line,
                definition_type: def_type,
                scope: current_scope.clone(),
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

        for child in node.children(&mut node.walk()) {
            if child.kind() == "import_clause" {
                for import_child in child.children(&mut child.walk()) {
                    match import_child.kind() {
                        "named_imports" => {
                            let mut named_imports_cursor = import_child.walk();
                            for named_import_child in
                                import_child.children(&mut named_imports_cursor)
                            {
                                if named_import_child.kind() == "import_specifier" {
                                    if let Some(name_node) =
                                        named_import_child.child_by_field_name("name")
                                    {
                                        let name = name_node
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
                                    if let Some(alias_node) =
                                        named_import_child.child_by_field_name("alias")
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
                        "namespace_import" => {
                            if let Some(alias_node) = import_child.child_by_field_name("alias") {
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
                        "identifier" => {
                            let name = import_child
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
                        _ => {}
                    }
                }
            }
        }
    }

    fn collect_closure_definitions<'a>(
        &self,
        _node: Node<'a>,
        _source_code: &'a str,
        _definitions: &mut Vec<Definition>,
        _current_scope: &Option<String>,
    ) {
    }

    fn collect_macro_definitions<'a>(
        &self,
        _node: Node<'a>,
        _source_code: &'a str,
        _definitions: &mut Vec<Definition>,
        _current_scope: &Option<String>,
    ) {
        // No direct equivalent to Rust macros in TypeScript for definition collection.
    }
}
