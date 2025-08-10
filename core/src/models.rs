use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LineMetrics {
    pub line_number: usize,
    pub total_dependencies: usize,
    pub dependency_distance_cost: f64,
    pub depth: usize,
    pub transitive_dependencies: usize,
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
