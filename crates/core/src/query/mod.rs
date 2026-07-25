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

/// What a captured name node declares.
///
/// Both languages need the kind, and TypeScript needs hoisting as well — a class is visible before
/// its declaration while a method is not — so it travels with the role rather than being assumed.
#[derive(Debug, Clone, PartialEq)]
pub struct DeclaredAs {
    pub definition_type: crate::models::DefinitionType,
    pub is_hoisted: bool,
}

impl DeclaredAs {
    pub const fn plain(definition_type: crate::models::DefinitionType) -> Self {
        Self {
            definition_type,
            is_hoisted: false,
        }
    }

    pub const fn hoisted(definition_type: crate::models::DefinitionType) -> Self {
        Self {
            definition_type,
            is_hoisted: true,
        }
    }
}

/// Run a scope query and give the kind of scope each captured node introduces.
///
/// The captured node is the whole item, since a scope spans it.
pub fn scope_kinds(
    query_source: &str,
    source_code: &str,
    root_node: Node,
    mapping: &[(&str, crate::models::ScopeType)],
) -> Result<Roles<crate::models::ScopeType>, String> {
    capture_roles(query_source, source_code, root_node, mapping)
}

/// The position of the second capture paired with the text of the first.
///
/// Positions are how a captured node is matched back to a `Definition`, which carries a position
/// rather than a node.
pub fn text_by_position(
    query_source: &str,
    source_code: &str,
    root_node: Node,
    text_capture: &str,
    position_capture: &str,
) -> Result<HashMap<(usize, usize), String>, String> {
    let pairs = map_pairs(
        query_source,
        source_code,
        root_node,
        text_capture,
        position_capture,
        |named, located| {
            let position = (
                located.start_position().row + 1,
                located.start_position().column + 1,
            );
            Some((position, text(source_code, named)?))
        },
    )?;

    Ok(pairs.into_iter().collect())
}

fn text(source_code: &str, node: Node) -> Option<String> {
    node.utf8_text(source_code.as_bytes())
        .ok()
        .map(str::to_string)
}

/// Map the two named captures of every match with a caller-supplied closure.
///
/// The mapping happens inside the match loop because a captured node's lifetime is tied to the
/// query cursor, so a language needing the nodes themselves — to read a type annotation's shape,
/// say — reaches them here rather than through a returned node.
pub fn map_pairs<T>(
    query_source: &str,
    source_code: &str,
    root_node: Node,
    first: &str,
    second: &str,
    mut map: impl FnMut(Node, Node) -> Option<T>,
) -> Result<Vec<T>, String> {
    let language = &*root_node.language();
    let query = Query::new(language, query_source)
        .map_err(|error| format!("Failed to create query: {error}"))?;

    let first_index = query
        .capture_index_for_name(first)
        .ok_or_else(|| format!("Query is missing the @{first} capture"))?;
    let second_index = query
        .capture_index_for_name(second)
        .ok_or_else(|| format!("Query is missing the @{second} capture"))?;

    let mut cursor = QueryCursor::new();
    let mut matches = cursor.matches(&query, root_node, source_code.as_bytes());
    let mut mapped = Vec::new();

    while let Some(query_match) = matches.next() {
        let of = |index: u32| {
            query_match
                .captures
                .iter()
                .find(|capture| capture.index == index)
                .map(|capture| capture.node)
        };

        if let (Some(a), Some(b)) = (of(first_index), of(second_index)) {
            mapped.extend(map(a, b));
        }
    }

    Ok(mapped)
}

/// The first and last line of a captured node.
pub type LineSpan = (usize, usize);

/// A captured name together with the lines another capture spans.
pub type NamedSpan = (String, LineSpan);

/// The text of one capture paired with the line span of another.
///
/// Spans answer containment questions — which class body a line sits inside — that a single
/// position cannot.
pub fn text_by_span(
    query_source: &str,
    source_code: &str,
    root_node: Node,
    text_capture: &str,
    span_capture: &str,
) -> Result<Vec<NamedSpan>, String> {
    map_pairs(
        query_source,
        source_code,
        root_node,
        text_capture,
        span_capture,
        |named, spanned| {
            let span = (
                spanned.start_position().row + 1,
                spanned.end_position().row + 1,
            );
            Some((text(source_code, named)?, span))
        },
    )
}
