//! What an import or export statement introduces locally.
//!
//! The declarations these names refer to live in other files, which single-file analysis cannot see.
//! What a statement does here is introduce a local name for one of them.

use tree_sitter::Node;

use super::definition_extractor::TypeScriptDefinitionExtractor;
use crate::models::{Definition, DefinitionType, Position, ScopeId, Usage};

impl TypeScriptDefinitionExtractor {
    pub(super) fn extract_import_statement_definition(
        &self,
        _node: Node,
        _scope: ScopeId,
        _source: &str,
    ) -> Option<Definition> {
        None // import_statement itself doesn't create definitions
    }

    pub(super) fn extract_export_statement_definition(
        &self,
        _node: Node,
        _scope: ScopeId,
        _source: &str,
    ) -> Option<Definition> {
        None // export_statement itself doesn't create definitions
    }

    pub(super) fn extract_namespace_import_definition(
        &self,
        node: Node,
        scope: ScopeId,
        source: &str,
    ) -> Option<Definition> {
        // For namespace imports like: import * as Utils from './utils'
        if let Some(name_node) = node.child(2) {
            // The identifier after 'as'
            if name_node.kind() == "identifier" {
                let name_text = name_node.utf8_text(source.as_bytes()).ok()?;
                return Some(Definition {
                    name: Usage::normalize_line_endings(name_text),
                    definition_type: DefinitionType::ImportDefinition,
                    position: Position::from_node(&name_node),
                    scope_id: Some(scope),
                    accessibility: None,
                    is_hoisted: Some(false),
                });
            }
        }
        None
    }

    pub(super) fn extract_import_clause_definition(
        &self,
        node: Node,
        scope: ScopeId,
        source: &str,
    ) -> Option<Definition> {
        // Handle default imports - direct identifier in import_clause
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "identifier" {
                let name_text = child.utf8_text(source.as_bytes()).ok()?;
                return Some(Definition {
                    name: Usage::normalize_line_endings(name_text),
                    definition_type: DefinitionType::ImportDefinition,
                    position: Position::from_node(&child),
                    scope_id: Some(scope),
                    accessibility: None,
                    is_hoisted: Some(false),
                });
            }
        }
        None
    }
}
