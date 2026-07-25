use crate::query;
use std::collections::HashSet;
use tree_sitter::Node;

/// Identifiers that bind a name rather than reference one.
const QUERY: &str = include_str!("../../../queries/rust/bindings.scm");

/// The identifiers the query calls bindings, and the ones it calls references.
///
/// Both come from one file because they answer one question. A reference wins: the type a pattern
/// matches against is a direct child of that pattern just as the names it introduces are, so the
/// only thing telling them apart is the field, and a query cannot say "every child but this one".
pub fn bindings_and_references(
    source_code: &str,
    root_node: Node,
) -> Result<(HashSet<usize>, HashSet<usize>), String> {
    Ok((
        query::captured_nodes(QUERY, source_code, root_node, "binding")?,
        query::captured_nodes(QUERY, source_code, root_node, "reference")?,
    ))
}
