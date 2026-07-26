//! What role a name plays in a `::` path.
//!
//! Being inside a path at all is a fact about the tree, carried by the usage's context. Which segment
//! it is comes from position, and then only from adjacency across the `::`, so two names that merely
//! share a line are never mistaken for one path.

use super::rust_dependency_resolver::RustDependencyResolver;
use crate::models::{Definition, ScopeType, Usage};

impl RustDependencyResolver {
    /// Check if this usage should be skipped because it has no definition
    /// in the qualifier's scope
    pub(super) fn is_method_in_scoped_identifier_without_definition(
        &self,
        usage_node: &Usage,
        definitions: &[Definition],
        all_usage_nodes: &[Usage],
    ) -> bool {
        // Only apply to scoped identifiers
        if usage_node.context.as_ref() != Some(&"scoped_identifier".to_string()) {
            return false;
        }

        // Find the qualifier (type part) of this scoped identifier
        let qualifier = all_usage_nodes
            .iter()
            .filter(|u| {
                u.position.start_line == usage_node.position.start_line
                    && u.position.end_column < usage_node.position.start_column
                    && u.context.as_ref() == Some(&"scoped_identifier".to_string())
                    && matches!(u.kind, crate::models::UsageKind::Identifier)
            })
            .max_by_key(|u| u.position.start_column);

        if let Some(qualifier) = qualifier {
            // Find the qualifier's definition in symbol_table
            let qualifier_scope_id = self
                .symbol_table
                .scopes
                .find_scope_at_position(&qualifier.position)
                .unwrap_or(0);

            let mut current_scope_id = qualifier_scope_id;
            while let Some(scope) = self.symbol_table.scopes.get_scope(current_scope_id) {
                if let Some(qualifier_definitions) = scope.symbols.get(&qualifier.name) {
                    // Look for the method in definitions that are related to this qualifier
                    let has_method_definition = definitions.iter().any(|def| {
                        def.name == usage_node.name
                            && qualifier_definitions.iter().any(|qual_def| {
                                // Check if this method definition is related to the qualifier's scope
                                match qual_def.definition_type {
                                    crate::models::DefinitionType::StructDefinition
                                    | crate::models::DefinitionType::EnumDefinition
                                    | crate::models::DefinitionType::TypeDefinition => {
                                        // For local types, check if method is in nearby lines (impl block)
                                        (def.position.start_line as i32
                                            - qual_def.position.start_line as i32)
                                            .abs()
                                            < 20
                                    }
                                    _ => false, // For imports, no local method definitions
                                }
                            })
                    });

                    return !has_method_definition;
                }
                if let Some(parent_id) = scope.parent {
                    current_scope_id = parent_id;
                } else {
                    break;
                }
            }
        }

        // If we can't find qualifier or determine scope, don't skip
        false
    }

    /// Check if this usage is likely a method name in a qualified call (Type::method)
    pub(super) fn is_method_name_in_qualified_call(
        &self,
        usage_node: &Usage,
        all_usage_nodes: &[Usage],
    ) -> bool {
        // Must be in scoped_identifier context
        if usage_node.context.as_ref() != Some(&"scoped_identifier".to_string()) {
            return false;
        }

        // Must be an identifier, not a type identifier
        if !matches!(usage_node.kind, crate::models::UsageKind::Identifier) {
            return false;
        }

        // The qualifier must be the segment immediately before this one. Accepting anything
        // earlier on the line confused two separate paths, so `crate::V + crate::W` treated `V`
        // as the qualifier of `W`.
        all_usage_nodes.iter().any(|other| {
            is_in_path(other)
                && is_adjacent_segment(other, usage_node)
                && matches!(
                    other.kind,
                    crate::models::UsageKind::Identifier | crate::models::UsageKind::TypeIdentifier
                )
        })
    }

    /// Check if this usage is a type reference in a scoped identifier context
    pub(super) fn is_type_reference_in_scoped_identifier(&self, usage_node: &Usage) -> bool {
        // If it's a TypeIdentifier, it's definitely a type reference
        matches!(usage_node.kind, crate::models::UsageKind::TypeIdentifier)
    }

    /// Check if the usage represents a closure capturing a variable from an outer scope
    #[allow(dead_code)]
    pub(super) fn is_closure_capture(&self, _usage_node: &Usage, _def: &Definition) -> bool {
        // Find the closest enclosing function-like scope for the usage
        let usage_scope_id = self
            .symbol_table
            .scopes
            .find_scope_at_position(&_usage_node.position);

        if let Some(scope_id) = usage_scope_id {
            // Walk up the scope chain to find if we're inside a closure
            let mut current_scope_id = scope_id;
            while let Some(scope) = self.symbol_table.scopes.get_scope(current_scope_id) {
                if matches!(scope.scope_type, ScopeType::Closure) {
                    // We're inside a closure, so cross-function capture is allowed
                    return true;
                }

                if let Some(parent_id) = scope.parent {
                    current_scope_id = parent_id;
                } else {
                    break;
                }
            }
        }

        // Not inside a closure, so cross-function capture is not allowed
        false
    }

    /// Check if a TypeIdentifier is part of a qualified path (like "future" in "std::future::Future")
    /// Whether this name qualifies a later segment of a path rather than being what the path names.
    ///
    /// `Foo` in `std::Foo::Bar` qualifies `Bar`, so resolving it against a local definition of the
    /// same name would be wrong. `Foo` in `Foo::Bar` is not a qualifier of that kind — it is the
    /// type the associated item belongs to, and does resolve.
    ///
    /// Being inside a path at all is a fact about the tree, carried by the usage's context. Only
    /// which segment it is comes from position, and then only from adjacency across the `::`, so
    /// two names that merely share a line are never mistaken for one path.
    pub(super) fn is_part_of_qualified_path(
        &self,
        usage_node: &Usage,
        all_usage_nodes: &[Usage],
    ) -> bool {
        if !is_in_path(usage_node) {
            return false;
        }

        let has_preceding = all_usage_nodes
            .iter()
            .any(|other| is_adjacent_segment(other, usage_node));
        let has_following = all_usage_nodes
            .iter()
            .any(|other| is_adjacent_segment(usage_node, other));

        has_preceding && has_following
    }

    /// Whether this usage is a segment that something else is reached through.
    ///
    /// A path head names a module or a type — never a function or a local — so `mod T` beside
    /// `fn T` resolves `T::V` to the module. A later segment can be anything, which is why the
    /// question is about having a follower rather than about being in a path at all.
    pub(super) fn is_path_head(&self, usage_node: &Usage, all_usage_nodes: &[Usage]) -> bool {
        is_in_path(usage_node)
            && all_usage_nodes
                .iter()
                .any(|other| is_adjacent_segment(usage_node, other))
    }
}

/// Width of the `::` between path segments.
const PATH_SEPARATOR: usize = 2;

fn is_in_path(usage: &Usage) -> bool {
    matches!(
        usage.context.as_deref(),
        Some("scoped_identifier") | Some("scoped_type_identifier")
    )
}

/// Whether `earlier` is the segment immediately before `later` in one path.
fn is_adjacent_segment(earlier: &Usage, later: &Usage) -> bool {
    earlier.position.start_line == later.position.start_line
        && earlier.position.end_column + PATH_SEPARATOR == later.position.start_column
}
