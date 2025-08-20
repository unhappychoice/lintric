use super::{Definition, Dependency, SerializableUsage};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AnalysisMetadata {
    pub language: String,
    pub total_lines: usize,
    pub analysis_timestamp: String,
    pub lintric_version: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct IntermediateRepresentation {
    pub file_path: String,
    pub definitions: Vec<Definition>,
    pub dependencies: Vec<Dependency>,
    pub usage: Vec<SerializableUsage>,
    pub analysis_metadata: AnalysisMetadata,
}

impl IntermediateRepresentation {
    pub fn new(
        file_path: String,
        definitions: Vec<Definition>,
        dependencies: Vec<Dependency>,
        usage: Vec<SerializableUsage>,
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
            usage,
            analysis_metadata,
        }
    }
}
