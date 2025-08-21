pub mod collectors;
pub mod file_parser;
pub mod metric_calculator;
pub mod models;
pub mod s_expression_formatter;

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

pub fn analyze_content(
    content: String,
    language: Language,
) -> Result<(IntermediateRepresentation, AnalysisResult), String> {
    let file_parser = FileParser::from_content(content.clone(), language);
    let (file_content, language, tree) = file_parser.parse()?;

    let ir =
        _get_intermediate_representation("<memory>".to_string(), &file_content, language, tree)?;
    let mut result = calculate_metrics(&ir, &file_content)?;

    result
        .line_metrics
        .retain(|line_metrics| line_metrics.total_dependencies > 0);

    Ok((ir, result))
}

pub fn get_s_expression_from_content(
    content: String,
    language: Language,
) -> Result<String, String> {
    let file_parser = FileParser::from_content(content, language);
    file_parser.parse_as_s_expression()
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

    let usage_collector_instance =
        collector_factory::get_usage_node_collector(language.clone(), file_content)
            .map_err(|e| format!("Failed to get usage node collector: {e}"))?;
    let usage_nodes = usage_collector_instance
        .collect_usage_nodes(tree.root_node(), file_content)
        .map_err(|e| format!("Failed to collect usage nodes: {e}"))?;

    let dependency_resolver_instance = collector_factory::get_dependency_resolver(language.clone())
        .map_err(|e| format!("Failed to get dependency resolver: {e}"))?;
    let dependencies = dependency_resolver_instance
        .resolve_dependencies(file_content, tree.root_node(), &usage_nodes, &definitions)
        .map_err(|e| format!("Failed to resolve dependencies: {e}"))?;

    // Usage is now directly serializable

    Ok(IntermediateRepresentation::new(
        file_path,
        definitions,
        dependencies,
        usage_nodes,
        language.to_string(),
        total_lines,
    ))
}
