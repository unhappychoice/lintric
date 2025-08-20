use crate::collectors::common::definition_collectors::{
    find_identifier_nodes_in_node, DefinitionCollector,
};
use crate::models::{Definition, DefinitionType};
use tree_sitter::{Node, Query, QueryCursor};

pub struct RustDefinitionCollector<'a> {
    source_code: &'a str,
}

impl<'a> RustDefinitionCollector<'a> {
    pub fn new(source_code: &'a str) -> Self {
        Self { source_code }
    }
}

impl<'a> DefinitionCollector<'a> for RustDefinitionCollector<'a> {
    fn process_node(&self, node: Node<'a>, current_scope: &Option<String>) -> Vec<Definition> {
        match node.kind() {
            "function_item" | "function_signature_item" => {
                self.collect_function_definitions(node, current_scope)
            }
            "let_declaration" | "for_expression" | "if_expression" | "while_expression" => {
                self.collect_variable_definitions(node, current_scope)
            }
            "struct_item" | "enum_item" | "type_item" | "trait_item" | "impl_item" | "mod_item" => {
                self.collect_type_definitions(node, current_scope)
            }
            "use_declaration" => self.collect_import_definitions(node, current_scope),
            "closure_expression" => self.collect_closure_definitions(node, current_scope),
            "const_item" | "static_item" => self.collect_variable_definitions(node, current_scope),
            "macro_definition" => self.collect_macro_definitions(node, current_scope),
            _ => Vec::new(),
        }
    }

    fn determine_scope(&self, node: &Node<'a>, parent_scope: &Option<String>) -> Option<String> {
        let new_scope_name = match node.kind() {
            "function_item" | "struct_item" | "enum_item" | "trait_item" | "mod_item" => {
                node.child_by_field_name("name").map(|n| {
                    n.utf8_text(self.source_code.as_bytes())
                        .unwrap()
                        .trim()
                        .to_string()
                })
            }
            "impl_item" => node.child_by_field_name("type").map(|n| {
                n.utf8_text(self.source_code.as_bytes())
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

    fn collect_variable_definitions(
        &self,
        node: Node<'a>,
        current_scope: &Option<String>,
    ) -> Vec<Definition> {
        match node.kind() {
            "let_declaration" => {
                if let Some(pattern_node) = node.child_by_field_name("pattern") {
                    find_identifier_nodes_in_node(pattern_node)
                        .iter()
                        .map(|node| {
                            Definition::new(
                                node,
                                self.source_code,
                                DefinitionType::VariableDefinition,
                                current_scope.clone(),
                            )
                        })
                        .collect()
                } else {
                    vec![]
                }
            }
            "for_expression" => {
                if let Some(pattern_node) = node.child_by_field_name("pattern") {
                    find_identifier_nodes_in_node(pattern_node)
                        .iter()
                        .map(|node| {
                            Definition::new(
                                node,
                                self.source_code,
                                DefinitionType::VariableDefinition,
                                current_scope.clone(),
                            )
                        })
                        .collect()
                } else {
                    vec![]
                }
            }
            "if_expression" | "while_expression" => {
                let mut definitions = vec![];

                let mut cursor = node.walk();
                for let_condition_node in node.children(&mut cursor) {
                    if let_condition_node.kind() == "let_condition" {
                        let mut let_cursor = let_condition_node.walk();
                        for destruct_pattern_node in let_condition_node.children(&mut let_cursor) {
                            if destruct_pattern_node.kind() == "tuple_struct_pattern" {
                                let mut nodes =
                                    find_identifier_nodes_in_node(destruct_pattern_node)
                                        .into_iter();
                                nodes.next();

                                nodes.for_each(|node| {
                                    definitions.push(Definition::new(
                                        &node,
                                        self.source_code,
                                        DefinitionType::VariableDefinition,
                                        current_scope.clone(),
                                    ))
                                });
                            }
                        }
                    }
                }

                definitions
            }
            "const_item" | "static_item" => Definition::from_naming_node(
                &node,
                self.source_code,
                DefinitionType::ConstDefinition,
                current_scope.clone(),
            )
            .into_iter()
            .collect(),
            _ => vec![],
        }
    }

    fn collect_function_definitions(
        &self,
        node: Node<'a>,
        current_scope: &Option<String>,
    ) -> Vec<Definition> {
        let mut definitions = vec![];

        definitions.extend(Definition::from_naming_node(
            &node,
            self.source_code,
            DefinitionType::FunctionDefinition,
            current_scope.clone(),
        ));

        // Collect type parameters
        if let Some(type_params_node) = node.child_by_field_name("type_parameters") {
            definitions.extend(self.collect_type_parameters(type_params_node, current_scope));
        }

        if let Some(parameters_node) = node.child_by_field_name("parameters") {
            let mut param_cursor = parameters_node.walk();
            for param_child in parameters_node.children(&mut param_cursor) {
                if param_child.kind() == "parameter" {
                    if let Some(pattern_node) = param_child.child_by_field_name("pattern") {
                        find_identifier_nodes_in_node(pattern_node)
                            .iter()
                            .for_each(|node| {
                                definitions.push(Definition::new(
                                    node,
                                    self.source_code,
                                    DefinitionType::VariableDefinition,
                                    current_scope.clone(),
                                ))
                            });
                    }
                }
            }
        }

        definitions
    }

    fn collect_type_definitions(
        &self,
        node: Node<'a>,
        current_scope: &Option<String>,
    ) -> Vec<Definition> {
        let mut definitions = vec![];

        if let Some(name_node) = node.child_by_field_name("name") {
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

            definitions.push(Definition::new(
                &name_node,
                self.source_code,
                def_type,
                scope,
            ));
        }

        // Collect type parameters for structs, enums, traits, etc.
        if let Some(type_params_node) = node.child_by_field_name("type_parameters") {
            definitions.extend(self.collect_type_parameters(type_params_node, current_scope));
        }

        definitions
    }

    fn collect_import_definitions(
        &self,
        node: Node<'a>,
        current_scope: &Option<String>,
    ) -> Vec<Definition> {
        let mut definitions = vec![];
        let start_line = node.start_position().row + 1;
        let mut use_cursor = node.walk();
        for use_child in node.children(&mut use_cursor) {
            match use_child.kind() {
                "scoped_identifier" | "identifier" => {
                    let full_name = use_child
                        .utf8_text(self.source_code.as_bytes())
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
                                .utf8_text(self.source_code.as_bytes())
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
                                definitions.push(Definition::new(
                                    &alias_node,
                                    self.source_code,
                                    DefinitionType::ModuleDefinition,
                                    current_scope.clone(),
                                ));
                            }
                        }
                    }
                }
                _ => {}
            }
        }

        definitions
    }

    fn collect_closure_definitions(
        &self,
        node: Node<'a>,
        current_scope: &Option<String>,
    ) -> Vec<Definition> {
        let mut definitions = vec![];
        if let Some(parameters_node) = node.child_by_field_name("parameters") {
            let mut param_cursor = parameters_node.walk();
            for param_child in parameters_node.children(&mut param_cursor) {
                find_identifier_nodes_in_node(param_child)
                    .iter()
                    .for_each(|node| {
                        definitions.push(Definition::new(
                            node,
                            self.source_code,
                            DefinitionType::VariableDefinition,
                            current_scope.clone(),
                        ))
                    });
            }
        }

        definitions
    }

    fn collect_macro_definitions(
        &self,
        node: Node<'a>,
        current_scope: &Option<String>,
    ) -> Vec<Definition> {
        let mut definitions = vec![];

        definitions.extend(Definition::from_naming_node(
            &node,
            self.source_code,
            DefinitionType::MacroDefinition,
            current_scope.clone(),
        ));

        if let Some(macro_node) =
            run_query("(token_binding_pattern) @rule", node, self.source_code).first()
        {
            let nodes = run_query("(metavariable) @meta", *macro_node, self.source_code);

            for node in nodes {
                definitions.push(Definition::new(
                    &node,
                    self.source_code,
                    DefinitionType::MacroVariableDefinition,
                    current_scope.clone(),
                ));
            }
        }

        definitions
    }
}

impl<'a> RustDefinitionCollector<'a> {
    fn collect_type_parameters(
        &self,
        type_params_node: Node<'a>,
        current_scope: &Option<String>,
    ) -> Vec<Definition> {
        let mut definitions = vec![];

        let mut cursor = type_params_node.walk();
        for child in type_params_node.children(&mut cursor) {
            match child.kind() {
                "type_identifier" => {
                    definitions.push(Definition::new(
                        &child,
                        self.source_code,
                        DefinitionType::TypeDefinition,
                        current_scope.clone(),
                    ));
                }
                "lifetime" => {
                    // Extract identifier from lifetime node (e.g., 'a from lifetime('a))
                    let mut lifetime_cursor = child.walk();
                    for lifetime_child in child.children(&mut lifetime_cursor) {
                        if lifetime_child.kind() == "identifier" {
                            definitions.push(Definition::new(
                                &lifetime_child,
                                self.source_code,
                                DefinitionType::TypeDefinition, // Using TypeDefinition for lifetime parameters
                                current_scope.clone(),
                            ));
                        }
                    }
                }
                _ => {}
            }
        }

        definitions
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
