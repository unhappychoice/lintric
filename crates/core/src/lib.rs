pub mod ast_formatter;
pub mod definition_collectors;
pub mod definition_context;
pub mod dependency_resolver;
pub mod file_parser;
pub mod languages;
pub mod metric_calculator;
pub mod models;
pub mod usage_collector;

use serde::Serialize;

pub use file_parser::FileParser;
use languages::language_factory;
use metric_calculator::calculate_metrics;
pub use models::{
    Accessibility, AnalysisMetadata, AnalysisResult, IntermediateRepresentation, Language,
    LineMetrics, ScopeId, ScopeTree, ScopeType, SymbolTable,
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

    // For dependency resolution, we need to create a temporary SymbolTable from the new context
    // This is a compatibility layer while dependency resolver is being refactored
    let mut symbol_table = SymbolTable::new();

    // Create scopes in SymbolTable to match the structure from new context
    // Sort scopes by ID to ensure proper parent-child creation order
    let mut all_scopes = context.scopes.get_all_scopes();
    all_scopes.sort_by_key(|scope| scope.id);

    for scope in &all_scopes {
        let scope_id = scope.id;
        let parent_id = scope.parent;
        let position = scope.position;
        let scope_type = scope.scope_type.clone();

        // Skip root scope (ID 0) as it's created automatically
        if scope_id == 0 {
            continue;
        }

        // Create scope in symbol table
        let created_scope_id = symbol_table
            .scopes
            .create_scope(parent_id, scope_type, position);

        // Ensure the scope ID matches (they should be the same)
        assert_eq!(
            created_scope_id, scope_id,
            "Scope ID mismatch during migration: expected {}, got {}",
            scope_id, created_scope_id
        );
    }

    // Add definitions to the SymbolTable for dependency resolution
    for definition in &definitions {
        symbol_table.add_enhanced_symbol(definition.name.clone(), definition.clone());
    }

    // Resolve dependencies using existing resolver
    let dependencies = language_factory::get_dependency_resolver(language.clone(), symbol_table)?
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
