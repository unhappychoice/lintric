use crate::definition_collectors::{find_identifier_nodes_in_node, DefinitionCollector};
use crate::models::{
    Accessibility, Definition, DefinitionType, Position, ScopeId, ScopeType, SymbolTable,
};
use tree_sitter::Node;

pub struct TypescriptDefinitionCollector<'a> {
    _source_code: &'a str,
}

struct TypeScriptCollector<'a> {
    source_code: &'a str,
    symbol_table: SymbolTable,
    current_scope: ScopeId,
}

impl<'a> TypeScriptCollector<'a> {
    fn new(source_code: &'a str) -> Self {
        let symbol_table = SymbolTable::new();
        let current_scope = symbol_table.scopes.root;

        Self {
            source_code,
            symbol_table,
            current_scope,
        }
    }

    fn collect_from_node(&mut self, root: Node<'a>) -> Result<(), String> {
        self.visit_node(root)?;
        Ok(())
    }

    fn visit_node(&mut self, node: Node<'a>) -> Result<(), String> {
        let node_type = node.kind();
        let original_scope = self.current_scope;

        self.visit_ts_node(node, node_type)?;

        // Visit children
        for i in 0..node.child_count() {
            if let Some(child) = node.child(i) {
                self.visit_node(child)?;
            }
        }

        // Restore original scope
        if matches!(
            node_type,
            "function_declaration"
                | "arrow_function"
                | "method_definition"
                | "class_declaration"
                | "interface_declaration"
                | "namespace_declaration"
                | "module_declaration"
                | "internal_module"
                | "block"
                | "for_statement"
                | "while_statement"
                | "if_statement"
        ) {
            self.current_scope = original_scope;
        }

        Ok(())
    }

    fn visit_ts_node(&mut self, node: Node<'a>, node_type: &str) -> Result<(), String> {
        match node_type {
            "function_declaration" | "arrow_function" => {
                self.handle_function_scope(node)?;
            }
            "method_definition" => {
                self.handle_method_scope(node)?;
            }
            "class_declaration" => {
                self.handle_class_scope(node)?;
            }
            "interface_declaration" => {
                self.handle_interface_scope(node)?;
            }
            "namespace_declaration" | "module_declaration" | "internal_module" => {
                self.handle_namespace_scope(node)?;
            }
            "block" | "for_statement" | "while_statement" | "if_statement" => {
                self.handle_block_scope(node)?;
            }
            _ => {
                // Collect definitions without creating scope
                self.collect_definitions_for_node(node);
            }
        }

        Ok(())
    }

    fn handle_function_scope(&mut self, node: Node<'a>) -> Result<(), String> {
        let position = Position::from_node(&node);
        let scope_id = self.symbol_table.scopes.create_scope(
            Some(self.current_scope),
            ScopeType::Function,
            position,
        );
        self.current_scope = scope_id;

        self.collect_function_definitions(node, scope_id);
        Ok(())
    }

    fn handle_method_scope(&mut self, node: Node<'a>) -> Result<(), String> {
        let position = Position::from_node(&node);
        let scope_id = self.symbol_table.scopes.create_scope(
            Some(self.current_scope),
            ScopeType::Function,
            position,
        );
        self.current_scope = scope_id;

        self.collect_method_definitions(node, scope_id);
        Ok(())
    }

    fn handle_class_scope(&mut self, node: Node<'a>) -> Result<(), String> {
        let position = Position::from_node(&node);
        let scope_id = self.symbol_table.scopes.create_scope(
            Some(self.current_scope),
            ScopeType::Class,
            position,
        );
        self.current_scope = scope_id;

        self.collect_class_definitions(node, scope_id);
        Ok(())
    }

    fn handle_interface_scope(&mut self, node: Node<'a>) -> Result<(), String> {
        let position = Position::from_node(&node);
        let scope_id = self.symbol_table.scopes.create_scope(
            Some(self.current_scope),
            ScopeType::Interface,
            position,
        );
        self.current_scope = scope_id;

        self.collect_interface_definitions(node, scope_id);
        Ok(())
    }

    fn handle_namespace_scope(&mut self, node: Node<'a>) -> Result<(), String> {
        let position = Position::from_node(&node);
        let scope_id = self.symbol_table.scopes.create_scope(
            Some(self.current_scope),
            ScopeType::Module,
            position,
        );
        self.current_scope = scope_id;

        self.collect_namespace_definitions(node, scope_id);
        Ok(())
    }

    fn handle_block_scope(&mut self, node: Node<'a>) -> Result<(), String> {
        let position = Position::from_node(&node);
        let scope_id = self.symbol_table.scopes.create_scope(
            Some(self.current_scope),
            ScopeType::Block,
            position,
        );
        self.current_scope = scope_id;
        Ok(())
    }

    fn collect_definitions_for_node(&mut self, node: Node<'a>) {
        let definitions = match node.kind() {
            "variable_declarator" => self.collect_variable_definitions(node),
            "type_alias_declaration" => self.collect_type_alias_definitions(node),
            "import_statement" | "export_statement" => self.collect_import_definitions(node),
            "public_field_definition" | "private_field_definition" | "field_definition" => {
                self.collect_class_field_definitions(node)
            }
            "property_signature" | "method_signature" => {
                self.collect_interface_member_definitions(node)
            }
            _ => vec![],
        };

        for mut definition in definitions {
            definition.set_context(self.current_scope, &Accessibility::ScopeLocal, false);
            self.symbol_table
                .add_enhanced_symbol(definition.name.clone(), definition);
        }
    }

