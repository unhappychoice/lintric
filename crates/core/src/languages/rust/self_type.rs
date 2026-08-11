//! `Self`, resolved to the type it stands for.
//!
//! `Self` names a type without spelling it, so resolution needs the name it stands for rather than
//! the keyword. Rewriting the usage to that name is what lets ordinary resolution reach the type's
//! own declaration; the position stays on the keyword and the context records the rewrite, so
//! `debug ir` still shows where the name came from.

use tree_sitter::Node;

use super::usage_extractor::RustUsageExtractor;
use crate::models::{Position, ScopeId, Usage, UsageKind};

impl RustUsageExtractor {
    pub(super) fn is_self_type(&self, node: Node, source: &str) -> bool {
        node.utf8_text(source.as_bytes()) == Ok("Self")
    }

    /// `Self`, resolved to the type it stands for.
    ///
    /// Rewriting the usage to that name is what lets ordinary resolution reach the type's own
    /// declaration. The position stays on the keyword and the context records the rewrite, so
    /// `debug ir` still shows where the name came from.
    pub(super) fn extract_self_type_usage(
        &self,
        node: Node,
        scope: ScopeId,
        source: &str,
    ) -> Option<Usage> {
        let name = self.enclosing_self_type(node, source)?;

        Some(Usage {
            name,
            kind: UsageKind::TypeIdentifier,
            position: Position::from_node(&node),
            context: Some("self_type".to_string()),
            scope_id: Some(scope),
        })
    }

    /// The name `Self` stands for: the type of the nearest enclosing `impl`, or the trait itself
    /// inside a trait declaration.
    fn enclosing_self_type(&self, node: Node, source: &str) -> Option<String> {
        let mut current = node.parent();

        while let Some(parent) = current {
            let named = match parent.kind() {
                "impl_item" => parent.child_by_field_name("type"),
                "trait_item" => parent.child_by_field_name("name"),
                _ => None,
            };

            if let Some(named) = named {
                return self.type_name_text(named, source);
            }

            current = parent.parent();
        }

        None
    }

    /// The bare name of a type, looking through generic arguments so that `Container<T>` yields
    /// `Container`.
    fn type_name_text(&self, node: Node, source: &str) -> Option<String> {
        let named = match node.kind() {
            "generic_type" => node.child_by_field_name("type")?,
            _ => node,
        };

        named.utf8_text(source.as_bytes()).ok().map(str::to_string)
    }
}
