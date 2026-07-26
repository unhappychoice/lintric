//! Locating what a Rust file reads.
//!
//! Which identifiers declare rather than read comes from `queries/rust/bindings.scm`; the arms here
//! handle what a query cannot state — a `Self` standing for the type it is written inside, and the
//! captures inside a format string, which have no nodes at all.

use tree_sitter::Node;

use super::format_string;
use crate::models::{ast_traverser::NodeUsageExtractor, Position, ScopeId, Usage, UsageKind};

/// Rust-specific usage extractor
pub struct RustUsageExtractor {
    roles: super::binding_queries::Roles,
}

impl RustUsageExtractor {
    /// Fails if the binding query does not compile, which is a bug in the `.scm` file rather than
    /// anything about the source being analyzed.
    pub fn new(source_code: &str, root_node: Node) -> Result<Self, String> {
        Ok(Self {
            roles: super::binding_queries::roles(source_code, root_node)?,
        })
    }
}

impl NodeUsageExtractor for RustUsageExtractor {
    fn extract_usage(&self, node: Node, scope: ScopeId, source: &str) -> Vec<Usage> {
        let kind = match node.kind() {
            "identifier" | "type_identifier" if self.is_self_type(node, source) => {
                // `Self` names a type without spelling it, so resolution needs the name it stands
                // for rather than the keyword
                return self
                    .extract_self_type_usage(node, scope, source)
                    .into_iter()
                    .collect();
            }
            "identifier" => {
                // Only treat identifier as usage if it's not in a definition context
                // and not the function name part of a call_expression (to avoid duplication)
                if self.is_identifier_in_definition_context(node)
                    || self.is_function_name_in_call_expression(node)
                {
                    None
                } else if self.is_identifier_in_type_context(node) {
                    // Check if this identifier is in a type context (like use statements)
                    Some(UsageKind::TypeIdentifier)
                } else {
                    Some(UsageKind::Identifier)
                }
            }
            "type_identifier" => {
                // Only treat type_identifier as usage if it's not in a definition context
                // This is for dependency resolution to work correctly
                if self.is_identifier_in_definition_context(node) {
                    None
                } else {
                    Some(UsageKind::TypeIdentifier)
                }
            }
            "call_expression" => {
                // Use special handling for call expressions to extract function name
                return self
                    .extract_call_usage(node, scope, source)
                    .into_iter()
                    .collect();
            }
            "field_expression" => {
                // Use special handling for field expressions to extract field name
                return self
                    .extract_field_usage(node, scope, source)
                    .into_iter()
                    .collect();
            }
            "field_initializer" => {
                // `Point { x: value }` references the declaration of `x`, not just the value
                return self
                    .extract_field_initializer_usage(node, scope, source)
                    .into_iter()
                    .collect();
            }
            "field_identifier" | "shorthand_field_identifier" => {
                // A field named by a pattern references the field's declaration, the same way one
                // named by a struct literal does
                return self
                    .extract_pattern_field_usage(node, scope, source)
                    .into_iter()
                    .collect();
            }
            "shorthand_field_initializer" => {
                // `Point { x }` references the declaration of `x` as well as reading the binding,
                // which the inner identifier already covers
                return self
                    .extract_shorthand_field_initializer_usage(node, scope, source)
                    .into_iter()
                    .collect();
            }
            "metavariable" => Some(UsageKind::Metavariable),
            "string_content" => {
                // Inline format string captures have no nodes of their own, so they are parsed
                // out of the literal rather than reached by traversal
                return format_string::capture_usages(node, scope, source);
            }
            _ => None,
        };

        kind.and_then(|k| {
            let name_text = node.utf8_text(source.as_bytes()).ok()?;
            Some(Usage {
                name: Usage::normalize_line_endings(name_text),
                kind: k,
                position: Position::from_node(&node),
                context: self.get_node_context(&node),
                scope_id: Some(scope),
            })
        })
        .into_iter()
        .collect()
    }
}

impl RustUsageExtractor {
    /// Whether this identifier declares the name rather than reading it.
    ///
    /// The patterns live in `queries/rust/bindings.scm`; a reference capture wins, which is how the
    /// type a pattern matches against stays a usage while the names it introduces do not.
    fn is_identifier_in_definition_context(&self, node: Node) -> bool {
        self.roles.bindings.contains(&node.id()) && !self.roles.references.contains(&node.id())
    }

    /// Whether the call expression around this identifier already records it.
    fn is_function_name_in_call_expression(&self, node: Node) -> bool {
        self.roles.call_targets.contains(&node.id())
    }

