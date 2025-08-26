pub mod ast_formatter;
pub mod definition_context;
pub mod dependency_resolver;
pub mod file_parser;
pub mod languages;
pub mod metric_calculator;
pub mod models;

use serde::Serialize;

pub use file_parser::FileParser;
use languages::language_factory;
use metric_calculator::calculate_metrics;
pub use models::{
    Accessibility, AnalysisMetadata, AnalysisResult, IntermediateRepresentation, Language,
    LineMetrics, ScopeId, ScopeTree, ScopeType,
};

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
    // Use new unified analysis with single AST traversal
    let context =
        language_factory::analyze_code_unified(language.clone(), file_content, tree.root_node())?;

    let mut definitions: Vec<_> = context
        .definitions
        .get_all_definitions()
        .values()
        .flatten()
        .cloned()
        .collect();

    // Sort definitions by position for consistent output
    definitions.sort();

    let usages = context.usages.get_all_usages().clone();

    // Resolve dependencies using new context-based resolver
    let dependencies = language_factory::get_dependency_resolver(language.clone(), context)?
        .resolve_dependencies(file_content, tree.root_node(), &usages, &definitions)
        .map_err(|e| format!("Failed to resolve dependencies: {e}"))?;

    Ok(IntermediateRepresentation {
        file_path: file_path.clone(),
        definitions,
        usage: usages,
        dependencies,
        analysis_metadata: AnalysisMetadata {
            language: language.to_string(),
            total_lines: file_content.lines().count(),
            analysis_timestamp: "now".to_string(),
            lintric_version: env!("CARGO_PKG_VERSION").to_string(),
        },
    })
}
