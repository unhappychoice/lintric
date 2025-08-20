use crate::collectors::common::definition_collectors::{
    find_identifier_nodes_in_node, DefinitionCollector,
};
use crate::models::{Definition, DefinitionType};
use tree_sitter::Node;

pub struct TypescriptDefinitionCollector<'a> {
    source_code: &'a str,
}

impl<'a> TypescriptDefinitionCollector<'a> {
    pub fn new(source_code: &'a str) -> Self {
        Self { source_code }
    }
}

impl<'a> DefinitionCollector<'a> for TypescriptDefinitionCollector<'a> {
    fn process_node(&self, node: Node<'a>, current_scope: &Option<String>) -> Vec<Definition> {
        let mut definitions = Vec::new();

        match node.kind() {
            "function_declaration" | "method_definition" | "arrow_function" => {
                definitions.extend(self.collect_function_definitions(node, current_scope));
            }
            "variable_declarator" => {
                definitions.extend(self.collect_variable_definitions(node, current_scope));
            }
            "class_declaration" | "interface_declaration" | "type_alias_declaration" => {
                definitions.extend(self.collect_type_definitions(node, current_scope));
            }
            "import_statement" | "export_statement" => {
                definitions.extend(self.collect_import_definitions(node, current_scope));
            }
            _ => {}
        }
        definitions
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

    fn collect_variable_definitions(
        &self,
        node: Node<'a>,
        current_scope: &Option<String>,
    ) -> Vec<Definition> {
        Definition::from_naming_node(
            &node,
            self.source_code,
            DefinitionType::VariableDefinition,
            current_scope.clone(),
        )
        .into_iter()
        .collect()
    }

    fn collect_function_definitions(
        &self,
        node: Node<'a>,
        current_scope: &Option<String>,
    ) -> Vec<Definition> {
        let mut definitions = Vec::new();

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
                if param_child.kind() == "required_parameter"
                    || param_child.kind() == "optional_parameter"
                {
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
        let mut definitions = Vec::new();

        let def_type = match node.kind() {
            "class_declaration" => DefinitionType::ClassDefinition,
            "interface_declaration" => DefinitionType::InterfaceDefinition,
            "type_alias_declaration" => DefinitionType::TypeDefinition,
            _ => DefinitionType::Other(node.kind().to_string()),
        };

        definitions.extend(Definition::from_naming_node(
            &node,
            self.source_code,
            def_type,
            current_scope.clone(),
        ));

        // Collect type parameters for classes, interfaces, and type aliases
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
        let mut definitions: Vec<Definition> = Vec::new();

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
                                    definitions.extend(Definition::from_naming_node(
                                        &named_import_child,
                                        self.source_code,
                                        DefinitionType::ModuleDefinition,
                                        current_scope.clone(),
                                    ));
                                }

                                if let Some(alias_node) =
                                    named_import_child.child_by_field_name("alias")
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
                        "namespace_import" => {
                            if let Some(alias_node) = import_child.child_by_field_name("alias") {
                                definitions.push(Definition::new(
                                    &alias_node,
                                    self.source_code,
                                    DefinitionType::ModuleDefinition,
                                    current_scope.clone(),
                                ));
                            }
                        }
                        "identifier" => {
                            definitions.push(Definition::new(
                                &import_child,
                                self.source_code,
                                DefinitionType::ModuleDefinition,
                                current_scope.clone(),
                            ));
                        }
                        _ => {}
                    }
                }
            }
        }

        definitions
    }

    fn collect_closure_definitions(
        &self,
        _node: Node<'a>,
        _current_scope: &Option<String>,
    ) -> Vec<Definition> {
        vec![]
    }

    fn collect_macro_definitions(
        &self,
        _node: Node<'a>,
        _current_scope: &Option<String>,
    ) -> Vec<Definition> {
        vec![]
    }
}

impl<'a> TypescriptDefinitionCollector<'a> {
    fn collect_type_parameters(
        &self,
        type_params_node: Node<'a>,
        current_scope: &Option<String>,
    ) -> Vec<Definition> {
        let mut definitions = vec![];

        let mut cursor = type_params_node.walk();
        for child in type_params_node.children(&mut cursor) {
            if child.kind() == "type_parameter" {
                // TypeScript type_parameter structure: type_parameter -> type_identifier
                let mut param_cursor = child.walk();
                for param_child in child.children(&mut param_cursor) {
                    if param_child.kind() == "type_identifier" {
                        definitions.push(Definition::new(
                            &param_child,
                            self.source_code,
                            DefinitionType::TypeDefinition,
                            current_scope.clone(),
                        ));
                    }
                }
            }
        }

        definitions
    }
}
