use crate::logger::Logger;
use lintric_core::{analyze_code, AnalysisResult, Language};
use std::fs;
use std::path::{Path, PathBuf};

/// Analysis results for one or more paths, together with whether anything failed.
///
/// Failures are reported as they happen and analysis continues, so one unreadable
/// file does not hide the metrics for every other file. `failed` carries that fact
/// on to the process exit code.
#[derive(Default)]
pub struct AnalysisOutcome {
    pub results: Vec<AnalysisResult>,
    pub failed: bool,
}

impl AnalysisOutcome {
    pub fn merge(mut self, other: Self) -> Self {
        self.results.extend(other.results);
        self.failed |= other.failed;
        self
    }

    pub fn files_analyzed(&self) -> usize {
        self.results.len()
    }

    pub fn total_complexity_score(&self) -> f64 {
        self.results
            .iter()
            .map(|result| result.overall_complexity_score)
            .sum()
    }

    fn analyzed(results: Vec<AnalysisResult>) -> Self {
        Self {
            results,
            failed: false,
        }
    }

    fn failure() -> Self {
        Self {
            results: Vec::new(),
            failed: true,
        }
    }
}

/// Process a single file and return its analysis result
pub fn process_file(file_path: &Path) -> Result<AnalysisResult, String> {
    let (_, result) = analyze_code(file_path.to_string_lossy().into_owned())?;
    Ok(result)
}

/// Process a directory recursively and return analysis results for all supported files
pub fn process_directory(path: &Path, logger: &dyn Logger) -> Result<AnalysisOutcome, String> {
    let entries = fs::read_dir(path)
        .map_err(|e| format!("Error reading directory {}: {}", path.display(), e))?;

    Ok(entries
        .map(|entry| match entry {
            Ok(entry) => process_entry(&entry.path(), logger),
            Err(e) => {
                logger.error(&format!("Error reading directory entry: {e}"));
                AnalysisOutcome::failure()
            }
        })
        .fold(AnalysisOutcome::default(), AnalysisOutcome::merge))
}

/// Process a path (file or directory) and return analysis results
pub fn process_path(path_str: &str, logger: &dyn Logger) -> AnalysisOutcome {
    let path = PathBuf::from(path_str);

    if path.is_file() {
        process_requested_file(&path, logger)
    } else if path.is_dir() {
        into_outcome(process_directory(&path, logger), logger, || {
            format!("Error processing directory {}", path.display())
        })
    } else {
        logger.error(&format!(
            "Error: Path {} is neither a file nor a directory.",
            path.display()
        ));
        AnalysisOutcome::failure()
    }
}

/// A file named on the command line: an unsupported extension is worth a warning,
/// unlike the same file encountered while walking a directory.
fn process_requested_file(path: &Path, logger: &dyn Logger) -> AnalysisOutcome {
    if Language::from_extension(path).is_none() {
        logger.warn(&format!(
            "Warning: Skipping unsupported file type: {}",
            path.display()
        ));
        return AnalysisOutcome::default();
    }
    analyze(path, logger)
}

fn process_entry(path: &Path, logger: &dyn Logger) -> AnalysisOutcome {
    if path.is_file() && Language::from_extension(path).is_some() {
        analyze(path, logger)
    } else if path.is_dir() {
        into_outcome(process_directory(path, logger), logger, || {
            format!("Error processing subdirectory {}", path.display())
        })
    } else {
        AnalysisOutcome::default()
    }
}

fn analyze(path: &Path, logger: &dyn Logger) -> AnalysisOutcome {
    match process_file(path) {
        Ok(result) => AnalysisOutcome::analyzed(vec![result]),
        Err(e) => {
            logger.error(&format!("Error processing file {}: {}", path.display(), e));
            AnalysisOutcome::failure()
        }
    }
}

fn into_outcome(
    outcome: Result<AnalysisOutcome, String>,
    logger: &dyn Logger,
    context: impl Fn() -> String,
) -> AnalysisOutcome {
    outcome.unwrap_or_else(|e| {
        logger.error(&format!("{}: {}", context(), e));
        AnalysisOutcome::failure()
    })
}
