use lintric_core::models::{AnalysisResult, OverallAnalysisReport};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use tera::{Context, Tera};

// Helper functions (moved to top for scope)
fn sanitize_filename(path: &str) -> String {
    path.replace("/", "_")
        .replace("\\", "_") // Corrected: escape backslash
        .replace(":", "_")
        .replace(" ", "_")
        .replace(".", "_") // Remove dots to avoid issues with file extensions
        .replace("__", "_") // Replace double underscores that might result from multiple replacements
        .trim_matches('_')
        .to_string()
}

fn escape_html(s: &str) -> String {
    s.replace("&", "&amp;")
        .replace("<", "&lt;")
        .replace(">", "&gt;")
        .replace("\"", "&quot;") // Corrected: escape double quote
        .replace("'", "&#x27;")
}

fn get_complexity_class(score: f64) -> &'static str {
    if score > 10.0 {
        "high"
    } else if score > 5.0 {
        "medium"
    } else if score > 0.0 {
        "low"
    } else {
        "none"
    }
}

fn write_file(path: &Path, content: &str) -> Result<(), String> {
    let mut file = fs::File::create(path)
        .map_err(|e| format!("Error creating file {}: {}", path.display(), e))?;
    file.write_all(content.as_bytes())
        .map_err(|e| format!("Error writing to file {}: {}", path.display(), e))?;
    Ok(())
}

pub fn generate_html_report(report: &OverallAnalysisReport) {
    let output_dir = PathBuf::from(".lintric/output/html");
    if let Err(e) = fs::create_dir_all(&output_dir) {
        eprintln!(
            "Error creating output directory {}: {}",
            output_dir.display(),
            e
        );
        return;
    }

    // Initialize Tera with templates embedded in the binary
    let mut tera = Tera::default();
    tera.add_raw_template("index.html", include_str!("../templates/index.html"))
        .unwrap();
    tera.add_raw_template("file.html", include_str!("../templates/file.html"))
        .unwrap();

    let mut index_context = Context::new();
    index_context.insert("total_files_analyzed", &report.total_files_analyzed);
    index_context.insert("average_complexity_score", &report.average_complexity_score);

    let mut results_for_template: Vec<serde_json::Value> = Vec::new();

    for result in &report.results {
        let html_file_name = format!("{}.html", sanitize_filename(&result.file_path));

        // Prepare data for index template
        let mut file_data = serde_json::to_value(result).unwrap();
        file_data["html_file_name"] = serde_json::to_value(html_file_name.clone()).unwrap();
        results_for_template.push(file_data);

        // Generate individual file HTML
        if let Err(e) = generate_file_html(&output_dir, result, &tera) {
            eprintln!("Error generating HTML for file {}: {}", result.file_path, e);
        }
    }
    index_context.insert("results", &results_for_template);

    let index_html_content = match tera.render("index.html", &index_context) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Error rendering index.html: {e}");
            return;
        }
    };

    let index_file_path = output_dir.join("index.html");
    if let Err(e) = write_file(&index_file_path, &index_html_content) {
        eprintln!("Error writing index.html: {e}");
    } else {
        println!("HTML report generated at: {}", index_file_path.display());
    }
}

fn generate_file_html(
    output_dir: &Path,
    result: &AnalysisResult,
    tera: &Tera,
) -> Result<(), String> {
    let source_code = fs::read_to_string(&result.original_file_path).map_err(|e| {
        format!(
            "Error reading source file {}: {}",
            result.original_file_path, e
        )
    })?;

    let mut code_lines_for_template: Vec<serde_json::Value> = Vec::new();
    let lines: Vec<&str> = source_code.lines().collect();

    let file_extension = Path::new(&result.file_path)
        .extension()
        .and_then(|s| s.to_str())
        .unwrap_or("text"); // Default to "text" if no extension

    for (i, line_content) in lines.iter().enumerate() {
        let line_number = i + 1;
        let line_metrics = result
            .line_metrics
            .iter()
            .find(|m| m.line_number == line_number);

        let metrics_str = if let Some(metrics) = line_metrics {
            if metrics.total_dependencies == 0 {
                "".to_string()
            } else {
                let class = get_complexity_class(metrics.total_dependencies as f64);
                format!(
                    "<div class=\"metrics line-highlight-{}\">\
                    TD: <span class=\"metric-value\">{}</span>\
                    DDC: <span class=\"metric-value\">{:.2}</span>\
                    Depth: <span class=\"metric-value\">{}</span>\
                    TransD: <span class=\"metric-value\">{}</span>\
                </div>",
                    class,
                    metrics.total_dependencies,
                    metrics.dependency_distance_cost,
                    metrics.depth,
                    metrics.transitive_dependencies
                )
            }
        } else {
            "".to_string()
        };

        let mut line_data = Context::new();
        line_data.insert("line_number", &line_number);
        line_data.insert("code", &escape_html(line_content));
        line_data.insert("metrics_str", &metrics_str);
        code_lines_for_template.push(line_data.into_json());
    }

    let mut file_context = Context::new();
    file_context.insert("file_path", &result.file_path);
    file_context.insert("overall_complexity_score", &result.overall_complexity_score);
    file_context.insert("code_lines", &code_lines_for_template);
    file_context.insert("language_extension", &file_extension);

    let file_html_content = match tera.render("file.html", &file_context) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Error rendering file.html for {}: {}", result.file_path, e);
            return Err(format!("Error rendering file.html: {e}"));
        }
    };

    let html_file_name = format!("{}.html", sanitize_filename(&result.file_path));
    let file_path = output_dir.join(html_file_name);
    write_file(&file_path, &file_html_content)?;

    Ok(())
}
