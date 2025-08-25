use super::position::Position;
use serde::{Deserialize, Serialize};
use std::fmt;
use tree_sitter::Node;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum UsageKind {
    Identifier,
    TypeIdentifier,
    CallExpression,
    FieldExpression,
    StructExpression,
    Metavariable,
    Read, // Added for testing
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Usage {
    pub name: String,
    pub kind: UsageKind,
    pub position: Position,
    pub context: Option<String>,
}

impl Usage {
    pub fn new(node: &Node, source_code: &str, kind: UsageKind) -> Self {
        let name = node
            .utf8_text(source_code.as_bytes())
            .unwrap_or("")
            .trim()
            .replace("\r\n", "\n");

        // Check if this node is part of a scoped_identifier
        let context = Self::get_node_context(node);

        Usage {
            name,
            kind,
            position: Position::from_node(node),
            context,
        }
    }

    fn get_node_context(node: &Node) -> Option<String> {
        let mut current = node.parent();
        while let Some(parent) = current {
            match parent.kind() {
                "scoped_identifier" => return Some("scoped_identifier".to_string()),
                "field_expression" => return Some("field_expression".to_string()),
                "call_expression" => return Some("call_expression".to_string()),
                _ => current = parent.parent(),
            }
        }
        None
    }

    // Helper function for testing
    pub fn new_simple(name: String, position: Position, kind: UsageKind) -> Self {
        Usage {
            name,
            kind,
            position,
            context: None,
        }
    }

    pub fn new_call_expression(node: &Node, source_code: &str) -> Self {
        // Extract function name from call_expression by getting the first child (function)
        let function_name = if let Some(function_node) = node.child(0) {
            function_node
                .utf8_text(source_code.as_bytes())
                .unwrap_or("")
                .trim()
                .replace("\r\n", "\n")
        } else {
            // Fallback to full text if we can't get the function child
            node.utf8_text(source_code.as_bytes())
                .unwrap_or("")
                .trim()
                .replace("\r\n", "\n")
        };

        Usage {
            name: function_name,
            kind: UsageKind::CallExpression,
            position: Position::from_node(node),
            context: Some("call_expression".to_string()),
        }
    }

    pub fn new_field_expression(node: &Node, source_code: &str) -> Self {
        // Extract field name from field_expression by getting the field child (usually the last child)
        // For "obj.field", we want just "field"
        let field_name = if let Some(field_node) = node.child_by_field_name("field") {
            field_node
                .utf8_text(source_code.as_bytes())
                .unwrap_or("")
                .trim()
                .replace("\r\n", "\n")
        } else if let Some(last_child) = node.child(node.child_count().saturating_sub(1)) {
            // Fallback: try the last child
            last_child
                .utf8_text(source_code.as_bytes())
                .unwrap_or("")
                .trim()
                .replace("\r\n", "\n")
        } else {
            // Final fallback to full text
            node.utf8_text(source_code.as_bytes())
                .unwrap_or("")
                .trim()
                .replace("\r\n", "\n")
        };

        Usage {
            name: field_name,
            kind: UsageKind::FieldExpression,
            position: Position::from_node(node),
            context: Some("field_expression".to_string()),
        }
    }
}

impl fmt::Debug for Usage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Usage {{ position: {:?}, name: {:?}, kind: {:?}, context: {:?} }}",
            self.position, self.name, self.kind, self.context
        )
    }
}
