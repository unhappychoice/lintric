use crate::models::{Definition, Dependency, Type, Usage, UsageKind};
use std::collections::HashMap;
use tree_sitter::Node;

#[derive(Debug, Clone)]
pub struct MethodResolutionResult {
    pub resolved_method: Definition,
    pub receiver_type: Type,
    pub confidence: f64,
}

#[derive(Debug, Clone)]
pub struct MethodResolver {
    class_methods: HashMap<String, Vec<Definition>>,
}

impl Default for MethodResolver {
    fn default() -> Self {
        Self::new()
    }
}

impl MethodResolver {
    pub fn new() -> Self {
        Self {
            class_methods: HashMap::new(),
        }
    }

    pub fn resolve_method_call(
        &self,
        usage: &Usage,
        _source_code: &str,
        _root_node: Node,
        definitions: &[Definition],
    ) -> Option<MethodResolutionResult> {
        // Basic method resolution for TypeScript
        // This is a simplified implementation that can be expanded

        if !usage.name.contains('.') {
            return None;
        }

        let method_name = usage.name.split('.').next_back()?;

        // Find matching method definitions
        for definition in definitions {
            if definition.name == method_name {
                if let Some(class_type) = self.infer_receiver_type(usage) {
                    return Some(MethodResolutionResult {
                        resolved_method: definition.clone(),
                        receiver_type: class_type,
                        confidence: 0.8,
                    });
                }
            }
        }

        None
    }

    fn infer_receiver_type(&self, _usage: &Usage) -> Option<Type> {
        // Simplified type inference for TypeScript
        // This would be expanded to handle actual TypeScript type analysis
        Some(Type::Concrete("any".to_string()))
    }

    pub fn add_class_methods(&mut self, class_name: String, methods: Vec<Definition>) {
        self.class_methods.insert(class_name, methods);
    }

    /// Resolve struct/interface field access dependencies for TypeScript
    pub fn resolve_struct_field_access(
        &self,
        usage_node: &Usage,
        definitions: &[Definition],
    ) -> Vec<Dependency> {
        let mut dependencies = Vec::new();

        // Only handle FieldExpression usage
        if usage_node.kind != UsageKind::FieldExpression {
            return dependencies;
        }

        // For field expressions like "obj.field", extract the field name "field"
        let field_name = if usage_node.name.contains('.') {
            usage_node
                .name
                .split('.')
                .next_back()
                .unwrap_or(&usage_node.name)
                .to_string()
        } else {
            usage_node.name.clone()
        };

        // Find interface/class field definitions by the extracted field name
        for definition in definitions {
            if definition.name == field_name
                && matches!(
                    definition.definition_type,
                    crate::models::DefinitionType::StructFieldDefinition
                        | crate::models::DefinitionType::PropertyDefinition
                )
            {
                let source_line = usage_node.position.start_line;
                let target_line = definition.position.start_line;

                // Don't create self-referential dependencies
                if source_line != target_line {
                    let dependency = Dependency {
                        source_line,
                        target_line,
                        symbol: field_name.clone(),
                        dependency_type: crate::models::DependencyType::StructFieldAccess,
                        context: Some("field_access".to_string()),
                    };
                    dependencies.push(dependency);
                }
            }
        }

        dependencies
    }
}
