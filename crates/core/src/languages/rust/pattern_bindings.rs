//! What a Rust pattern binds.
//!
//! A pattern names both things it declares and things it reads: `let S::F(v) = s` binds `v` while
//! naming the enum and its variant, and a match guard reads the names the pattern binds without
//! binding any itself. Telling those apart is the whole of this module.

use tree_sitter::Node;

use super::definition_extractor::RustDefinitionExtractor;
use crate::models::{Definition, DefinitionType, Position, ScopeId};

impl RustDefinitionExtractor {
    pub(super) fn extract_closure_parameter_definition(
        &self,
        node: Node,
        scope: ScopeId,
        source: &str,
    ) -> Option<Definition> {
        let name_text = node.utf8_text(source.as_bytes()).ok()?;

        Some(Definition {
            name: name_text.to_string(),
            definition_type: DefinitionType::VariableDefinition,
            position: Position::from_node(&node),
            scope_id: Some(scope),
            accessibility: None, // Will be set by ASTScopeTraverser to ScopeLocal
            is_hoisted: Some(false),
        })
    }

    #[allow(dead_code)]
    pub(super) fn extract_closure_definitions(
        &self,
        node: Node,
        _scope: ScopeId,
        source: &str,
    ) -> Vec<Definition> {
        let mut definitions = vec![];
        if let Some(params_node) = node.child_by_field_name("parameters") {
            definitions.extend(self.extract_closure_parameters(params_node, source));
        }
        definitions
    }

    #[allow(dead_code)]
    fn extract_closure_parameters(&self, node: Node, source: &str) -> Vec<Definition> {
        let mut definitions = vec![];
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "identifier" {
                definitions.push(Definition::new(
                    &child,
                    source,
                    DefinitionType::VariableDefinition,
                ));
            }
        }
        definitions
    }

    pub(super) fn is_pattern_binding(&self, node: Node) -> bool {
        // Check if this identifier is inside a scoped_identifier (like Vec::<T>::new)
        // If so, it's a usage, not a pattern binding
        let mut current = node.parent();
        while let Some(parent) = current {
            if parent.kind() == "scoped_identifier" {
                return false;
            }
            current = parent.parent();
        }

        let mut current = node;

        // Traverse up to find pattern contexts
        while let Some(parent) = current.parent() {
            match parent.kind() {
                "for_expression" => {
                    if let Some(pattern_field) = parent.child_by_field_name("pattern") {
                        if self.is_child_of(node, pattern_field) {
                            return true;
                        }
                    }
                }
                // `if let` and `while let`, chained or not: a chain holds several `let_condition`s,
                // so asking about the condition itself misses every pattern in one.
                "let_condition" => {
                    if let Some(pattern_field) = parent.child_by_field_name("pattern") {
                        if self.is_child_of(node, pattern_field) {
                            return true;
                        }
                    }
                }
                "match_arm" => {
                    if let Some(pattern_field) = parent.child_by_field_name("pattern") {
                        if self.is_child_of(node, pattern_field) {
                            return !self.is_in_match_guard(node);
                        }
                    }
                }
                _ => {}
            }
            current = parent;
        }
        false
    }

    /// Whether this identifier sits in a match arm's guard.
    ///
    /// A guard is the `condition:` field of the `match_pattern`, so it is a descendant of the arm's
    /// `pattern:` field — but it reads the names the pattern binds rather than binding any itself.
    /// `n if flag` makes the guard the identifier itself rather than an expression containing one,
    /// which is why the condition is compared as well as searched.
    fn is_in_match_guard(&self, node: Node) -> bool {
        std::iter::successors(node.parent(), |parent| parent.parent())
            .take_while(|parent| parent.kind() != "match_arm")
            .filter(|parent| parent.kind() == "match_pattern")
            .filter_map(|pattern| pattern.child_by_field_name("condition"))
            .any(|condition| condition.id() == node.id() || self.is_child_of(node, condition))
    }

    /// The identifiers a pattern binds.
    ///
    /// A pattern's `type:` field names the struct or variant being matched, so it is a reference
    /// rather than a binding and is skipped — otherwise `let S::F(v) = s` would register the enum
    /// and its variant as locals. A shorthand field pattern binds under the field's own name, which
    /// is a `shorthand_field_identifier` rather than an `identifier`.
    #[allow(clippy::only_used_in_recursion)]
    pub(super) fn find_pattern_bindings<'a>(&self, pattern: Node<'a>) -> Vec<Node<'a>> {
        if matches!(pattern.kind(), "identifier" | "shorthand_field_identifier") {
            return vec![pattern];
        }

        let matched_type = pattern.child_by_field_name("type").map(|node| node.id());
        let mut cursor = pattern.walk();
        let children: Vec<Node<'a>> = pattern.children(&mut cursor).collect();

        children
            .into_iter()
            .filter(|child| Some(child.id()) != matched_type)
            .flat_map(|child| self.find_pattern_bindings(child))
            .collect()
    }

    #[allow(clippy::only_used_in_recursion, dead_code)]
    fn find_identifier_nodes_in_node<'a>(&self, node: Node<'a>) -> Vec<Node<'a>> {
        let mut identifiers = vec![];
        if node.kind() == "identifier" {
            identifiers.push(node);
        } else {
            let mut cursor = node.walk();
            for child in node.children(&mut cursor) {
                identifiers.extend(self.find_identifier_nodes_in_node(child));
            }
        }
        identifiers
    }

    #[allow(clippy::only_used_in_recursion)]
    fn is_child_of(&self, child: Node, parent: Node) -> bool {
        let mut cursor = parent.walk();
        for descendant in parent.children(&mut cursor) {
            if descendant.id() == child.id() {
                return true;
            }
            if self.is_child_of(child, descendant) {
                return true;
            }
        }
        false
    }

    pub(super) fn is_for_loop_pattern(&self, node: Node) -> bool {
        if let Some(parent) = node.parent() {
            if parent.kind() == "for_expression" {
                if let Some(pattern_field) = parent.child_by_field_name("pattern") {
                    return node.id() == pattern_field.id();
                }
            }
        }
        false
    }

    pub(super) fn extract_pattern_binding_definition(
        &self,
        node: Node,
        scope: ScopeId,
        source: &str,
    ) -> Option<Definition> {
        let name_text = node.utf8_text(source.as_bytes()).ok()?;

        // Skip constructors and enum variants (start with uppercase)
        if name_text.chars().next().is_some_and(|c| c.is_uppercase()) {
            return None;
        }

        Some(Definition {
            name: name_text.to_string(),
            definition_type: DefinitionType::VariableDefinition,
            position: Position::from_node(&node),
            scope_id: Some(scope),
            accessibility: None, // Will be set by ASTScopeTraverser to ScopeLocal
            is_hoisted: Some(false),
        })
    }
}
