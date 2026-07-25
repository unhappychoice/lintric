use crate::query;
use std::collections::HashSet;
use tree_sitter::Node;

/// Identifiers that bind a name rather than reference one.
const QUERY: &str = include_str!("../../../queries/typescript/bindings.scm");

/// What the query says about each identifier it captures.
///
/// One file because the extractor asks one question of it — whether this identifier is a reference
/// worth recording — and the reasons it may not be sit side by side.
pub struct Roles {
    /// Identifiers that declare a name.
    bindings: HashSet<usize>,
    /// Callees already recorded through the call expression itself.
    call_targets: HashSet<usize>,
    /// Object literal and pattern keys, which reference no declared member.
    shape_keys: HashSet<usize>,
}

pub fn roles(source_code: &str, root_node: Node) -> Result<Roles, String> {
    Ok(Roles {
        bindings: query::captured_nodes(QUERY, source_code, root_node, "binding")?,
        call_targets: query::captured_nodes(QUERY, source_code, root_node, "call_target")?,
        shape_keys: query::captured_nodes(QUERY, source_code, root_node, "shape_key")?,
    })
}

impl Roles {
    /// Whether this identifier declares a name.
    pub fn declares(&self, node: Node) -> bool {
        self.bindings.contains(&node.id())
    }

    /// Whether this identifier is a reference this extractor should record.
    ///
    /// A declaration is not, a callee is already recorded through its call expression, and a shape
    /// key names no declared member.
    pub fn reads(&self, node: Node) -> bool {
        !self.declares(node)
            && !self.call_targets.contains(&node.id())
            && !self.shape_keys.contains(&node.id())
    }
}
