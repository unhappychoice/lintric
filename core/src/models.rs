use serde::{Deserialize, Serialize};
use std::path::Path;
use tree_sitter::Language as TreeSitterLanguage;
use tree_sitter_rust;
use tree_sitter_typescript;

#[derive(Debug, Clone, PartialEq)]
pub enum Language {
    Rust,
    TypeScript,
    // Add other languages as needed
}

impl Language {
    pub fn from_extension(path: &Path) -> Option<Self> {
        path.extension()
            .and_then(|ext| ext.to_str())
            .and_then(|ext_str| match ext_str {
                "rs" => Some(Language::Rust),
                "ts" | "tsx" | "js" | "jsx" => Some(Language::TypeScript),
                _ => None,
            })
    }

    pub fn get_tree_sitter_language(&self) -> TreeSitterLanguage {
        match self {
            Language::Rust => tree_sitter_rust::language(),
            Language::TypeScript => tree_sitter_typescript::language_typescript(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LineMetrics {
    pub line_number: usize,
    pub total_dependencies: usize,
    pub dependency_distance_cost: f64,
    pub depth: usize,
    pub transitive_dependencies: usize,
    pub dependent_lines: Vec<usize>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AnalysisResult {
    pub file_path: String,
    pub original_file_path: String,
    pub line_metrics: Vec<LineMetrics>,
    pub overall_complexity_score: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OverallAnalysisReport {
    pub results: Vec<AnalysisResult>,
    pub total_files_analyzed: usize,
    pub total_overall_complexity_score: f64,
    pub average_complexity_score: f64,
}
