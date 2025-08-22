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
        }
    }

    fn get_context(&self, usage_node: &Usage) -> Option<String> {
        match usage_node.kind {
            UsageKind::CallExpression => Some("call_expression".to_string()),
            UsageKind::FieldExpression => Some("field_access".to_string()),
            UsageKind::StructExpression => Some("struct_instantiation".to_string()),
            UsageKind::Metavariable => Some("metavariable_use".to_string()),
            UsageKind::TypeIdentifier => Some("type_reference".to_string()),
            UsageKind::Identifier => Some("variable_use".to_string()),
            UsageKind::Read => Some("variable_read".to_string()),
        }
    }
}
