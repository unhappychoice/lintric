pub mod models;
pub mod ast_parser;
pub mod metric_calculator;
pub mod parsers;

pub use models::{AnalysisResult, LineMetrics};
use ast_parser::parse_code;
use metric_calculator::calculate_metrics;

pub fn analyze_code(
    content: &str,
    file_path: String,
    original_file_path: String,
) -> Result<AnalysisResult, String> {
    let (graph, _line_nodes) = parse_code(content, &file_path)?;

    calculate_metrics(graph, content, file_path, original_file_path)
}
