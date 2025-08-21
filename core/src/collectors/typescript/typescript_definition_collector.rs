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
    fn process_node(&self, node: Node<'a>) -> Vec<Definition> {
        let mut definitions = Vec::new();

        match node.kind() {
            "function_declaration" | "method_definition" | "arrow_function" => {
                definitions.extend(self.collect_function_definitions(node));
            }
            "variable_declarator" => {
                definitions.extend(self.collect_variable_definitions(node));
            }
            "class_declaration" | "interface_declaration" | "type_alias_declaration" => {
                definitions.extend(self.collect_type_definitions(node));
            }
            "import_statement" | "export_statement" => {
                definitions.extend(self.collect_import_definitions(node));
            }
            "public_field_definition" | "private_field_definition" | "field_definition" => {
                definitions.extend(self.collect_class_field_definitions(node));
            }
            "property_signature" | "method_signature" => {
                definitions.extend(self.collect_interface_member_definitions(node));
            }
            _ => {}
        }
        definitions
    }

    fn collect_variable_definitions(&self, node: Node<'a>) -> Vec<Definition> {
        if let Some(name_node) = node.child_by_field_name("name") {
            find_identifier_nodes_in_node(name_node)
                .iter()
                .map(|node| {
                    Definition::new(node, self.source_code, DefinitionType::VariableDefinition)
                })
                .collect()
        } else {
            vec![]
        }
    }

    fn collect_function_definitions(&self, node: Node<'a>) -> Vec<Definition> {
        let mut definitions = vec![];

        // Function name
        definitions.extend(Definition::from_naming_node(
            &node,
            self.source_code,
            DefinitionType::FunctionDefinition,
        ));

        // Function parameters
        if let Some(params_node) = node.child_by_field_name("parameters") {
            definitions.extend(self.collect_function_parameters(params_node));
        }

        definitions
    }

    fn collect_type_definitions(&self, node: Node<'a>) -> Vec<Definition> {
        let mut definitions = vec![];

        let definition_type = match node.kind() {
            "class_declaration" => DefinitionType::ClassDefinition,
            "interface_declaration" => DefinitionType::InterfaceDefinition,
            "type_alias_declaration" => DefinitionType::TypeDefinition,
            _ => DefinitionType::Other("unknown".to_string()),
        };

        // Collect the main type definition
        definitions.extend(Definition::from_naming_node(
            &node,
            self.source_code,
            definition_type,
        ));

        // Collect type parameters
        if let Some(type_params) = node.child_by_field_name("type_parameters") {
            definitions.extend(self.collect_type_parameters(type_params));
        }

        definitions
    }

    fn collect_import_definitions(&self, node: Node<'a>) -> Vec<Definition> {
        let mut definitions = vec![];
        let mut cursor = node.walk();

        // Traverse the import statement to find import specifiers
        for child in node.children(&mut cursor) {
            definitions.extend(self.collect_import_identifiers(child));
        }

        definitions
    }

    fn collect_closure_definitions(&self, node: Node<'a>) -> Vec<Definition> {
        let mut definitions = vec![];
        if let Some(params_node) = node.child_by_field_name("parameters") {
            definitions.extend(self.collect_function_parameters(params_node));
        }
        definitions
    }

    fn collect_macro_definitions(&self, _node: Node<'a>) -> Vec<Definition> {
        // TypeScript doesn't have macros like Rust
        vec![]
    }
}

impl<'a> TypescriptDefinitionCollector<'a> {
    fn collect_class_field_definitions(&self, node: Node<'a>) -> Vec<Definition> {
        // Look for property_identifier node in class field definitions
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "property_identifier" {
                return vec![Definition::new(
                    &child,
                    self.source_code,
                    DefinitionType::PropertyDefinition,
                )];
            }
        }
        vec![]
    }

    fn collect_interface_member_definitions(&self, node: Node<'a>) -> Vec<Definition> {
        let definition_type = match node.kind() {
            "property_signature" => DefinitionType::PropertyDefinition,
            "method_signature" => DefinitionType::MethodDefinition,
            _ => DefinitionType::Other("unknown".to_string()),
        };

        if let Some(name_node) = node.child_by_field_name("name") {
            vec![Definition::new(
                &name_node,
                self.source_code,
                definition_type,
            )]
        } else {
            vec![]
        }
    }

    fn collect_function_parameters(&self, node: Node<'a>) -> Vec<Definition> {
        let mut definitions = vec![];
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            match child.kind() {
                "required_parameter" | "optional_parameter" => {
                    if let Some(pattern_node) = child.child_by_field_name("pattern") {
                        definitions.extend(
                            find_identifier_nodes_in_node(pattern_node)
                                .iter()
                                .map(|n| {
                                    Definition::new(
                                        n,
                                        self.source_code,
                                        DefinitionType::VariableDefinition,
                                    )
                                })
                                .collect::<Vec<_>>(),
                        );
                    }
                }
                _ => {}
            }
        }
        definitions
    }

    fn collect_import_identifiers(&self, node: Node<'a>) -> Vec<Definition> {
        let mut definitions = vec![];
        let mut cursor = node.walk();

        match node.kind() {
            "import_specifier" => {
                // Look for the identifier in import specifier
                for child in node.children(&mut cursor) {
                    if child.kind() == "identifier" {
                        definitions.push(Definition::new(
                            &child,
                            self.source_code,
                            DefinitionType::ImportDefinition,
                        ));
                    }
                }
            }
            "namespace_import" => {
                // Handle import * as name
                if let Some(name_node) = node.child_by_field_name("name") {
                    definitions.push(Definition::new(
                        &name_node,
                        self.source_code,
                        DefinitionType::ImportDefinition,
                    ));
                }
            }
            _ => {
                // Recursively search in children
                for child in node.children(&mut cursor) {
                    definitions.extend(self.collect_import_identifiers(child));
                }
            }
        }

        definitions
    }

    fn collect_type_parameters(&self, node: Node<'a>) -> Vec<Definition> {
        let mut definitions = vec![];
        let mut cursor = node.walk();

        for child in node.children(&mut cursor) {
            if child.kind() == "type_parameter" {
                // Look for type_identifier in type_parameter
                let mut param_cursor = child.walk();
                for param_child in child.children(&mut param_cursor) {
                    if param_child.kind() == "type_identifier" {
                        definitions.push(Definition::new(
                            &param_child,
                            self.source_code,
                            DefinitionType::TypeDefinition,
                        ));
                    }
                }
            }
        }

        definitions
    }
}
