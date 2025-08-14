// Public entry point for the CLI
use crate::logger::Logger;

pub fn run() {
    let args = Args::parse();
    let logger = logger::StdIoLogger;

    match args.command {
        Some(Commands::Debug { command }) => match command {
            DebugCommands::Ast { path } => match lintric_core::get_s_expression(path) {
                Ok(s_expr_output) => {
                    logger.info(&s_expr_output);
                }
                Err(e) => {
                    logger.error(&format!("Error: {e}"));
                }
            },
            DebugCommands::IntermediateRepresentation { path } => {
                match lintric_core::get_intermediate_representation(path) {
                    Ok(ir) => {
                        logger.info(
                            &serde_json::to_string_pretty(&ir)
                                .expect("Failed to serialize IR to JSON"),
                        );
                    }
                    Err(e) => {
                        logger.error(&format!("Error: {e}"));
                    }
                }
            }
        },
        None => {
            let mut all_results: Vec<lintric_core::AnalysisResult> = Vec::new();
            let mut total_overall_complexity_score = 0.0;
            let mut total_files_analyzed = 0;

            for path_str in &args.paths {
                let (results, score, count) = file_processor::process_path(path_str, &logger);
                all_results.extend(results);
                total_overall_complexity_score += score;
                total_files_analyzed += count;
            }

            let overall_report = lintric_core::models::OverallAnalysisReport {
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
                display::display_json(&overall_report, &args.paths, &logger);
            } else if args.verbose {
                display::display_verbose(&overall_report, &args.paths, &logger);
            } else if args.html {
                html_output::generate_html_report(&overall_report, &logger);
            } else {
                display::display_summary(&overall_report, &args.paths, &logger);
            }
        }
    }
}

use clap::{ArgAction, Parser, Subcommand};

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
    /// Outputs a list of definitions and dependencies in the input file
    #[command(name = "ir", about = "Outputs the IR of the input file")]
    IntermediateRepresentation {
        /// Path to the source code file to analyze
        #[arg(required = true)]
        path: String,
    },
}

mod display;
mod file_processor;
mod html_output;
pub mod logger;
