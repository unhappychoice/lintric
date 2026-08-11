use crate::models::ScopeType;
use crate::query::{self, Roles};
use tree_sitter::Node;

/// Nodes that introduce a scope.
const QUERY: &str = include_str!("../../../queries/rust/scopes.scm");

const ROLES: [(&str, ScopeType); 6] = [
    ("scope.function", ScopeType::Function),
    ("scope.closure", ScopeType::Closure),
    ("scope.impl", ScopeType::Impl),
    ("scope.trait", ScopeType::Trait),
    ("scope.module", ScopeType::Module),
    ("scope.block", ScopeType::Block),
];

/// The kind of scope each scope-introducing node opens.
pub fn scope_kinds(source_code: &str, root_node: Node) -> Result<Roles<ScopeType>, String> {
    query::scope_kinds(QUERY, source_code, root_node, &ROLES)
}
