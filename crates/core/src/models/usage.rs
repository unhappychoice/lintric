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
}
