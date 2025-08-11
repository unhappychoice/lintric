pub mod collectors;
pub mod dependency_graph_builder;
pub mod metric_calculator;
pub mod models;

use petgraph::visit::EdgeRef;
use serde::Serialize;
use std::fs;
use std::path::PathBuf;

use collectors::collector_factory;
use dependency_graph_builder::build_graph;
use metric_calculator::calculate_metrics;
pub use models::{AnalysisResult, Language, LineMetrics};

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
    content: &str,
    file_path: String,
    original_file_path: String,
) -> Result<AnalysisResult, String> {
    let path = std::path::PathBuf::from(&original_file_path);
    let language = Language::from_extension(&path)
        .ok_or_else(|| format!("Unsupported file type for analysis: {original_file_path}"))?;

    let (graph, _line_nodes) = build_graph(content, language)?;

    calculate_metrics(graph, content, file_path, original_file_path)
}

pub fn parse_source_file(path: String) -> Result<String, String> {
    let (file_content, _, tree) = prepare_analysis_data(path, "AST")?;
    Ok(format_s_expression(tree.root_node(), 0, &file_content))
}

pub fn get_definitions(path: String) -> Result<Vec<DefinitionEntry>, String> {
    let (file_content, language, tree) = prepare_analysis_data(path, "definitions")?;
    let collector_instance = collector_factory::get_definition_collector(language)?;
    let definitions_map = collector_instance
        .collect_definitions_from_root(tree.root_node(), &file_content)
        .map_err(|e| format!("Failed to collect definitions: {e}"))?;

    let mut definitions: Vec<DefinitionEntry> = definitions_map
        .into_iter()
        .map(|(name, line)| DefinitionEntry { name, line })
        .collect();
    definitions.sort_by(|a, b| a.line.cmp(&b.line).then_with(|| a.name.cmp(&b.name)));
    Ok(definitions)
}

pub fn get_dependencies(path: String) -> Result<Vec<DependencyEdge>, String> {
    let (file_content, language, tree) = prepare_analysis_data(path, "dependencies")?;
    let def_collector_instance = collector_factory::get_definition_collector(language.clone())?;
    let defs = def_collector_instance
        .collect_definitions_from_root(tree.root_node(), &file_content)
        .map_err(|e| format!("Failed to collect definitions: {e}"))?;
    let dep_collector_instance = collector_factory::get_dependency_collector(language)?;
    let (graph, _) = dep_collector_instance
        .collect_dependencies_from_root(tree.root_node(), &file_content, &defs)
        .map_err(|e| format!("Failed to collect dependencies: {e}"))?;

    let mut edges: Vec<DependencyEdge> = Vec::new();
    for edge_ref in graph.edge_references() {
        edges.push(DependencyEdge {
            source: graph[edge_ref.source()],
            target: graph[edge_ref.target()],
        });
    }
    Ok(edges)
}

fn prepare_analysis_data(
    path: String,
    context: &str,
) -> Result<(String, Language, tree_sitter::Tree), String> {
    let file_content =
        fs::read_to_string(&path).map_err(|e| format!("Failed to read file {path}: {e}"))?;
    let language = Language::from_extension(&PathBuf::from(&path))
        .ok_or_else(|| format!("Unsupported file type for {context}: {path}"))?;
    let tree = dependency_graph_builder::parse_file(language.clone(), &file_content)?;
    Ok((file_content, language, tree))
}

fn format_s_expression(node: tree_sitter::Node, depth: usize, file_content: &str) -> String {
    let indent = "  ".repeat(depth);
    let mut s_expr = String::new();

    let node_text = if node.kind() == "identifier" {
        format!(
            "identifier {}",
            node.utf8_text(file_content.as_bytes()).unwrap()
        )
    } else {
        node.kind().to_string()
    };

    s_expr.push_str(&format!("({node_text}"));

    let mut children_s_exprs = Vec::new();
    for i in 0..node.named_child_count() {
        if let Some(child) = node.named_child(i) {
            children_s_exprs.push(format_s_expression(child, depth + 1, file_content));
        }
    }

    if !children_s_exprs.is_empty() {
        for child_s_expr in children_s_exprs {
            s_expr.push_str(&format!("\n{}{}", "  ".repeat(depth + 1), child_s_expr));
        }
        s_expr.push_str(&format!("\n{indent})"));
    } else {
        s_expr.push(')');
    }

    s_expr
}
