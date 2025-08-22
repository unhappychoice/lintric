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

                    // Don't create self-referential dependencies
                    if source_line != target_line {
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

            // Don't create self-referential dependencies
            if source_line != target_line {
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
