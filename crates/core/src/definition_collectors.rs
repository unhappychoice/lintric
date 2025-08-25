use crate::models::SymbolTable;
use tree_sitter::Node;

pub trait DefinitionCollector<'a>: Send + Sync {
    fn collect(&self, source_code: &str, root: Node<'a>) -> Result<SymbolTable, String>;
}

pub fn find_identifier_nodes_in_node<'a>(node: Node<'a>) -> Vec<Node<'a>> {
    let mut identifiers = vec![];
    find_identifier_nodes_recursive(node, &mut identifiers);
    identifiers
}

fn find_identifier_nodes_recursive<'a>(node: Node<'a>, identifiers: &mut Vec<Node<'a>>) {
    match node.kind() {
        "identifier" => {
            identifiers.push(node);
        }
        "shorthand_property_identifier_pattern" => {
            // For TypeScript/TSX object destructuring patterns like { prop }
            // The shorthand_property_identifier_pattern node itself represents the identifier
            identifiers.push(node);
        }
        "object_assignment_pattern" => {
            // For TypeScript/TSX patterns like { prop = defaultValue }
            // Only extract the property name (left side), not the default value (right side)
            if let Some(left_node) = node.child_by_field_name("left") {
                find_identifier_nodes_recursive(left_node, identifiers);
            }
            // Skip the right side (default value) as it's a usage, not a definition
        }
        "assignment_pattern" => {
            // For array destructuring with defaults like [a = defaultValue]
            // Only extract the variable name (left side), not the default value (right side)
            if let Some(left_node) = node.child_by_field_name("left") {
                find_identifier_nodes_recursive(left_node, identifiers);
            }
            // Skip the right side (default value) as it's a usage, not a definition
        }
        _ => {
            // Continue traversing for other node types
            let mut cursor = node.walk();
            for child in node.children(&mut cursor) {
                find_identifier_nodes_recursive(child, identifiers);
            }
        }
    }
}
