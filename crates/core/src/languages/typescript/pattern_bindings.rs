//! What a TypeScript pattern binds.
//!
//! Not every identifier under a pattern is a bound name: a default value and a computed key are read
//! rather than declared, and collecting them as bindings made each shadow the declaration it was
//! naming. A constructor parameter is the other way round — a modifier turns it into a property
//! declaration as well as a parameter.

use tree_sitter::Node;

use super::definition_extractor::TypeScriptDefinitionExtractor;

impl TypeScriptDefinitionExtractor {
    /// The names a destructuring pattern binds.
    #[allow(clippy::only_used_in_recursion)]
    pub(super) fn find_identifier_nodes_in_node<'a>(&self, node: Node<'a>) -> Vec<Node<'a>> {
        // A shorthand pattern name is itself the identifier, since `{ x }` has no separate node.
        if matches!(
            node.kind(),
            "identifier" | "shorthand_property_identifier_pattern"
        ) {
            return vec![node];
        }

        Self::binding_children(node)
            .iter()
            .flat_map(|child| self.find_identifier_nodes_in_node(*child))
            .collect()
    }

    /// The children of a pattern node that can hold a bound name.
    ///
    /// `x = fallback` binds only its left side, and `[k]: v` binds only the value — the key names
    /// something declared elsewhere.
    pub(super) fn binding_children<'a>(node: Node<'a>) -> Vec<Node<'a>> {
        match node.kind() {
            "assignment_pattern" | "object_assignment_pattern" => {
                node.child_by_field_name("left").into_iter().collect()
            }
            "computed_property_name" => vec![],
            _ => {
                let mut cursor = node.walk();
                node.children(&mut cursor).collect()
            }
        }
    }
}
