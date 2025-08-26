use crate::models::{Definition, Dependency, DependencyType, Usage, UsageKind};
use tree_sitter::Node;

pub trait DependencyResolver: Send + Sync {
    fn resolve_dependencies(
        &self,
        source_code: &str,
        root_node: Node,
        usage_nodes: &[Usage],
        definitions: &[Definition],
    ) -> Result<Vec<Dependency>, String>;

    fn resolve_single_dependency(
        &self,
        source_code: &str,
        root_node: Node,
        usage_node: &Usage,
        definitions: &[Definition],
    ) -> Vec<Dependency>;

    fn get_dependency_type(&self, usage_node: &Usage) -> DependencyType {
        match usage_node.kind {
            UsageKind::Identifier => DependencyType::VariableUse,
            UsageKind::TypeIdentifier => DependencyType::TypeReference,
            UsageKind::CallExpression => DependencyType::FunctionCall,
            UsageKind::FieldExpression => DependencyType::StructFieldAccess,
            UsageKind::StructExpression => DependencyType::TypeReference,
            UsageKind::Metavariable => DependencyType::MacroVariable,
            UsageKind::Read => DependencyType::VariableUse,
            // Keep these for backward compatibility, but they should not be used in new code
            UsageKind::Reference => DependencyType::VariableUse,
            UsageKind::Call => DependencyType::FunctionCall,
            UsageKind::FieldAccess => DependencyType::StructFieldAccess,
        }
    }

    fn get_context(&self, usage_node: &Usage) -> Option<String> {
        // Default implementation, can be overridden
        Some(format!(
            "{:?}:{}:{}",
            usage_node.kind, usage_node.position.start_line, usage_node.position.start_column
        ))
    }
}
