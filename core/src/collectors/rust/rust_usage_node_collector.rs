use crate::collectors::common::definition_context::{DefinitionContextChecker, DefinitionPattern};
use crate::collectors::common::usage_node_collector::UsageNodeCollector;
use crate::models::{Usage, UsageKind};
use tree_sitter::Node;

pub struct RustUsageNodeCollector {
    definition_checker: DefinitionContextChecker,
}

impl RustUsageNodeCollector {
    pub fn new(_source_code: &str) -> Self {
        let patterns = vec![
            // Variable declarations
            DefinitionPattern::new("let_declaration", "pattern"),
            DefinitionPattern::new("parameter", "pattern"),
            DefinitionPattern::new("for_expression", "pattern"),
            // Closure parameters - definition context
            DefinitionPattern::with_any_field("closure_parameters"),
            // Type parameters - both contexts
            DefinitionPattern::with_any_field("type_parameters"),
            // Lifetime parameters - both contexts
            DefinitionPattern::with_any_field("lifetime"),
            // Named items
            DefinitionPattern::new("function_item", "name"),
            DefinitionPattern::new("struct_item", "name"),
            DefinitionPattern::new("enum_item", "name"),
            DefinitionPattern::new("trait_item", "name"),
            DefinitionPattern::new("mod_item", "name"),
            DefinitionPattern::new("const_item", "name"),
            DefinitionPattern::new("static_item", "name"),
            DefinitionPattern::new("type_item", "name"),
            DefinitionPattern::new("associated_type", "name"),
        ];

        Self {
            definition_checker: DefinitionContextChecker::new(patterns),
        }
    }
}

impl UsageNodeCollector for RustUsageNodeCollector {
    fn extract_node_if_usage(&self, node: Node, source_code: &str) -> Option<Usage> {
        let kind = match node.kind() {
            "identifier" => {
                // Only treat identifier as usage if it's not in a definition context
                if self
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
            "field_expression" => Some(UsageKind::FieldExpression),
            "struct_expression" => Some(UsageKind::StructExpression),
            "metavariable" => Some(UsageKind::Metavariable),
            _ => None,
        };

        kind.map(|k| Usage::new(&node, source_code, k))
    }
}
