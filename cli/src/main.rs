use clap::{ArgAction, Parser};
use lintric_core::models::OverallAnalysisReport;
use lintric_core::AnalysisResult;

mod display;
mod file_processor;
mod html_output; // Add this line

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Paths to the source code files or directories to analyze
    #[arg(required = true, action = ArgAction::Append)]
    paths: Vec<String>,

    /// Output in JSON format
    #[arg(long, default_value_t = false)]
    json: bool,

    /// Show verbose output (line-by-line metrics)
    #[arg(long, default_value_t = false)]
    verbose: bool,

    /// Output in HTML format
    #[arg(long, default_value_t = false)]
    html: bool, // Add this line
}

fn main() {
    let args = Args::parse();

    let mut all_results: Vec<AnalysisResult> = Vec::new();
    let mut total_overall_complexity_score = 0.0;
    let mut total_files_analyzed = 0;

    for path_str in &args.paths {
        let (results, score, count) = file_processor::process_path(path_str);
        all_results.extend(results);
        total_overall_complexity_score += score;
        total_files_analyzed += count;
    }

    let overall_report = OverallAnalysisReport {
        results: all_results,
        total_files_analyzed,
        total_overall_complexity_score,
        average_complexity_score: if total_files_analyzed > 0 {
            total_overall_complexity_score / total_files_analyzed as f64
        } else {
            0.0
        },
    };

    if args.json {
        display::display_json(&overall_report);
    } else if args.verbose {
        display::display_verbose(&overall_report);
    } else if args.html {
        // Add this line
        html_output::generate_html_report(&overall_report); // Add this line
    } else {
        display::display_summary(&overall_report);
    }
}
