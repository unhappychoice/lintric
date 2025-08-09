use serde::{Serialize, Deserialize};
use approx::abs_diff_eq;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LineMetrics {
    pub line_number: usize,
    pub total_dependencies: usize,
    pub dependency_distance_cost: f64,
    pub depth: usize,
    pub transitive_dependencies: usize,
}

impl PartialEq for LineMetrics {
    fn eq(&self, other: &Self) -> bool {
        self.line_number == other.line_number &&
        self.total_dependencies == other.total_dependencies &&
        abs_diff_eq!(self.dependency_distance_cost, other.dependency_distance_cost, epsilon = f64::EPSILON) &&
        self.depth == other.depth &&
        self.transitive_dependencies == other.transitive_dependencies
    }
}


#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AnalysisResult {
    pub file_path: String,
    pub line_metrics: Vec<LineMetrics>,
    pub overall_complexity_score: f64,
}

impl PartialEq for AnalysisResult {
    fn eq(&self, other: &Self) -> bool {
        self.file_path == other.file_path &&
        self.line_metrics == other.line_metrics &&
        abs_diff_eq!(self.overall_complexity_score, other.overall_complexity_score, epsilon = f64::EPSILON)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OverallAnalysisReport {
    pub results: Vec<AnalysisResult>,
    pub total_files_analyzed: usize,
    pub total_overall_complexity_score: f64,
    pub average_complexity_score: f64,
}
