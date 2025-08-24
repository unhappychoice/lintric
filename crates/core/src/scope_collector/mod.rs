use crate::models::{Definition, ScopeTree, SymbolTable, Usage};
use tree_sitter::Node;

pub trait ScopeCollector {
    fn collect(
        &self,
        source_code: &str,
        root_node: Node,
        usage_nodes: &[Usage],
        definitions: &[Definition],
    ) -> Result<SymbolTable, String>;

    fn scopes(&mut self, root_node: Node, source_code: &str) -> Result<ScopeTree, String>;
}
