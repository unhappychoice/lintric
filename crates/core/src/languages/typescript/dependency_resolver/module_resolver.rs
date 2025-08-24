use crate::models::{Definition, Usage};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct ModuleResolver {
    imports: HashMap<String, String>, // import_name -> module_path
}

impl Default for ModuleResolver {
    fn default() -> Self {
        Self::new()
    }
}

impl ModuleResolver {
    pub fn new() -> Self {
        Self {
            imports: HashMap::new(),
        }
    }

    pub fn resolve_import(&self, usage: &Usage) -> Option<String> {
        self.imports.get(&usage.name).cloned()
    }

    pub fn add_import(&mut self, import_name: String, module_path: String) {
        self.imports.insert(import_name, module_path);
    }

    pub fn check_visibility(&self, _usage: &Usage, _definition: &Definition) -> bool {
        // TypeScript visibility checking
        // This is a simplified implementation
        true
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

    /// Check if two positions are within the same function scope (TypeScript-specific)
    pub fn are_in_same_function_scope(&self, usage: &Usage, definition: &Definition) -> bool {
        // TypeScript has different scope rules - for now, allow all
        // In a real implementation, this would handle TypeScript-specific scoping
        let _ = (usage, definition);
        true
    }

    /// Check if a dependency is valid according to TypeScript-specific rules
    pub fn is_valid_dependency(&self, usage: &Usage, definition: &Definition) -> bool {
        // TypeScript-specific dependency validation - for now, allow all
        // In a real implementation, this would handle TypeScript-specific rules like import/export
        let _ = (usage, definition);
        true
    }

    /// Select the most appropriate definition from multiple candidates (TypeScript-specific)
    pub fn select_preferred_definition<'a>(
        &self,
        usage_node: &Usage,
        matching_definitions: &[&'a Definition],
    ) -> Option<&'a Definition> {
        if matching_definitions.is_empty() {
            return None;
        }

        // For TypeScript, we could implement interface/type preference logic
        // For now, use simple closest accessible definition
        self.find_closest_by_line_ts(usage_node, matching_definitions)
    }

    fn find_closest_by_line_ts<'a>(
        &self,
        usage_node: &Usage,
        definitions: &[&'a Definition],
    ) -> Option<&'a Definition> {
        let mut closest_definition: Option<&'a Definition> = None;
        let mut closest_distance = usize::MAX;

        for definition in definitions {
            if definition.position.start_line <= usage_node.position.start_line {
                let distance = usage_node.position.start_line - definition.position.start_line;
                if distance < closest_distance {
                    closest_distance = distance;
                    closest_definition = Some(definition);
                }
            }
        }

        closest_definition.or_else(|| definitions.first().copied())
    }
}
