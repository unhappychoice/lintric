use crate::models::{
    scope::{ScopeId, SymbolTable},
    Definition, Type, Usage,
};
use tree_sitter::Node;

pub struct TypeScriptEnhancedResolver {
    #[allow(dead_code)]
    symbol_table: SymbolTable,
}

impl TypeScriptEnhancedResolver {
    pub fn new(symbol_table: SymbolTable) -> Self {
        Self { symbol_table }
    }

    /// Analyze TypeScript-specific type parameters and generics
    pub fn analyze_type_parameters(
        &mut self,
        source_code: &str,
        root_node: Node,
    ) -> Result<(), String> {
        let mut cursor = root_node.walk();
        self.traverse_and_analyze_type_parameters(source_code, root_node, &mut cursor)
    }

    fn traverse_and_analyze_type_parameters<'a>(
        &mut self,
        source_code: &str,
        node: Node<'a>,
        cursor: &mut tree_sitter::TreeCursor<'a>,
    ) -> Result<(), String> {
        match node.kind() {
            "function_declaration" | "method_definition" | "arrow_function" => {
                if let Some(type_params) = node.child_by_field_name("type_parameters") {
                    self.analyze_function_type_parameters(source_code, type_params)?;
                }
            }
            "class_declaration" | "interface_declaration" => {
                if let Some(type_params) = node.child_by_field_name("type_parameters") {
                    self.analyze_class_type_parameters(source_code, type_params)?;
                }
            }
            "type_alias_declaration" => {
                if let Some(type_params) = node.child_by_field_name("type_parameters") {
                    self.analyze_type_alias_parameters(source_code, type_params)?;
                }
            }
            _ => {}
        }

        // Recursively traverse children
        let children: Vec<_> = node.children(cursor).collect();
        for child in children {
            let mut child_cursor = child.walk();
            self.traverse_and_analyze_type_parameters(source_code, child, &mut child_cursor)?;
        }

        Ok(())
    }

    fn analyze_function_type_parameters(
        &mut self,
        source_code: &str,
        type_params_node: Node,
    ) -> Result<(), String> {
        let mut cursor = type_params_node.walk();
        for child in type_params_node.children(&mut cursor) {
            if child.kind() == "type_parameter" {
                self.process_type_parameter(source_code, child)?;
            }
        }
        Ok(())
    }

    fn analyze_class_type_parameters(
        &mut self,
        source_code: &str,
        type_params_node: Node,
    ) -> Result<(), String> {
        let mut cursor = type_params_node.walk();
        for child in type_params_node.children(&mut cursor) {
            if child.kind() == "type_parameter" {
                self.process_type_parameter(source_code, child)?;
            }
        }
        Ok(())
    }

    fn analyze_type_alias_parameters(
        &mut self,
        source_code: &str,
        type_params_node: Node,
    ) -> Result<(), String> {
        let mut cursor = type_params_node.walk();
        for child in type_params_node.children(&mut cursor) {
            if child.kind() == "type_parameter" {
                self.process_type_parameter(source_code, child)?;
            }
        }
        Ok(())
    }

    fn process_type_parameter(
        &mut self,
        source_code: &str,
        type_param_node: Node,
    ) -> Result<(), String> {
        let mut cursor = type_param_node.walk();
        let mut param_name = String::new();
        let mut constraint_type: Option<String> = None;
        let mut default_type: Option<String> = None;

        for child in type_param_node.children(&mut cursor) {
            match child.kind() {
                "type_identifier" => {
                    param_name = child
                        .utf8_text(source_code.as_bytes())
                        .map_err(|e| format!("Failed to extract type parameter name: {}", e))?
                        .to_string();
                }
                "constraint" => {
                    if let Some(constraint_child) = child.child(1) {
                        // Skip 'extends' keyword
                        constraint_type = Some(
                            constraint_child
                                .utf8_text(source_code.as_bytes())
                                .map_err(|e| format!("Failed to extract constraint type: {}", e))?
                                .to_string(),
                        );
                    }
                }
                "default_type" => {
                    if let Some(default_child) = child.child(1) {
                        // Skip '=' symbol
                        default_type = Some(
                            default_child
                                .utf8_text(source_code.as_bytes())
                                .map_err(|e| format!("Failed to extract default type: {}", e))?
                                .to_string(),
                        );
                    }
                }
                _ => {}
            }
        }

        // Store the type parameter information in symbol table
        if !param_name.is_empty() {
            self.symbol_table
                .add_type_parameter(param_name, constraint_type, default_type);
        }

        Ok(())
    }

    /// Resolve TypeScript interface types and inheritance
    pub fn resolve_interface_type(&self, usage: &Usage, scope_id: ScopeId) -> Option<Type> {
        // Look for interface definition in symbol table
        let interface_definitions = self
            .symbol_table
            .lookup_symbol_in_scope(&usage.name, scope_id);

        for definition in interface_definitions {
            if matches!(
                definition.definition_type,
                crate::models::DefinitionType::InterfaceDefinition
            ) {
                return self.build_interface_type(&definition.name, scope_id);
            }
        }

        // Check if this is a generic interface type
        if usage.name.contains("<") {
            return self.resolve_generic_interface_type(usage, scope_id);
        }

        None
    }

    fn build_interface_type(&self, interface_name: &str, _scope_id: ScopeId) -> Option<Type> {
        // For basic interfaces, return a concrete type
        Some(Type::Concrete(interface_name.to_string()))
    }

    fn resolve_generic_interface_type(&self, usage: &Usage, scope_id: ScopeId) -> Option<Type> {
        // Parse generic interface syntax like "Array<string>" or "Map<string, number>"
        if let Some(angle_start) = usage.name.find("<") {
            let base_name = usage.name[..angle_start].to_string();
            let type_args_str = &usage.name[angle_start + 1..usage.name.len() - 1];

            // Parse type arguments (simple comma-separated for now)
            let type_args: Vec<Type> = type_args_str
                .split(",")
                .map(|arg| self.parse_type_argument(arg.trim(), scope_id))
                .collect::<Option<Vec<_>>>()?;

            // Check if base interface exists
            let interface_definitions = self
                .symbol_table
                .lookup_symbol_in_scope(&base_name, scope_id);
            for definition in interface_definitions {
                if matches!(
                    definition.definition_type,
                    crate::models::DefinitionType::InterfaceDefinition
                ) {
                    return Some(Type::Generic(base_name, type_args));
                }
            }
        }

        None
    }

    fn parse_type_argument(&self, type_str: &str, scope_id: ScopeId) -> Option<Type> {
        match type_str {
            "string" | "number" | "boolean" | "void" | "any" | "unknown" | "never" => {
                Some(Type::Concrete(type_str.to_string()))
            }
            _ => {
                // Check if it's a type parameter
                if let Some(type_params) = self.symbol_table.lookup_type_parameter(type_str) {
                    if !type_params.is_empty() {
                        return Some(Type::TypeParameter(type_str.to_string()));
                    }
                }

                // Check if it's another interface or type
                let definitions = self.symbol_table.lookup_symbol_in_scope(type_str, scope_id);
                for definition in definitions {
                    match definition.definition_type {
                        crate::models::DefinitionType::InterfaceDefinition
                        | crate::models::DefinitionType::TypeDefinition
                        | crate::models::DefinitionType::ClassDefinition => {
                            return Some(Type::Concrete(type_str.to_string()));
                        }
                        _ => {}
                    }
                }

                // If nothing found, return as unknown
                Some(Type::Unknown)
            }
        }
    }

    /// Resolve interface inheritance chains
    pub fn resolve_interface_inheritance(
        &self,
        interface_name: &str,
        scope_id: ScopeId,
    ) -> Vec<String> {
        let mut inheritance_chain = Vec::new();
        let mut visited = std::collections::HashSet::new();

        self.collect_interface_inheritance(
            interface_name,
            scope_id,
            &mut inheritance_chain,
            &mut visited,
        );
        inheritance_chain
    }

    fn collect_interface_inheritance(
        &self,
        interface_name: &str,
        _scope_id: ScopeId,
        inheritance_chain: &mut Vec<String>,
        visited: &mut std::collections::HashSet<String>,
    ) {
        if visited.contains(interface_name) {
            return; // Avoid circular inheritance
        }

        visited.insert(interface_name.to_string());
        inheritance_chain.push(interface_name.to_string());

        // TODO: In a full implementation, we would parse the AST to find 'extends' clauses
        // For now, this is a placeholder that could be extended with actual inheritance parsing
    }

    /// Handle TypeScript module resolution and imports
    pub fn resolve_module_import(
        &self,
        usage: &Usage,
        definitions: &[Definition],
    ) -> Option<Definition> {
        // Handle different types of imports
        match usage.kind {
            crate::models::UsageKind::Identifier => self.resolve_named_import(usage, definitions),
            _ => self.resolve_generic_import(usage, definitions),
        }
    }

    fn resolve_named_import(
        &self,
        usage: &Usage,
        definitions: &[Definition],
    ) -> Option<Definition> {
        // Look for import definitions that match this usage
        for definition in definitions {
            if matches!(
                definition.definition_type,
                crate::models::DefinitionType::ImportDefinition
            ) {
                if definition.name == usage.name {
                    return Some(definition.clone());
                }

                // Handle aliased imports (import { foo as bar })
                if self.matches_aliased_import(&definition.name, &usage.name) {
                    return Some(definition.clone());
                }
            }
        }

        // Check for namespace imports (import * as name)
        if let Some(namespace_def) = self.resolve_namespace_import(usage, definitions) {
            return Some(namespace_def);
        }

        None
    }

    fn resolve_generic_import(
        &self,
        usage: &Usage,
        definitions: &[Definition],
    ) -> Option<Definition> {
        // Fallback for other usage types
        for definition in definitions {
            if matches!(
                definition.definition_type,
                crate::models::DefinitionType::ImportDefinition
            ) && definition.name == usage.name
            {
                return Some(definition.clone());
            }
        }
        None
    }

    fn matches_aliased_import(&self, import_name: &str, usage_name: &str) -> bool {
        // Simple check for aliased imports - in a full implementation,
        // this would parse the import statement to understand the alias structure
        import_name.contains(usage_name)
    }

    fn resolve_namespace_import(
        &self,
        usage: &Usage,
        definitions: &[Definition],
    ) -> Option<Definition> {
        // Handle namespace access like namespace.member
        if usage.name.contains(".") {
            let parts: Vec<&str> = usage.name.split(".").collect();
            if parts.len() >= 2 {
                let namespace_name = parts[0];

                for definition in definitions {
                    if matches!(
                        definition.definition_type,
                        crate::models::DefinitionType::ImportDefinition
                    ) && definition.name == namespace_name
                    {
                        return Some(definition.clone());
                    }
                }
            }
        }
        None
    }

    /// Resolve dynamic imports (import())
    pub fn resolve_dynamic_import(&self, import_path: &str) -> Option<String> {
        // Handle dynamic import resolution
        self.resolve_module_path(import_path)
    }

    /// Resolve module paths (relative, absolute, node_modules)
    pub fn resolve_module_path(&self, module_path: &str) -> Option<String> {
        if module_path.starts_with("./") || module_path.starts_with("../") {
            // Relative path
            Some(self.resolve_relative_path(module_path))
        } else if module_path.starts_with("/") {
            // Absolute path
            Some(module_path.to_string())
        } else {
            // Node modules or package imports
            Some(self.resolve_package_import(module_path))
        }
    }

    fn resolve_relative_path(&self, path: &str) -> String {
        // In a full implementation, this would resolve relative to the current file
        // For now, return the path as-is
        path.to_string()
    }

    fn resolve_package_import(&self, package_name: &str) -> String {
        // In a full implementation, this would look up the package in node_modules
        // and resolve to the main entry point or specific subpath
        format!("node_modules/{}", package_name)
    }

    /// Handle barrel exports (re-exports)
    pub fn resolve_barrel_export(
        &self,
        usage: &Usage,
        definitions: &[Definition],
    ) -> Vec<Definition> {
        let mut resolved_exports = Vec::new();

        // Look for export statements that might re-export the symbol
        for definition in definitions {
            if definition.name == usage.name {
                match definition.definition_type {
                    crate::models::DefinitionType::ImportDefinition => {
                        // This might be a re-export
                        resolved_exports.push(definition.clone());
                    }
                    _ => {
                        resolved_exports.push(definition.clone());
                    }
                }
            }
        }

        resolved_exports
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{Definition, DefinitionType, Position, Usage, UsageKind};

    fn create_test_symbol_table() -> SymbolTable {
        SymbolTable::new()
    }

    fn create_test_position() -> Position {
        Position {
            start_line: 1,
            start_column: 1,
            end_line: 1,
            end_column: 1,
        }
    }

    fn create_test_interface_definition(name: &str) -> Definition {
        Definition {
            name: name.to_string(),
            position: create_test_position(),
            definition_type: DefinitionType::InterfaceDefinition,
        }
    }

    fn create_test_import_definition(name: &str) -> Definition {
        Definition {
            name: name.to_string(),
            position: create_test_position(),
            definition_type: DefinitionType::ImportDefinition,
        }
    }

    #[test]
    fn test_resolve_interface_type_basic() {
        let symbol_table = create_test_symbol_table();
        let resolver = TypeScriptEnhancedResolver::new(symbol_table);

        let usage = Usage {
            name: "TestInterface".to_string(),
            kind: UsageKind::TypeIdentifier,
            position: create_test_position(),
        };

        // This test demonstrates the structure - in a real test,
        // we would need to set up the symbol table properly
        let result = resolver.resolve_interface_type(&usage, 0);
        // For now, this should return None since symbol table is empty
        assert!(result.is_none());
    }

    #[test]
    fn test_resolve_generic_interface_type() {
        let symbol_table = create_test_symbol_table();
        let resolver = TypeScriptEnhancedResolver::new(symbol_table);

        let usage = Usage {
            name: "Array<string>".to_string(),
            kind: UsageKind::TypeIdentifier,
            position: create_test_position(),
        };

        let result = resolver.resolve_interface_type(&usage, 0);
        // Should return None since Array interface is not in our symbol table
        assert!(result.is_none());
    }

    #[test]
    fn test_resolve_module_import_named() {
        let symbol_table = create_test_symbol_table();
        let resolver = TypeScriptEnhancedResolver::new(symbol_table);

        let usage = Usage {
            name: "testFunction".to_string(),
            kind: UsageKind::Identifier,
            position: create_test_position(),
        };

        let definitions = vec![
            create_test_import_definition("testFunction"),
            create_test_interface_definition("TestInterface"),
        ];

        let result = resolver.resolve_module_import(&usage, &definitions);
        assert!(result.is_some());
        assert_eq!(result.unwrap().name, "testFunction");
    }

    #[test]
    fn test_resolve_namespace_import() {
        let symbol_table = create_test_symbol_table();
        let resolver = TypeScriptEnhancedResolver::new(symbol_table);

        let usage = Usage {
            name: "myNamespace.someFunction".to_string(),
            kind: UsageKind::Identifier,
            position: create_test_position(),
        };

        let definitions = vec![create_test_import_definition("myNamespace")];

        let result = resolver.resolve_module_import(&usage, &definitions);
        assert!(result.is_some());
        assert_eq!(result.unwrap().name, "myNamespace");
    }

    #[test]
    fn test_resolve_module_path_relative() {
        let symbol_table = create_test_symbol_table();
        let resolver = TypeScriptEnhancedResolver::new(symbol_table);

        let result = resolver.resolve_module_path("./relative/path");
        assert_eq!(result, Some("./relative/path".to_string()));
    }

    #[test]
    fn test_resolve_module_path_package() {
        let symbol_table = create_test_symbol_table();
        let resolver = TypeScriptEnhancedResolver::new(symbol_table);

        let result = resolver.resolve_module_path("lodash");
        assert_eq!(result, Some("node_modules/lodash".to_string()));
    }

    #[test]
    fn test_parse_type_argument_primitive() {
        let symbol_table = create_test_symbol_table();
        let resolver = TypeScriptEnhancedResolver::new(symbol_table);

        let result = resolver.parse_type_argument("string", 0);
        assert_eq!(result, Some(Type::Concrete("string".to_string())));

        let result = resolver.parse_type_argument("number", 0);
        assert_eq!(result, Some(Type::Concrete("number".to_string())));

        let result = resolver.parse_type_argument("boolean", 0);
        assert_eq!(result, Some(Type::Concrete("boolean".to_string())));
    }

    #[test]
    fn test_interface_inheritance_basic() {
        let symbol_table = create_test_symbol_table();
        let resolver = TypeScriptEnhancedResolver::new(symbol_table);

        let result = resolver.resolve_interface_inheritance("TestInterface", 0);
        assert_eq!(result, vec!["TestInterface".to_string()]);
    }
}
