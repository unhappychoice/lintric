use lintric_core::models::{AnalysisResult, OverallAnalysisReport};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use syntect::highlighting::ThemeSet;
use syntect::html::css_for_theme_with_class_style;
use syntect::parsing::SyntaxSet;
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
    let ps = SyntaxSet::load_defaults_newlines();
    let ts = ThemeSet::load_defaults();
    let theme = &ts.themes["base16-ocean.dark"]; // 好みのテーマを選択
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
        if let Err(e) = generate_file_html(&output_dir, result, &tera, &ps, theme) {
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
    ps: &SyntaxSet,
    theme: &syntect::highlighting::Theme,
) -> Result<(), String> {
    let source_code = fs::read_to_string(&result.original_file_path).map_err(|e| {
        format!(
            "Error reading source file {}: {}",
            result.original_file_path, e
        )
    })?;

    let file_extension = Path::new(&result.file_path)
        .extension()
        .and_then(|s| s.to_str())
        .unwrap_or("txt"); // Default to "txt" if no extension

    let syntax = ps
        .find_syntax_by_extension(file_extension)
        .unwrap_or_else(|| ps.find_syntax_plain_text());

    use syntect::easy::HighlightLines;
    use syntect::util::LinesWithEndings;

    let mut h = HighlightLines::new(syntax, theme);
    let mut highlighted_lines: Vec<String> = Vec::new();

    for line in LinesWithEndings::from(&source_code) {
        let ranges = h
            .highlight_line(line, ps)
            .map_err(|e| format!("Error highlighting line: {e}"))?;
        let html = syntect::html::styled_line_to_highlighted_html(
            &ranges[..],
            syntect::html::IncludeBackground::No,
        )
        .map_err(|e| format!("Error converting to HTML: {e}"))?;
        highlighted_lines.push(html);
    }

    let css = css_for_theme_with_class_style(theme, syntect::html::ClassStyle::Spaced)
        .map_err(|e| format!("Error generating CSS: {e}"))?;

    let mut code_lines_for_template: Vec<serde_json::Value> = Vec::new();
    let lines: Vec<&str> = source_code.lines().collect();

    for (i, _line_content) in lines.iter().enumerate() {
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
        // highlighted_linesから対応する行のHTMLを取得
        let highlighted_code_line = highlighted_lines
            .get(i)
            .unwrap_or(&String::new())
            .to_string();
        line_data.insert("code", &highlighted_code_line);
        line_data.insert("metrics_str", &metrics_str);
        // Add dependent_lines to line_data
        line_data.insert(
            "dependent_lines",
            &line_metrics.map_or(vec![], |m| m.dependent_lines.clone()),
        );
        code_lines_for_template.push(line_data.into_json());
    }

    let mut file_context = Context::new();
    file_context.insert("file_path", &result.file_path);
    file_context.insert("overall_complexity_score", &result.overall_complexity_score);
    file_context.insert("code_lines", &code_lines_for_template);
    file_context.insert("language_extension", &file_extension);
    file_context.insert("highlight_css", &css);

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
