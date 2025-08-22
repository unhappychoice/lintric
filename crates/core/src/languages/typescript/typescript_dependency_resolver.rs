use crate::dependency_resolver::DependencyResolver;
use crate::models::{Definition, Dependency, Usage, UsageKind};
use tree_sitter::Node;

pub struct TypescriptDependencyResolver;

impl TypescriptDependencyResolver {
    pub fn new() -> Self {
        Self
    }
}

impl Default for TypescriptDependencyResolver {
    fn default() -> Self {
        Self::new()
    }
}

impl DependencyResolver for TypescriptDependencyResolver {
    fn resolve_dependencies(
        &self,
        source_code: &str,
        root_node: Node,
        usage_nodes: &[Usage],
        definitions: &[Definition],
    ) -> Result<Vec<Dependency>, String> {
        let mut all_dependencies = Vec::new();

        for usage_node in usage_nodes {
            let mut deps =
                self.resolve_single_dependency(source_code, root_node, usage_node, definitions);
            all_dependencies.append(&mut deps);
        }

        Ok(all_dependencies)
    }

    fn resolve_single_dependency(
        &self,
        source_code: &str,
        root_node: Node,
        usage_node: &Usage,
        definitions: &[Definition],
    ) -> Vec<Dependency> {
        let mut dependencies = Vec::new();

        match usage_node.kind {
            UsageKind::CallExpression => {
                // Handle call expressions like add(1, 2)
                dependencies.extend(self.resolve_call_expression_dependency(
                    source_code,
                    root_node,
                    usage_node,
                    definitions,
                ));
            }
            _ => {
                // Simple name-based matching for other cases
                if let Some(def) = definitions.iter().find(|d| d.name == usage_node.name) {
                    let source_line = usage_node.position.line_number();
                    let target_line = def.line_number();

                    // Don't create self-referential dependencies and check scope accessibility
                    if source_line != target_line && self.is_accessible(usage_node, def) {
                        dependencies.push(Dependency {
                            source_line,
                            target_line,
                            symbol: usage_node.name.clone(),
                            dependency_type: self.get_dependency_type(usage_node),
                            context: self.get_context(usage_node),
                        });
                    }
                }
            }
        }

        dependencies
    }
}

impl TypescriptDependencyResolver {
    fn is_accessible(&self, usage: &Usage, definition: &Definition) -> bool {
        // Check for hoisting rules first
        if self.is_hoisted(definition) {
            return true; // Hoisted definitions are always accessible
        }

        // Basic position check: definition must come before usage
        if definition.position.start_line < usage.position.start_line {
            return true;
        }

        if definition.position.start_line == usage.position.start_line {
            return definition.position.start_column < usage.position.start_column;
        }

        // Definition comes after usage - not accessible for basic forward reference check
        false
    }

    fn is_hoisted(&self, definition: &Definition) -> bool {
        use crate::models::DefinitionType;
        match definition.definition_type {
            // In JavaScript/TypeScript, function declarations are hoisted
            DefinitionType::FunctionDefinition => true,
            // var declarations are also hoisted (but not let/const)
            DefinitionType::VariableDefinition => {
                // Note: We can't distinguish var from let/const without more context
                // For now, assume all variable definitions follow position-based rules
                // This could be improved by checking the actual declaration syntax
                false
            }
            // Type definitions are also accessible from anywhere in TypeScript
            DefinitionType::TypeDefinition => true,
            DefinitionType::InterfaceDefinition => true,
            DefinitionType::ClassDefinition => true,
            DefinitionType::EnumDefinition => true,
            _ => false,
        }
    }

    fn resolve_call_expression_dependency(
        &self,
        _source_code: &str,
        _root_node: Node,
        usage_node: &Usage,
        definitions: &[Definition],
    ) -> Vec<Dependency> {
        let mut dependencies = Vec::new();

        // The usage_node.name should now contain only the function name (extracted during usage collection)
        let function_name = &usage_node.name;

        // Look for function definition or import with this name
        if let Some(function_def) = definitions.iter().find(|d| {
            d.name == *function_name
                && matches!(
                    d.definition_type,
                    crate::models::DefinitionType::FunctionDefinition
                        | crate::models::DefinitionType::ImportDefinition
                )
        }) {
            let source_line = usage_node.position.line_number();
            let target_line = function_def.line_number();

            // Don't create self-referential dependencies and check scope accessibility
            if source_line != target_line && self.is_accessible(usage_node, function_def) {
                dependencies.push(Dependency {
                    source_line,
                    target_line,
                    symbol: function_name.to_string(),
                    dependency_type: self.get_dependency_type(usage_node),
                    context: self.get_context(usage_node),
                });
            }
        }

        dependencies
    }
}
