pub mod collectors;
pub mod file_parser;
pub mod metric_calculator;
pub mod models;

use serde::Serialize;

use collectors::collector_factory;
use file_parser::FileParser;
use metric_calculator::calculate_metrics;
pub use models::{AnalysisResult, IntermediateRepresentation, Language, LineMetrics};

#[derive(Debug, Serialize)]
pub struct DependencyEdge {
    pub source: usize,
    pub target: usize,
}

#[derive(Debug, Serialize)]
pub struct DefinitionEntry {
    pub name: String,
    pub line: usize,
}

pub fn analyze_code(
    file_path: String,
) -> Result<(IntermediateRepresentation, AnalysisResult), String> {
    let file_parser = FileParser::new(file_path.clone())?;
    let (file_content, language, tree) = file_parser.parse()?;

    let ir = _get_intermediate_representation(file_path, &file_content, language, tree)?;
    let mut result = calculate_metrics(&ir, &file_content)?;

    result
        .line_metrics
        .retain(|line_metrics| line_metrics.total_dependencies > 0);

    Ok((ir, result))
}

pub fn get_intermediate_representation(
    file_path: String,
) -> Result<IntermediateRepresentation, String> {
    let file_parser = FileParser::new(file_path.clone())?;
    let (file_content, language, tree) = file_parser.parse()?;

    _get_intermediate_representation(file_path, &file_content, language, tree)
}

pub fn get_s_expression(path: String) -> Result<String, String> {
    FileParser::new(path)?.parse_as_s_expression()
}

fn _get_intermediate_representation(
    file_path: String,
    file_content: &str,
    language: Language,
    tree: tree_sitter::Tree,
) -> Result<IntermediateRepresentation, String> {
    let total_lines = file_content.lines().count();

    let def_collector_instance =
        collector_factory::get_definition_collector(language.clone(), file_content)?;
    let definitions = def_collector_instance
        .collect_definitions_from_root(tree.root_node())
        .map_err(|e| format!("Failed to collect definitions: {e}"))?;

    let dep_collector_instance = collector_factory::get_dependency_collector(language.clone())?;
    let dependencies = dep_collector_instance
        .collect_dependencies_from_root(tree.root_node(), file_content, &definitions)
        .map_err(|e| format!("Failed to collect dependencies: {e}"))?;

    Ok(IntermediateRepresentation::new(
        file_path,
        definitions,
        dependencies,
        language.to_string(),
        total_lines,
    ))
}
