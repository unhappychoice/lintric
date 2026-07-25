use super::receiver_narrowing::ReceiverNarrowing;
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

    /// Resolve `receiver.field` to the field the receiver's type declares.
    ///
    /// Matching on the field name alone links an access to every type declaring that name, so the
    /// candidates are narrowed by what the receiver is; see `ReceiverNarrowing`.
    pub fn resolve_struct_field_access(
        &self,
        usage_node: &Usage,
        definitions: &[Definition],
        narrowing: &ReceiverNarrowing,
    ) -> Vec<Dependency> {
        if usage_node.kind != UsageKind::FieldExpression {
            return Vec::new();
        }

        let field_name = Self::accessed_field_name(usage_node);
        let candidates = definitions
            .iter()
            .filter(|definition| definition.name == field_name && Self::is_member(definition))
            .collect();

        narrowing
            .narrow(usage_node, candidates)
            .into_iter()
            .filter(|definition| definition.position.start_line != usage_node.position.start_line)
            .map(|definition| Self::field_access(usage_node, definition, &field_name))
            .collect()
    }

    /// The last segment of `obj.field`, which is the member being read.
    fn accessed_field_name(usage_node: &Usage) -> String {
        usage_node
            .name
            .split('.')
            .next_back()
            .unwrap_or(&usage_node.name)
            .to_string()
    }

    fn is_member(definition: &Definition) -> bool {
        matches!(
            definition.definition_type,
            crate::models::DefinitionType::StructFieldDefinition
                | crate::models::DefinitionType::PropertyDefinition
        )
    }

    fn field_access(usage_node: &Usage, definition: &Definition, field_name: &str) -> Dependency {
        Dependency {
            source_line: usage_node.position.start_line,
            target_line: definition.position.start_line,
            symbol: field_name.to_string(),
            dependency_type: crate::models::DependencyType::StructFieldAccess,
            context: Some("field_access".to_string()),
        }
    }
}
