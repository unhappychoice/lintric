use clap::{ArgAction, Parser, Subcommand};
use lintric_core::models::OverallAnalysisReport;
use lintric_core::AnalysisResult;
use serde_json;

mod display;
mod file_processor;
mod html_output;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Option<Commands>,

    /// Paths to the source code files or directories to analyze
    #[arg(required = false, action = ArgAction::Append)]
    paths: Vec<String>,

    /// Output in JSON format
    #[arg(long, default_value_t = false)]
    json: bool,

    /// Show verbose output (line-by-line metrics)
    #[arg(long, default_value_t = false)]
    verbose: bool,

    /// Output in HTML format
    #[arg(long, default_value_t = false)]
    html: bool,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Debugging utilities
    Debug {
        #[command(subcommand)]
        command: DebugCommands,
    },
}

#[derive(Subcommand, Debug)]
enum DebugCommands {
    /// Outputs the AST of the input file
    Ast {
        /// Path to the source code file to analyze
        #[arg(required = true)]
        path: String,
    },
    /// Outputs a list of definitions in the input file
    Definition {
        /// Path to the source code file to analyze
        #[arg(required = true)]
        path: String,
    },
    /// Outputs a list of definitions and dependencies in the input file
    Dependency {
        /// Path to the source code file to analyze
        #[arg(required = true)]
        path: String,
    },
}

fn main() {
    let args = Args::parse();

    match args.command {
        Some(Commands::Debug { command }) => match command {
            DebugCommands::Ast { path } => {
                match lintric_core::parse_source_file(path) {
                    Ok(s_expr_output) => {
                        println!("{}", s_expr_output);
                    }
                    Err(e) => {
                        eprintln!("Error: {}", e);
                    }
                }
            }
            DebugCommands::Definition { path } => {
                match lintric_core::get_definitions(path) {
                    Ok(definitions) => {
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&definitions)
                                .expect("Failed to serialize definitions to JSON")
                        );
                    }
                    Err(e) => {
                        eprintln!("Error: {}", e);
                    }
                }
            }
            DebugCommands::Dependency { path } => {
                match lintric_core::get_dependencies(path) {
                    Ok(edges) => {
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&edges)
                                .expect("Failed to serialize dependencies to JSON")
                        );
                    }
                    Err(e) => {
                        eprintln!("Error: {}", e);
                    }
                }
            }
        },
        None => {
            // Existing logic for analysis
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
                html_output::generate_html_report(&overall_report);
            } else {
                display::display_summary(&overall_report);
            }
        }
    }
}
