use serde::{Deserialize, Serialize};
use tree_sitter::Node;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum UsageKind {
    Identifier,
    CallExpression,
    FieldExpression,
    StructExpression,
    Metavariable,
}

#[derive(Debug, Clone)]
pub struct Usage<'a> {
    pub node: Node<'a>,
    pub kind: UsageKind,
    pub scope: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializableUsage {
    pub name: String,
    pub kind: UsageKind,
    pub scope: Option<String>,
    pub start_line: usize,
    pub start_column: usize,
    pub end_line: usize,
    pub end_column: usize,
}

impl<'a> Usage<'a> {
    pub fn to_serializable(&self, source_code: &str) -> SerializableUsage {
        let start_point = self.node.start_position();
        let end_point = self.node.end_position();
        let name = self
            .node
            .utf8_text(source_code.as_bytes())
            .unwrap_or("")
            .replace("\r\n", "\n");

        SerializableUsage {
            name,
            kind: self.kind.clone(),
            scope: self.scope.clone(),
            start_line: start_point.row + 1,
            start_column: start_point.column + 1,
            end_line: end_point.row + 1,
            end_column: end_point.column + 1,
        }
    }
}
