use lintric_core::{analyze_code, AnalysisResult};
use std::fs;
use std::path::{Path, PathBuf};

/// Process a single file and return its analysis result
pub fn process_file(path: &Path, base_path: Option<&Path>) -> Result<AnalysisResult, String> {
    let content = fs::read_to_string(path)
        .map_err(|e| format!("Error reading file {}: {}", path.display(), e))?;

    let display_path = if let Some(base) = base_path {
        if let Ok(relative) = path.strip_prefix(base) {
            relative.to_string_lossy().into_owned()
        } else {
            path.to_string_lossy().into_owned()
        }
    } else {
        path.to_string_lossy().into_owned()
    };

    analyze_code(&content, display_path, path.to_string_lossy().into_owned())
}

/// Process a directory recursively and return analysis results for all supported files
pub fn process_directory(
    path: &Path,
    base_path: Option<&Path>,
) -> Result<Vec<AnalysisResult>, String> {
    let mut results = Vec::new();
    for entry in fs::read_dir(path)
        .map_err(|e| format!("Error reading directory {}: {}", path.display(), e))?
    {
        let entry = entry.map_err(|e| format!("Error reading directory entry: {e}"))?;
        let entry_path = entry.path();

        if entry_path.is_file()
            && entry_path.extension().is_some_and(|ext| {
                let ext_str = ext.to_string_lossy();
                ext_str == "rs" || ext_str == "ts" || ext_str == "tsx"
            })
        {
            match process_file(&entry_path, base_path) {
                Ok(result) => results.push(result),
                Err(e) => eprintln!("Error processing file {}: {}", entry_path.display(), e),
            }
        } else if entry_path.is_dir() {
            match process_directory(&entry_path, base_path) {
                Ok(mut sub_results) => results.append(&mut sub_results),
                Err(e) => eprintln!(
                    "Error processing subdirectory {}: {}",
                    entry_path.display(),
                    e
                ),
            }
        }
    }
    Ok(results)
}

/// Process a path (file or directory) and return analysis results
pub fn process_path(path_str: &str) -> (Vec<AnalysisResult>, f64, usize) {
    let mut all_results: Vec<AnalysisResult> = Vec::new();
    let mut total_overall_complexity_score = 0.0;
    let mut total_files_analyzed = 0;

    let path = PathBuf::from(path_str);
    let base_path = Some(&path as &Path);

    if path.is_file() {
        match process_file(&path, None) {
            Ok(result) => {
                all_results.push(result.clone());
                total_overall_complexity_score += result.overall_complexity_score;
                total_files_analyzed += 1;
            }
            Err(e) => eprintln!("Error processing file {}: {}", path.display(), e),
        }
    } else if path.is_dir() {
        match process_directory(&path, base_path) {
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
        eprintln!(
            "Error: Path {} is neither a file nor a directory.",
            path.display()
        );
    }

    (
        all_results,
        total_overall_complexity_score,
        total_files_analyzed,
    )
}
