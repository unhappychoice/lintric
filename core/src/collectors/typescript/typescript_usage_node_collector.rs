use crate::collectors::common::definition_context::{DefinitionContextChecker, DefinitionPattern};
use crate::collectors::common::usage_node_collector::UsageNodeCollector;
use crate::models::{Usage, UsageKind};
use tree_sitter::Node;

pub struct TypescriptUsageNodeCollector<'a> {
    source_code: &'a str,
    definition_checker: DefinitionContextChecker,
}

impl<'a> TypescriptUsageNodeCollector<'a> {
    pub fn new(source_code: &'a str) -> Self {
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
            source_code,
            definition_checker: DefinitionContextChecker::new(patterns),
        }
    }

    fn is_default_value_in_assignment_pattern(&self, node: Node<'a>) -> bool {
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

impl<'a> UsageNodeCollector<'a> for TypescriptUsageNodeCollector<'a> {
    fn extract_node_if_usage(
        &self,
        node: Node<'a>,
        current_scope: &Option<String>,
    ) -> Option<Usage<'a>> {
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

        kind.map(|k| Usage {
            node,
            kind: k,
            scope: current_scope.clone(),
        })
    }

    fn determine_scope(&self, node: &Node<'a>, parent_scope: &Option<String>) -> Option<String> {
        let new_scope_name = match node.kind() {
            "function_declaration" | "class_declaration" | "interface_declaration" | "module" => {
                node.child_by_field_name("name").map(|n| {
                    n.utf8_text(self.source_code.as_bytes())
                        .unwrap()
                        .trim()
                        .to_string()
                })
            }
            _ => None,
        };

        if let Some(name) = new_scope_name {
            Some(
                parent_scope
                    .as_ref()
                    .map_or(name.clone(), |p| format!("{p}.{name}")),
            )
        } else {
            parent_scope.clone()
        }
    }
}
