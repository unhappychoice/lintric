//! Running tree-sitter queries and turning their captures into roles.
//!
//! A query file states which nodes matter and what each one is, using capture names. The language
//! supplies a table mapping those names to whatever it wants to know — a `DefinitionType`, say —
//! and this module returns the answer keyed by node, so an extractor can ask about the node it is
//! looking at without repeating the pattern in Rust.

use std::collections::HashMap;
use streaming_iterator::StreamingIterator;
use tree_sitter::{Node, Query, QueryCursor};

/// The role each captured node plays, keyed by node id.
pub type Roles<T> = HashMap<usize, T>;

/// Run a query and label every captured node with the role its capture name maps to.
///
/// Capture names absent from `mapping` are ignored, so a query may capture nodes for other
/// purposes without disturbing this one.
pub fn capture_roles<T: Clone>(
    query_source: &str,
    source_code: &str,
    root_node: Node,
    mapping: &[(&str, T)],
) -> Result<Roles<T>, String> {
    let language = &*root_node.language();
    let query = Query::new(language, query_source)
        .map_err(|error| format!("Failed to create query: {error}"))?;

    let roles_by_index = indexed_roles(&query, mapping);
    let mut cursor = QueryCursor::new();
    let mut matches = cursor.matches(&query, root_node, source_code.as_bytes());
    let mut roles = Roles::new();

    while let Some(query_match) = matches.next() {
        for capture in query_match.captures {
            if let Some(role) = roles_by_index.get(&capture.index) {
                roles.insert(capture.node.id(), role.clone());
            }
        }
    }

    Ok(roles)
}

/// Resolve the mapping's capture names to the indices the query assigned them, once, so the match
/// loop compares integers rather than strings.
fn indexed_roles<T: Clone>(query: &Query, mapping: &[(&str, T)]) -> HashMap<u32, T> {
    mapping
        .iter()
        .filter_map(|(name, role)| {
            query
                .capture_index_for_name(name)
                .map(|index| (index, role.clone()))
        })
        .collect()
}
