use super::position::Position;
use serde::{Deserialize, Serialize};
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Usage {
    pub name: String,
    pub kind: UsageKind,
    pub position: Position,
}

impl Usage {
    pub fn new(node: &Node, source_code: &str, kind: UsageKind) -> Self {
        let name = node
            .utf8_text(source_code.as_bytes())
            .unwrap_or("")
            .trim()
            .replace("\r\n", "\n");

        Usage {
            name,
            kind,
            position: Position::from_node(node),
        }
    }

    // Helper function for testing
    pub fn new_simple(name: String, position: Position, kind: UsageKind) -> Self {
        Usage {
            name,
            kind,
            position,
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
        }
    }
}
