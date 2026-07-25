use crate::query;
use std::collections::HashSet;
use tree_sitter::Node;

/// Identifiers that bind a name rather than reference one.
const QUERY: &str = include_str!("../../../queries/rust/bindings.scm");

/// What the query says about each identifier it captures.
///
/// One file because the extractor asks one question of it — whether this identifier is a reference
/// worth recording — and the reasons it may not be sit side by side.
pub struct Roles {
    /// Identifiers that declare a name.
    pub bindings: HashSet<usize>,
    /// Identifiers a binding pattern names but that read rather than declare: the type the pattern
    /// matches against, which is a direct child just as the names it introduces are. A query cannot
    /// say "every child but this one", so this overrides `bindings`.
    pub references: HashSet<usize>,
    /// Callees already recorded through the call expression itself.
    pub call_targets: HashSet<usize>,
}

pub fn roles(source_code: &str, root_node: Node) -> Result<Roles, String> {
    Ok(Roles {
        bindings: query::captured_nodes(QUERY, source_code, root_node, "binding")?,
        references: query::captured_nodes(QUERY, source_code, root_node, "reference")?,
        call_targets: query::captured_nodes(QUERY, source_code, root_node, "call_target")?,
    })
}
