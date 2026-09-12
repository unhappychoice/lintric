use crate::file_processor::AnalysisOutcome;
use crate::logger::Logger;
use clap::{ArgAction, Parser, Subcommand};
use lintric_core::models::OverallAnalysisReport;
use std::ffi::OsString;
use std::process::ExitCode;

mod display;
mod file_processor;
mod html_output;
pub mod logger;

/// Whether a run completed without reporting any error.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Outcome {
    Success,
    Failure,
}

impl Outcome {
    fn of(failed: bool) -> Self {
        if failed {
            Self::Failure
        } else {
            Self::Success
        }
    }
}

impl From<Outcome> for ExitCode {
    fn from(outcome: Outcome) -> Self {
        match outcome {
            Outcome::Success => ExitCode::SUCCESS,
            Outcome::Failure => ExitCode::FAILURE,
        }
    }
}

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

pub fn run() -> ExitCode {
    let logger = logger::StdIoLogger;
    run_from_iter(std::env::args_os(), &logger).into()
}

/// Execute CLI with provided arguments and logger.
/// The first arg should be the binary name (e.g., "lintric").
pub fn run_from_iter<I, T>(args: I, logger: &dyn Logger) -> Outcome
where
    I: IntoIterator<Item = T>,
    T: Into<OsString> + Clone,
{
    let args = Args::parse_from(args);

    match args.command {
        Some(Commands::Debug { command }) => run_debug(command, logger),
        None => run_analysis(&args, logger),
    }
}

fn run_debug(command: DebugCommands, logger: &dyn Logger) -> Outcome {
    let output = match command {
        DebugCommands::Ast { path } => lintric_core::get_s_expression(path),
        DebugCommands::IntermediateRepresentation { path } => {
            lintric_core::get_intermediate_representation(path).and_then(|ir| {
                serde_json::to_string_pretty(&ir)
                    .map_err(|e| format!("Failed to serialize IR to JSON: {e}"))
            })
        }
    };

    match output {
        Ok(output) => {
            logger.info(&output);
            Outcome::Success
        }
        Err(e) => {
            logger.error(&format!("Error: {e}"));
            Outcome::Failure
        }
    }
}

fn run_analysis(args: &Args, logger: &dyn Logger) -> Outcome {
    let outcome = args
        .paths
        .iter()
        .map(|path| file_processor::process_path(path, logger))
        .fold(AnalysisOutcome::default(), AnalysisOutcome::merge);

    let failed = outcome.failed;
    let report = build_report(outcome);
    let rendered = render(args, &report, logger);

    Outcome::of(failed || rendered == Outcome::Failure)
}

fn build_report(outcome: AnalysisOutcome) -> OverallAnalysisReport {
    let total_files_analyzed = outcome.files_analyzed();
    let total_overall_complexity_score = outcome.total_complexity_score();

    OverallAnalysisReport {
        results: outcome.results,
        total_files_analyzed,
        total_overall_complexity_score,
        average_complexity_score: if total_files_analyzed > 0 {
            total_overall_complexity_score / total_files_analyzed as f64
        } else {
            0.0
        },
    }
}

fn render(args: &Args, report: &OverallAnalysisReport, logger: &dyn Logger) -> Outcome {
    if args.json {
        display::display_json(report, &args.paths, logger);
    } else if args.verbose {
        display::display_verbose(report, &args.paths, logger);
    } else if args.html {
        return html_output::generate_html_report(report, logger);
    } else {
        display::display_summary(report, &args.paths, logger);
    }
    Outcome::Success
}
