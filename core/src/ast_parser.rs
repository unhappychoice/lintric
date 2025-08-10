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
        let definitions =
            RustDefinitionCollector::collect_definitions(content, false, &mut parser)?;
        RustDependencyCollector::collect_dependencies(content, false, &mut parser, &definitions)
    } else if file_path.ends_with(".ts") {
        let definitions =
            TypescriptDefinitionCollector::collect_definitions(content, false, &mut parser)?;
        TypescriptDependencyCollector::collect_dependencies(
            content,
            false,
            &mut parser,
            &definitions,
        )
    } else if file_path.ends_with(".tsx") {
        let definitions =
            TypescriptDefinitionCollector::collect_definitions(content, true, &mut parser)?;
        TypescriptDependencyCollector::collect_dependencies(
            content,
            true,
            &mut parser,
            &definitions,
        )
    } else {
        Err(format!(
            "Unsupported file extension for {file_path}. Only .rs, .ts, .tsx are supported."
        ))
    }
}
