use crate::collectors::common::definition_collectors::DefinitionCollector;
use crate::collectors::common::dependency_collectors::DependencyCollector;
use crate::collectors::rust::rust_definition_collector::RustDefinitionCollector;
use crate::collectors::rust::rust_dependency_collector::RustDependencyCollector;
use crate::collectors::typescript::typescript_definition_collector::TypescriptDefinitionCollector;
use crate::collectors::typescript::typescript_dependency_collector::TypescriptDependencyCollector;
use petgraph::graph::{DiGraph, NodeIndex};
use std::collections::HashMap;
use tree_sitter::Parser as TreeSitterParser;

#[allow(clippy::type_complexity)]
pub fn parse_code(
    content: &str,
    file_path: &str,
) -> Result<(DiGraph<usize, usize>, HashMap<usize, NodeIndex>), String> {
    let mut parser = TreeSitterParser::new();

    if file_path.ends_with(".rs") {
        parser
            .set_language(&tree_sitter_rust::language())
            .map_err(|e| format!("Error loading Rust grammar: {e}"))?;
        let tree = parser
            .parse(content, None)
            .ok_or_else(|| "Failed to parse the source code.".to_string())?;

        let definitions =
            RustDefinitionCollector::collect_definitions_from_root(tree.root_node(), content)?;
        RustDependencyCollector::collect_dependencies_from_root(
            tree.root_node(),
            content,
            &definitions,
        )
    } else if file_path.ends_with(".ts") {
        parser
            .set_language(&tree_sitter_typescript::language_typescript())
            .map_err(|e| format!("Error loading TypeScript grammar: {e}"))?;
        let tree = parser
            .parse(content, None)
            .ok_or_else(|| "Failed to parse the source code.".to_string())?;
        let definitions = TypescriptDefinitionCollector::collect_definitions_from_root(
            tree.root_node(),
            content,
        )?;
        TypescriptDependencyCollector::collect_dependencies_from_root(
            tree.root_node(),
            content,
            &definitions,
        )
    } else if file_path.ends_with(".tsx") {
        parser
            .set_language(&tree_sitter_typescript::language_tsx())
            .map_err(|e| format!("Error loading TSX grammar: {e}"))?;
        let tree = parser
            .parse(content, None)
            .ok_or_else(|| "Failed to parse the source code.".to_string())?;
        let definitions = TypescriptDefinitionCollector::collect_definitions_from_root(
            tree.root_node(),
            content,
        )?;
        TypescriptDependencyCollector::collect_dependencies_from_root(
            tree.root_node(),
            content,
            &definitions,
        )
    } else {
        Err(format!(
            "Unsupported file extension for {file_path}. Only .rs, .ts, .tsx are supported."
        ))
    }
}
