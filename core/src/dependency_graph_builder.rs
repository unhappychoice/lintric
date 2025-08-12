use tree_sitter::{Parser as TreeSitterParser, Tree};

use crate::collectors::collector_factory::{get_definition_collector, get_dependency_collector};
use crate::models::{IntermediateRepresentation, Language};

pub fn build_ir(
    content: &str,
    language: Language,
    file_path: String,
) -> Result<IntermediateRepresentation, String> {
    let tree = parse_file(language.clone(), content)?;
    let definition_collector = get_definition_collector(language.clone());
    let dependency_collector = get_dependency_collector(language.clone());

    let definition_collector_instance = definition_collector?;
    let detailed_definitions =
        definition_collector_instance.collect_definitions_from_root(tree.root_node(), content)?;

    let dependency_collector_instance = dependency_collector?;
    let detailed_dependencies = dependency_collector_instance.collect_dependencies_from_root(
        tree.root_node(),
        content,
        &detailed_definitions,
    )?;

    let language_str = match language {
        Language::Rust => "Rust",
        Language::TypeScript => "TypeScript",
    }
    .to_string();

    let total_lines = content.lines().count();

    Ok(IntermediateRepresentation::new(
        file_path,
        detailed_definitions,
        detailed_dependencies,
        language_str,
        total_lines,
    ))
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
