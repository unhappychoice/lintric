use comfy_table::presets::UTF8_FULL_CONDENSED;
use comfy_table::{Cell, Row, Table};
use lintric_core::models::OverallAnalysisReport;

/// Display the analysis results in JSON format
pub fn display_json(overall_report: &OverallAnalysisReport) {
    #[derive(serde::Serialize)]
    struct JsonReport {
        results: Vec<lintric_core::models::AnalysisResult>,
        total_files_analyzed: usize,
        total_overall_complexity_score: f64,
        average_complexity_score: f64,
    }

    let mut filtered_results = overall_report.results.clone();
    for result in &mut filtered_results {
        result
            .line_metrics
            .retain(|metrics| metrics.total_dependencies > 0);
    }

    let report_for_json = JsonReport {
        results: filtered_results,
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
pub fn display_verbose(overall_report: &OverallAnalysisReport) {
    let mut sorted_results = overall_report.results.clone();
    sorted_results.sort_by(|a, b| a.file_path.cmp(&b.file_path));

    for result in &sorted_results {
        println!("\n--- Analysis for {} ---", result.file_path);
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

    display_summary(overall_report);
}

/// Display summary analysis results with file-level metrics
pub fn display_summary(overall_report: &OverallAnalysisReport) {
    let mut table = Table::new();
    table.load_preset(UTF8_FULL_CONDENSED);
    table.set_header(vec!["File", "Overall Complexity Score"]);

    // Sort results by file_path before displaying
    let mut sorted_results = overall_report.results.clone();
    sorted_results.sort_by(|a, b| a.file_path.cmp(&b.file_path));

    for result in &sorted_results {
        table.add_row(Row::from(vec![
            Cell::new(&result.file_path),
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
