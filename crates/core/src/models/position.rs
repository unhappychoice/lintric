use serde::{Deserialize, Serialize};
use std::fmt;
use tree_sitter::Node;

#[derive(Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct Position {
    pub start_line: usize,
    pub start_column: usize,
    pub end_line: usize,
    pub end_column: usize,
}

impl Position {
    pub fn from_node(node: &Node) -> Self {
        let start_point = node.start_position();
        let end_point = node.end_position();

        Position {
            start_line: start_point.row + 1,
            start_column: start_point.column + 1,
            end_line: end_point.row + 1,
            end_column: end_point.column + 1,
        }
    }

    pub fn line_number(&self) -> usize {
        self.start_line
    }

    /// Find the AST node at this position from the root node
    pub fn find_node_at_position<'a>(&self, root: Node<'a>) -> Option<Node<'a>> {
        find_node_at_position_recursive(root, self.start_line - 1, self.start_column - 1)
    }
}

impl fmt::Debug for Position {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{{ {}:{} to {}:{} }}",
            self.start_line, self.start_column, self.end_line, self.end_column
        )
    }
}

/// Recursively search for a node at the given position
fn find_node_at_position_recursive(
    node: Node,
    target_row: usize,
    target_col: usize,
) -> Option<Node> {
    let start = node.start_position();
    let end = node.end_position();

    // Check if the target position is within this node's range
    if target_row < start.row || target_row > end.row {
        return None;
    }

    if target_row == start.row && target_col < start.column {
        return None;
    }

    if target_row == end.row && target_col > end.column {
        return None;
    }

    // Try to find a more specific child node
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        if let Some(found) = find_node_at_position_recursive(child, target_row, target_col) {
            return Some(found);
        }
    }

    // If no child contains the position, return this node
    Some(node)
}
