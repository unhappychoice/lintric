//! Locating what a TypeScript file reads.
//!
//! Which identifiers declare rather than read comes from `queries/typescript/bindings.scm`; the arms
//! here handle the shapes a query cannot label on its own.

use tree_sitter::Node;

use crate::models::{ast_traverser::NodeUsageExtractor, Position, ScopeId, Usage, UsageKind};

/// TypeScript-specific usage extractor
pub struct TypeScriptUsageExtractor {
    roles: super::binding_queries::Roles,
}

impl TypeScriptUsageExtractor {
    /// Fails if the binding query does not compile, which is a bug in the `.scm` file rather than
    /// anything about the source being analyzed.
    pub fn new(source_code: &str, root_node: Node) -> Result<Self, String> {
        Ok(Self {
            roles: super::binding_queries::roles(source_code, root_node)?,
        })
    }
}

impl NodeUsageExtractor for TypeScriptUsageExtractor {
    fn extract_usage(&self, node: Node, scope: ScopeId, source: &str) -> Vec<Usage> {
        let usage = match node.kind() {
            "identifier" => {
                if self.is_usage_context(node) {
                    self.extract_identifier_usage(node, scope, source)
                } else {
                    None
                }
            }
            "call_expression" => self.extract_call_usage(node, scope, source),
            "type_identifier" => {
                if self.is_type_identifier_in_definition_context(node) {
                    None
                } else {
                    self.extract_type_usage(node, scope, source)
                }
            }
            // A `#name` member is the same thing under a different node kind, and the definition
            // positions it can occupy are already covered
            "property_identifier" | "private_property_identifier" => {
                self.extract_property_identifier_usage(node, scope, source)
            }
            // `{ x }` is shorthand for `{ x: x }`, so it reads the binding. The pattern form is a
            // different node kind and stays a binding.
            "shorthand_property_identifier" => self.extract_identifier_usage(node, scope, source),
            _ => None,
        };

        usage.into_iter().collect()
    }
}

impl TypeScriptUsageExtractor {
    /// Whether this identifier reads a name, as opposed to declaring one or naming the callee of a
    /// call expression that is already recorded as the usage.
    ///
    /// The patterns live in `queries/typescript/bindings.scm`.
    fn is_usage_context(&self, node: Node) -> bool {
        self.roles.reads(node)
    }

    fn extract_identifier_usage(&self, node: Node, scope: ScopeId, source: &str) -> Option<Usage> {
        let name_text = node.utf8_text(source.as_bytes()).ok()?;

        // Determine context based on ancestor call_expression
        let context = self.find_call_expression_context(node);

        Some(Usage {
            name: Usage::normalize_line_endings(name_text),
            kind: UsageKind::Identifier,
            position: Position::from_node(&node),
            context,
            scope_id: Some(scope),
        })
    }

    fn find_call_expression_context(&self, node: Node) -> Option<String> {
        let mut current = node.parent();
        while let Some(parent) = current {
            if parent.kind() == "call_expression" {
                return Some("call_expression".to_string());
            }
            current = parent.parent();
        }
        None
    }

    fn extract_call_usage(&self, node: Node, scope: ScopeId, source: &str) -> Option<Usage> {
        // Extract function name from call_expression by getting the function field
        let function_node = node.child_by_field_name("function")?;
        if function_node.kind() != "identifier" {
            return None;
        }
        let function_name =
            Usage::normalize_line_endings(function_node.utf8_text(source.as_bytes()).ok()?);

        Some(Usage {
            name: function_name,
            kind: UsageKind::CallExpression,
            position: Position::from_node(&node), // Use the full call_expression range like old implementation
            context: Some("call_expression".to_string()), // Restore old implementation context
            scope_id: Some(scope),
        })
    }

    fn extract_type_usage(&self, node: Node, scope: ScopeId, source: &str) -> Option<Usage> {
        let name_text = node.utf8_text(source.as_bytes()).ok()?;

        Some(Usage {
            name: Usage::normalize_line_endings(name_text),
            kind: UsageKind::TypeIdentifier,
            position: Position::from_node(&node),
            context: None, // Match old implementation
            scope_id: Some(scope),
        })
    }

    /// A member read through a receiver, unless this occurrence declares the member instead.
    ///
    /// Which occurrences declare rather than read is stated in `queries/typescript/bindings.scm`.
    fn extract_property_identifier_usage(
        &self,
        node: Node,
        scope: ScopeId,
        source: &str,
    ) -> Option<Usage> {
        if !self.roles.reads(node) {
            return None;
        }

        // Property identifiers (like the "x" in "obj.x") should be treated as field expressions
        let name_text = node.utf8_text(source.as_bytes()).ok()?;

        Some(Usage {
            name: Usage::normalize_line_endings(name_text),
            kind: UsageKind::FieldExpression,
            position: Position::from_node(&node),
            context: None, // Match old implementation
            scope_id: Some(scope),
        })
    }

    /// Whether this type identifier is the name a declaration introduces.
    fn is_type_identifier_in_definition_context(&self, node: Node) -> bool {
        self.roles.declares(node)
    }
}
