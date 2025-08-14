use comfy_table::presets::UTF8_FULL_CONDENSED;
use comfy_table::{Cell, Row, Table};
use lintric_core::models::OverallAnalysisReport;
use std::path::Path;

/// Display the analysis results in JSON format
pub fn display_json(overall_report: &OverallAnalysisReport, base_paths: &[String]) {
    #[derive(serde::Serialize)]
    struct JsonReport {
        results: Vec<lintric_core::models::AnalysisResult>,
        total_files_analyzed: usize,
        total_overall_complexity_score: f64,
        average_complexity_score: f64,
    }

    let report_for_json = JsonReport {
        results: overall_report
            .results
            .iter()
            .map(|r| {
                let mut r_clone = r.clone();
                r_clone.file_path = format_file_path_for_display(&r.file_path, base_paths);
                r_clone
            })
            .collect(),
        total_files_analyzed: overall_report.total_files_analyzed,
        total_overall_complexity_score: overall_report.total_overall_complexity_score,
        average_complexity_score: overall_report.average_complexity_score,
    };

    println!(
        "{}",
        serde_json::to_string_pretty(&report_for_json).unwrap()
    );
}

/// Display verbose analysis results with line-by-line metrics
pub fn display_verbose(overall_report: &OverallAnalysisReport, base_paths: &[String]) {
    let mut sorted_results = overall_report.results.clone();
    sorted_results.sort_by(|a, b| a.file_path.cmp(&b.file_path));

    for result in &sorted_results {
        println!(
            "\n--- Analysis for {} ---",
            format_file_path_for_display(&result.file_path, base_paths)
        );
        let mut table = Table::new();
        table.load_preset(UTF8_FULL_CONDENSED);
        table.set_header(vec![
            "Line",
            "Total Deps",
            "Dist Cost",
            "Depth",
            "Transitive Deps",
        ]);
        for metrics in &result.line_metrics {
            table.add_row(Row::from(vec![
                Cell::new(metrics.line_number),
                Cell::new(metrics.total_dependencies),
                Cell::new(metrics.dependency_distance_cost),
                Cell::new(metrics.depth),
                Cell::new(metrics.transitive_dependencies),
            ]));
        }
        println!("{table}");
        println!(
            "Overall Complexity Score: {:.2}",
            result.overall_complexity_score
        );
    }

    display_summary(overall_report, base_paths);
}

/// Display summary analysis results with file-level metrics
pub fn display_summary(overall_report: &OverallAnalysisReport, base_paths: &[String]) {
    let mut table = Table::new();
    table.load_preset(UTF8_FULL_CONDENSED);
    table.set_header(vec!["File", "Overall Complexity Score"]);

    let mut sorted_results = overall_report.results.clone();
    sorted_results.sort_by(|a, b| a.file_path.cmp(&b.file_path));

    for result in &sorted_results {
        table.add_row(Row::from(vec![
            Cell::new(format_file_path_for_display(&result.file_path, base_paths)),
            Cell::new(format!("{:.2}", result.overall_complexity_score)),
        ]));
    }
    println!("{table}");

    display_overall_summary(overall_report);
}

/// Display the overall summary of the analysis
pub fn display_overall_summary(overall_report: &OverallAnalysisReport) {
    println!("\n--- Overall Report ---");
    println!(
        "Total Files Analyzed: {}",
        overall_report.total_files_analyzed
    );
    println!(
        "Total Overall Complexity Score: {:.2}",
        overall_report.total_overall_complexity_score
    );
    println!(
        "Average Complexity Score: {:.2}",
        overall_report.average_complexity_score
    );
}

fn format_file_path_for_display(file_path: &str, base_paths: &[String]) -> String {
    let normalized_path = file_path.replace('\\', "/");
    let original_path = Path::new(&normalized_path);

    let mut best_stripped_path = None;
    let mut best_stripped_len = usize::MAX;

    for base_path_str in base_paths {
        let base_path_normalized_str = base_path_str.replace('\\', "/");
        let base_path = Path::new(&base_path_normalized_str);

        // If the base path is a file, we want to strip its parent directory if it matches
        // Otherwise, use the base path directly
        let effective_base_path = if base_path.is_file() {
            base_path.parent().unwrap_or(base_path)
        } else {
            base_path
        };

        if let Ok(stripped) = original_path.strip_prefix(effective_base_path) {
            let stripped_str = stripped.to_string_lossy().into_owned();
            // Prefer the shortest stripped path, which implies the most specific base path
            if stripped_str.len() < best_stripped_len {
                best_stripped_path = Some(stripped_str.clone());
                best_stripped_len = stripped_str.len();
            }
        }
    }

    best_stripped_path.unwrap_or(normalized_path)
}