    /// Whether this identifier sits anywhere inside a `use` tree.
    ///
    /// Stays here rather than moving to the query file: an ancestor at any depth is not a shape a
    /// query states, and enumerating the depths a path can nest to would be worse than the walk.
    fn is_identifier_in_type_context(&self, node: Node) -> bool {
        let mut current = node.parent();
        while let Some(parent) = current {
            match parent.kind() {
                "use_declaration" | "use_as_clause" | "scoped_use_list" | "use_list" => {
                    return true;
                }
                "scoped_identifier" => {
                    // Check if this scoped_identifier is in a type context
                    current = parent.parent();
                    continue;
                }
                _ => current = parent.parent(),
            }
        }
        false
    }

    fn get_node_context(&self, node: &Node) -> Option<String> {
        // Use the same logic as Usage::get_node_context from the original
        let mut current = node.parent();
        while let Some(parent) = current {
            match parent.kind() {
                "scoped_identifier" => return Some("scoped_identifier".to_string()),
                // A path in type position is a different node, but it is still a path
                "scoped_type_identifier" => return Some("scoped_type_identifier".to_string()),
                "field_expression" => return Some("field_expression".to_string()),
                "call_expression" => return Some("call_expression".to_string()),
                _ => current = parent.parent(),
            }
        }
        None
    }

    #[allow(dead_code)]
    fn extract_identifier_usage(&self, node: Node, scope: ScopeId, source: &str) -> Option<Usage> {
        let name_text = node.utf8_text(source.as_bytes()).ok()?;

        Some(Usage {
            name: name_text.to_string(),
            kind: UsageKind::Identifier,
            position: Position::from_node(&node),
            context: None,
            scope_id: Some(scope),
        })
    }

    fn extract_call_usage(&self, node: Node, scope: ScopeId, source: &str) -> Option<Usage> {
        let function_node = node.child(0)?;
        if function_node.kind() != "identifier" {
            return None;
        }
        let function_name =
            Usage::normalize_line_endings(function_node.utf8_text(source.as_bytes()).ok()?);

        Some(Usage {
            name: function_name,
            kind: UsageKind::CallExpression,
            position: Position::from_node(&node),
            context: Some("call_expression".to_string()),
            scope_id: Some(scope),
        })
    }

    /// The field named by `Point { x: value }`.
    fn extract_field_initializer_usage(
        &self,
        node: Node,
        scope: ScopeId,
        source: &str,
    ) -> Option<Usage> {
        self.field_reference(node.child_by_field_name("field")?, scope, source)
    }

    /// The field named by a pattern, as in `let Point { x, y: renamed } = p`.
    ///
    /// Restricted to `field_pattern`, since the same node kinds appear in field declarations and
    /// field expressions, which are handled elsewhere.
    fn extract_pattern_field_usage(
        &self,
        node: Node,
        scope: ScopeId,
        source: &str,
    ) -> Option<Usage> {
        node.parent()
            .filter(|parent| parent.kind() == "field_pattern")
            .and_then(|_| self.field_reference(node, scope, source))
    }

    /// The field named by the shorthand `Point { x }`, whose identifier also reads a binding.
    fn extract_shorthand_field_initializer_usage(
        &self,
        node: Node,
        scope: ScopeId,
        source: &str,
    ) -> Option<Usage> {
        self.field_reference(node.child(0)?, scope, source)
    }

    fn field_reference(&self, node: Node, scope: ScopeId, source: &str) -> Option<Usage> {
        let name = node.utf8_text(source.as_bytes()).ok()?;

        Some(Usage {
            name: Usage::normalize_line_endings(name),
            kind: UsageKind::FieldInitializer,
            position: Position::from_node(&node),
            context: Some("field_initializer".to_string()),
            scope_id: Some(scope),
        })
    }

    fn extract_field_usage(&self, node: Node, scope: ScopeId, source: &str) -> Option<Usage> {
        // Use the same logic as Usage::new_field_expression from the original
        let field_name = if let Some(field_node) = node.child_by_field_name("field") {
            field_node
                .utf8_text(source.as_bytes())
                .unwrap_or("")
                .trim()
                .replace("\r\n", "\n")
        } else if let Some(last_child) = node.child(node.child_count().saturating_sub(1)) {
            // Fallback: try the last child
            last_child
                .utf8_text(source.as_bytes())
                .unwrap_or("")
                .trim()
                .replace("\r\n", "\n")
        } else {
            // Final fallback to full text
            node.utf8_text(source.as_bytes())
                .unwrap_or("")
                .trim()
                .replace("\r\n", "\n")
        };

        Some(Usage {
            name: field_name,
            kind: UsageKind::FieldExpression,
            position: Position::from_node(&node),
            context: Some("field_expression".to_string()),
            scope_id: Some(scope),
        })
    }
}
