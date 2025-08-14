use serde::{Deserialize, Serialize};
use std::path::Path;
use tree_sitter::Language as TreeSitterLanguage;
use tree_sitter_rust;
use tree_sitter_typescript;

#[derive(Debug, Clone, PartialEq)]
pub enum Language {
    Rust,
    TypeScript,
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

impl std::fmt::Display for Language {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
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

// Enhanced IR structures for debugging (Issue #49)

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum DefinitionType {
    FunctionDefinition,
    VariableDefinition,
    StructDefinition,
    EnumDefinition,
    TypeDefinition,
    ModuleDefinition,
    ClassDefinition,
    InterfaceDefinition,
    ConstDefinition,
    Other(String),
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum DependencyType {
    FunctionCall,
    VariableUse,
    Import,
    StructFieldAccess,
    TypeReference,
    ModuleReference,
    Other(String),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Definition {
    pub name: String,
    pub line_number: usize,
    pub definition_type: DefinitionType,
    pub scope: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Dependency {
    pub source_line: usize,
    pub target_line: usize,
    pub symbol: String,
    pub dependency_type: DependencyType,
    pub context: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct IntermediateRepresentation {
    pub file_path: String,
    pub definitions: Vec<Definition>,
    pub dependencies: Vec<Dependency>,
    pub analysis_metadata: AnalysisMetadata,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AnalysisMetadata {
    pub language: String,
    pub total_lines: usize,
    pub analysis_timestamp: String,
    pub lintric_version: String,
}

impl IntermediateRepresentation {
    pub fn new(
        file_path: String,
        definitions: Vec<Definition>,
        dependencies: Vec<Dependency>,
        language: String,
        total_lines: usize,
    ) -> Self {
        let analysis_metadata = AnalysisMetadata {
            language,
            total_lines,
            analysis_timestamp: "now".to_string(), // Placeholder for timestamp
            lintric_version: env!("CARGO_PKG_VERSION").to_string(),
        };

        IntermediateRepresentation {
            file_path,
            definitions,
            dependencies,
            analysis_metadata,
        }
    }
}
