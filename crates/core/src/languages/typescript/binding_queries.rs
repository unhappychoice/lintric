use crate::query;
use std::collections::HashSet;
use tree_sitter::Node;

/// Identifiers that bind a name rather than reference one.
const QUERY: &str = include_str!("../../../queries/typescript/bindings.scm");

/// The identifiers the query calls bindings, and the callees it says are already counted.
///
/// Both come from one file because the extractor asks one question of it — whether this identifier
/// is a reference worth recording — and the two reasons it may not be sit side by side.
pub fn bindings_and_call_targets(
    source_code: &str,
    root_node: Node,
) -> Result<(HashSet<usize>, HashSet<usize>), String> {
    Ok((
        query::captured_nodes(QUERY, source_code, root_node, "binding")?,
        query::captured_nodes(QUERY, source_code, root_node, "call_target")?,
    ))
}
