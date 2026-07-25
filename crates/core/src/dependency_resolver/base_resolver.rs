use crate::models::{Definition, DefinitionType, Dependency, DependencyType, Usage, UsageKind};
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

    /// Classify a resolved dependency.
    ///
    /// What a usage resolves to is usually the stronger signal: a method call and a field read are
    /// the same `field_expression` at the usage site, and only the definition distinguishes them.
    ///
    /// Call syntax is the exception. A call is a call whatever it resolves to, so calling a
    /// binding that holds a closure stays a `FunctionCall` rather than becoming a variable use.
    fn get_dependency_type(&self, usage_node: &Usage, definition: &Definition) -> DependencyType {
        match usage_node.kind {
            UsageKind::CallExpression | UsageKind::Call => DependencyType::FunctionCall,
            _ => dependency_type_of_definition(&definition.definition_type)
                .unwrap_or_else(|| dependency_type_of_usage(&usage_node.kind)),
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

fn dependency_type_of_definition(definition_type: &DefinitionType) -> Option<DependencyType> {
    match definition_type {
        DefinitionType::FunctionDefinition | DefinitionType::MethodDefinition => {
            Some(DependencyType::FunctionCall)
        }
        DefinitionType::StructFieldDefinition | DefinitionType::PropertyDefinition => {
            Some(DependencyType::StructFieldAccess)
        }
        DefinitionType::EnumVariantDefinition => Some(DependencyType::EnumVariantReference),
        DefinitionType::StructDefinition
        | DefinitionType::EnumDefinition
        | DefinitionType::TypeDefinition
        | DefinitionType::ClassDefinition
        | DefinitionType::InterfaceDefinition => Some(DependencyType::TypeReference),
        DefinitionType::ModuleDefinition | DefinitionType::Module => {
            Some(DependencyType::ModuleReference)
        }
        // Referencing an imported name is an ordinary reference; `Import` describes the edge from
        // the `use` statement to what it imports, which `resolve_import_dependencies` builds.
        DefinitionType::ImportDefinition => None,
        DefinitionType::MacroDefinition => Some(DependencyType::MacroInvocation),
        DefinitionType::MacroVariableDefinition => Some(DependencyType::MacroVariable),
        DefinitionType::VariableDefinition | DefinitionType::ConstDefinition => {
            Some(DependencyType::VariableUse)
        }
        DefinitionType::Other(_) => None,
    }
}

fn dependency_type_of_usage(kind: &UsageKind) -> DependencyType {
    match kind {
        UsageKind::Identifier => DependencyType::VariableUse,
        UsageKind::TypeIdentifier => DependencyType::TypeReference,
        UsageKind::CallExpression => DependencyType::FunctionCall,
        UsageKind::FieldExpression => DependencyType::StructFieldAccess,
        UsageKind::FieldInitializer => DependencyType::StructFieldAccess,
        UsageKind::StructExpression => DependencyType::TypeReference,
        UsageKind::Metavariable => DependencyType::MacroVariable,
        UsageKind::Read => DependencyType::VariableUse,
        // Keep these for backward compatibility, but they should not be used in new code
        UsageKind::Reference => DependencyType::VariableUse,
        UsageKind::Call => DependencyType::FunctionCall,
        UsageKind::FieldAccess => DependencyType::StructFieldAccess,
    }
}
