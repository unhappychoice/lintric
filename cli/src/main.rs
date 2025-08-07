use clap::{Parser, ArgAction};
use lintric_core::{analyze_code, AnalysisResult};
use lintric_core::models::OverallAnalysisReport;
use std::fs;
use std::path::{Path, PathBuf};
use comfy_table::{Table, Row, Cell};

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
}

fn main() {
    let args = Args::parse();

    let mut all_results: Vec<AnalysisResult> = Vec::new();
    let mut total_overall_complexity_score = 0.0;
    let mut total_files_analyzed = 0;

    for path_str in args.paths {
        let path = PathBuf::from(path_str);
        if path.is_file() {
            match process_file(&path) {
                Ok(result) => {
                    all_results.push(result.clone());
                    total_overall_complexity_score += result.overall_complexity_score;
                    total_files_analyzed += 1;
                }
                Err(e) => eprintln!("Error processing file {}: {}", path.display(), e),
            }
        } else if path.is_dir() {
            match process_directory(&path) {
                Ok(results) => {
                    for result in results {
                        all_results.push(result.clone());
                        total_overall_complexity_score += result.overall_complexity_score;
                        total_files_analyzed += 1;
                    }
                }
                Err(e) => eprintln!("Error processing directory {}: {}", path.display(), e),
            }
        } else {
            eprintln!("Error: Path {} is neither a file nor a directory.", path.display());
        }
    }

    let overall_report = OverallAnalysisReport {
        results: all_results,
        total_files_analyzed,
        total_overall_complexity_score,
    };

    if args.json {
        println!("{}", serde_json::to_string_pretty(&overall_report).unwrap());
    } else {
        if args.verbose {
            for result in &overall_report.results {
                println!("\n--- Analysis for {} ---", result.file_path);
                let mut table = Table::new();
                table.set_header(vec!["Line", "Total Deps", "Dist Cost", "Depth", "Transitive Deps"]);
                for metrics in &result.line_metrics {
                    table.add_row(Row::from(vec![
                        Cell::new(metrics.line_number),
                        Cell::new(metrics.total_dependencies),
                        Cell::new(metrics.dependency_distance_cost),
                        Cell::new(metrics.depth),
                        Cell::new(metrics.transitive_dependencies),
                    ]));
                }
                println!("{}", table);
                println!("Overall Complexity Score: {:.2}", result.overall_complexity_score);
            }
        } else {
            let mut table = Table::new();
            table.set_header(vec!["File", "Overall Complexity Score"]);
            for result in &overall_report.results {
                table.add_row(Row::from(vec![
                    Cell::new(&result.file_path),
                    Cell::new(format!("{:.2}", result.overall_complexity_score)),
                ]));
            }
            println!("{}", table);
        }
        println!("\n--- Overall Report ---");
        println!("Total Files Analyzed: {}", overall_report.total_files_analyzed);
        println!("Total Overall Complexity Score: {:.2}", overall_report.total_overall_complexity_score);
    }
}

fn process_file(path: &Path) -> Result<AnalysisResult, String> {
    let content = fs::read_to_string(path)
        .map_err(|e| format!("Error reading file {}: {}", path.display(), e))?;
    analyze_code(&content, path.to_string_lossy().into_owned())
}

fn process_directory(path: &Path) -> Result<Vec<AnalysisResult>, String> {
    let mut results = Vec::new();
    for entry in fs::read_dir(path)
        .map_err(|e| format!("Error reading directory {}: {}", path.display(), e))? {
        let entry = entry.map_err(|e| format!("Error reading directory entry: {}", e))?;
        let entry_path = entry.path();

        if entry_path.is_file() && entry_path.extension().map_or(false, |ext| {
            let ext_str = ext.to_string_lossy();
            ext_str == "rs" || ext_str == "ts" || ext_str == "tsx"
        }) {
            match process_file(&entry_path) {
                Ok(result) => results.push(result),
                Err(e) => eprintln!("Error processing file {}: {}", entry_path.display(), e),
            }
        } else if entry_path.is_dir() {
            match process_directory(&entry_path) {
                Ok(mut sub_results) => results.append(&mut sub_results),
                Err(e) => eprintln!("Error processing subdirectory {}: {}", entry_path.display(), e),
            }
        }
    }
    Ok(results)
}