use petgraph::graph::{DiGraph, NodeIndex};
use std::collections::HashMap;
use tree_sitter::{Parser as TreeSitterParser, Tree};

use crate::collectors::collector_factory::{get_definition_collector, get_dependency_collector};
use crate::models::Language;

pub fn build_graph(
    content: &str,
    language: Language,
) -> Result<(DiGraph<usize, usize>, HashMap<usize, NodeIndex>), String> {
    let tree = parse_file(language.clone(), content)?;
    let definition_collector = get_definition_collector(language.clone());
    let dependency_collector = get_dependency_collector(language);

    let definition_collector_instance = definition_collector?;
    let definitions =
        definition_collector_instance.collect_definitions_from_root(tree.root_node(), content)?;

    let dependency_collector_instance = dependency_collector?;
    let dependencies = dependency_collector_instance.collect_dependencies_from_root(
        tree.root_node(),
        content,
        &definitions,
    )?;
    Ok(dependencies)
}

pub fn parse_file(language: Language, content: &str) -> Result<Tree, String> {
    let mut parser = TreeSitterParser::new();

    let lang = language.get_tree_sitter_language();

    parser
        .set_language(&lang)
        .map_err(|e| format!("Error loading grammar: {e}"))?;

    let tree = parser
        .parse(content, None)
        .ok_or_else(|| "Failed to parse the source code.".to_string())?;

    Ok(tree)
}
