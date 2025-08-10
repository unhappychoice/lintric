use petgraph::graph::{DiGraph, NodeIndex};
use std::collections::HashMap;
use tree_sitter::Parser as TreeSitterParser;

type Dependencies = (DiGraph<usize, usize>, HashMap<usize, NodeIndex>);

pub trait DependencyCollector {
    fn collect_dependencies(
        content: &str,
        is_tsx: bool, // For TypeScript, this might be needed
        parser: &mut TreeSitterParser,
        definitions: &HashMap<String, usize>,
    ) -> Result<Dependencies, String>;
}
