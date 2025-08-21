use crate::dependency_resolver::DependencyResolver;
use crate::models::{Definition, Dependency, Usage};
use tree_sitter::Node;

pub struct RustDependencyResolver;

impl RustDependencyResolver {
    pub fn new() -> Self {
        Self
    }
}

impl Default for RustDependencyResolver {
    fn default() -> Self {
        Self::new()
    }
}

impl DependencyResolver for RustDependencyResolver {
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
        _source_code: &str,
        _root_node: Node,
        usage_node: &Usage,
        definitions: &[Definition],
    ) -> Vec<Dependency> {
        let mut dependencies = Vec::new();

        // Simple name-based matching for now
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

        dependencies
    }
}
