use crate::definition_context::{DefinitionContextChecker, DefinitionPattern};
use crate::models::{Usage, UsageKind};
use crate::usage_collector::UsageCollector;
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

impl UsageCollector for RustUsageNodeCollector {
    fn extract_node_if_usage(&self, node: Node, source_code: &str) -> Option<Usage> {
        let kind = match node.kind() {
            "identifier" => {
                // Only treat identifier as usage if it's not in a definition context
                // and not inside a call_expression
                if self
                    .definition_checker
                    .is_identifier_in_definition_context(node)
                    || self.is_identifier_in_call_expression(node)
                {
                    None
                } else if self.is_identifier_part_of_field_access(node, source_code) {
                    // Check if this identifier is part of a field access pattern like "self.field"
                    Some(UsageKind::FieldExpression)
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
            "call_expression" => {
                // Use special handling for call expressions to extract function name
                return Some(Usage::new_call_expression(&node, source_code));
            }
            "field_expression" => {
                // Use special handling for field expressions to extract field name
                return Some(Usage::new_field_expression(&node, source_code));
            }
            "struct_expression" => Some(UsageKind::StructExpression),
            "metavariable" => Some(UsageKind::Metavariable),
            _ => None,
        };

        kind.map(|k| Usage::new(&node, source_code, k))
    }
}

impl RustUsageNodeCollector {
    fn is_identifier_in_call_expression(&self, node: Node) -> bool {
        let mut current = node.parent();
        while let Some(parent) = current {
            match parent.kind() {
                "call_expression" => {
                    // Check if this identifier is the function name (first child of call_expression)
                    if let Some(function_node) = parent.child(0) {
                        return function_node.id() == node.id();
                    }
                    return false;
                }
                _ => current = parent.parent(),
            }
        }
        false
    }

    fn is_identifier_part_of_field_access(&self, node: Node, source_code: &str) -> bool {
        // Get the line containing this identifier
        let start_pos = node.start_position();
        let lines: Vec<&str> = source_code.lines().collect();
        if start_pos.row < lines.len() {
            let line = lines[start_pos.row];
            let col = start_pos.column;

            // Check if there's "self." before this identifier on the same line
            if col >= 5 && line.len() > col {
                let start_offset = col.saturating_sub(5);
                if let Some(preceding) = line.get(start_offset..col) {
                    if preceding == "self." {
                        return true;
                    }
                }
            }

            // Check if there's any identifier followed by dot before this identifier
            if col >= 2 && line.len() > col {
                let preceding_char = line.chars().nth(col - 1);
                if preceding_char == Some('.') {
                    return true;
                }
            }
        }

        false
    }
}