    fn collect_function_definitions(&mut self, node: Node<'a>, scope_id: ScopeId) {
        let mut definitions = vec![];

        // Function name
        if let Some(def) = Definition::from_naming_node(
            &node,
            self.source_code,
            DefinitionType::FunctionDefinition,
        ) {
            definitions.push(def);
        }

        // Parameters
        if let Some(params_node) = node.child_by_field_name("parameters") {
            definitions.extend(self.collect_function_parameters(params_node));
        }

        // Type parameters
        if let Some(type_params) = node.child_by_field_name("type_parameters") {
            definitions.extend(self.collect_type_parameters(type_params));
        }

        for mut def in definitions {
            def.set_context(scope_id, &Accessibility::ScopeLocal, false);
            self.symbol_table.add_enhanced_symbol(def.name.clone(), def);
        }
    }

    fn collect_method_definitions(&mut self, node: Node<'a>, scope_id: ScopeId) {
        let mut definitions = vec![];

        if let Some(def) =
            Definition::from_naming_node(&node, self.source_code, DefinitionType::MethodDefinition)
        {
            definitions.push(def);
        }

        if let Some(params_node) = node.child_by_field_name("parameters") {
            definitions.extend(self.collect_function_parameters(params_node));
        }

        for mut def in definitions {
            def.set_context(scope_id, &Accessibility::ScopeLocal, false);
            self.symbol_table.add_enhanced_symbol(def.name.clone(), def);
        }
    }

    fn collect_class_definitions(&mut self, node: Node<'a>, scope_id: ScopeId) {
        let mut definitions = vec![];

        if let Some(def) =
            Definition::from_naming_node(&node, self.source_code, DefinitionType::ClassDefinition)
        {
            definitions.push(def);
        }

        if let Some(type_params) = node.child_by_field_name("type_parameters") {
            definitions.extend(self.collect_type_parameters(type_params));
        }

        for mut def in definitions {
            def.set_context(scope_id, &Accessibility::ScopeLocal, false);
            self.symbol_table.add_enhanced_symbol(def.name.clone(), def);
        }
    }

    fn collect_interface_definitions(&mut self, node: Node<'a>, scope_id: ScopeId) {
        let mut definitions = vec![];

        if let Some(def) = Definition::from_naming_node(
            &node,
            self.source_code,
            DefinitionType::InterfaceDefinition,
        ) {
            definitions.push(def);
        }

        if let Some(type_params) = node.child_by_field_name("type_parameters") {
            definitions.extend(self.collect_type_parameters(type_params));
        }

        for mut def in definitions {
            def.set_context(scope_id, &Accessibility::ScopeLocal, false);
            self.symbol_table.add_enhanced_symbol(def.name.clone(), def);
        }
    }

    fn collect_namespace_definitions(&mut self, node: Node<'a>, scope_id: ScopeId) {
        if let Some(mut def) =
            Definition::from_naming_node(&node, self.source_code, DefinitionType::ModuleDefinition)
        {
            // Namespace definition should be added to parent scope, not the created namespace scope
            let parent_scope = self
                .symbol_table
                .scopes
                .get_scope(scope_id)
                .unwrap()
                .parent
                .unwrap_or(self.symbol_table.scopes.root);
            def.set_context(parent_scope, &Accessibility::ScopeLocal, false);
            self.symbol_table.add_enhanced_symbol(def.name.clone(), def);
        }
    }

    // Helper methods from original TypescriptDefinitionCollector
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

    fn collect_type_alias_definitions(&self, node: Node<'a>) -> Vec<Definition> {
        let mut definitions = vec![];

        if let Some(def) =
            Definition::from_naming_node(&node, self.source_code, DefinitionType::TypeDefinition)
        {
            definitions.push(def);
        }

        if let Some(type_params) = node.child_by_field_name("type_parameters") {
            definitions.extend(self.collect_type_parameters(type_params));
        }

        definitions
    }

    fn collect_import_definitions(&self, node: Node<'a>) -> Vec<Definition> {
        let mut definitions = vec![];
        let mut cursor = node.walk();

        for child in node.children(&mut cursor) {
            definitions.extend(self.collect_import_identifiers(child));
        }

        definitions
    }

    fn collect_class_field_definitions(&self, node: Node<'a>) -> Vec<Definition> {
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
                // Find the identifier after "as"
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
            "import_clause" => {
                // Handle default imports - direct identifier in import_clause
                for child in node.children(&mut cursor) {
                    if child.kind() == "identifier" {
                        definitions.push(Definition::new(
                            &child,
                            self.source_code,
                            DefinitionType::ImportDefinition,
                        ));
                    } else {
                        // Recursively process other children (named_imports, namespace_import)
                        definitions.extend(self.collect_import_identifiers(child));
                    }
                }
            }
            _ => {
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

impl<'a> TypescriptDefinitionCollector<'a> {
    pub fn new(source_code: &'a str) -> Self {
        Self {
            _source_code: source_code,
        }
    }
}

impl<'a> DefinitionCollector<'a> for TypescriptDefinitionCollector<'a> {
    fn collect(&self, source_code: &str, root: Node<'a>) -> Result<SymbolTable, String> {
        let mut collector = TypeScriptCollector::new(source_code);
        collector.collect_from_node(root)?;
        Ok(collector.symbol_table)
    }
}
