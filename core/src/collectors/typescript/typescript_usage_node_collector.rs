use crate::collectors::common::definition_context::{DefinitionContextChecker, DefinitionPattern};
use crate::collectors::common::usage_node_collector::UsageNodeCollector;
use crate::models::{Usage, UsageKind};
use tree_sitter::Node;

pub struct TypescriptUsageNodeCollector {
    definition_checker: DefinitionContextChecker,
}

impl TypescriptUsageNodeCollector {
    pub fn new(_source_code: &str) -> Self {
        let patterns = vec![
            // Variable declarations
            DefinitionPattern::new("variable_declarator", "name"),
            DefinitionPattern::new("required_parameter", "pattern"),
            DefinitionPattern::new("optional_parameter", "pattern"),
            // Type parameters
            DefinitionPattern::with_any_field("type_parameters"),
            DefinitionPattern::with_any_field("type_parameter"),
            // Named declarations
            DefinitionPattern::new("function_declaration", "name"),
            DefinitionPattern::new("class_declaration", "name"),
            DefinitionPattern::new("abstract_class_declaration", "name"),
            DefinitionPattern::new("interface_declaration", "name"),
            DefinitionPattern::new("type_alias_declaration", "name"),
        ];

        Self {
            definition_checker: DefinitionContextChecker::new(patterns),
        }
    }

    fn is_default_value_in_assignment_pattern<'a>(&self, node: Node<'a>) -> bool {
        // Check if this identifier is the default value (right side) in an assignment pattern
        if let Some(parent) = node.parent() {
            if parent.kind() == "object_assignment_pattern" || parent.kind() == "assignment_pattern"
            {
                // Check if this node is the right side (value field) of the assignment pattern
                if let Some(right_node) = parent.child_by_field_name("right") {
                    return node == right_node;
                }
            }
        }
        false
    }
}

impl UsageNodeCollector for TypescriptUsageNodeCollector {
    fn extract_node_if_usage(&self, node: Node, source_code: &str) -> Option<Usage> {
        let kind = match node.kind() {
            "identifier" => {
                // Special case: default values in object assignment patterns are always usage
                if self.is_default_value_in_assignment_pattern(node) {
                    Some(UsageKind::Identifier)
                }
                // Only treat identifier as usage if it's not in a definition context
                else if self
                    .definition_checker
                    .is_identifier_in_definition_context(node)
                {
                    None
                } else {
                    Some(UsageKind::Identifier)
                }
            }
            "type_identifier" => {
                // Only treat type_identifier as usage if it's not in a definition context
                if self
                    .definition_checker
                    .is_identifier_in_definition_context(node)
                {
                    None
                } else {
                    Some(UsageKind::TypeIdentifier)
                }
            }
            "call_expression" => Some(UsageKind::CallExpression),
            "property_identifier" => Some(UsageKind::FieldExpression),
            _ => None,
        };

        kind.map(|k| Usage::new(&node, source_code, k))
    }
}
