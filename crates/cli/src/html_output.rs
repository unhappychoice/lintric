use crate::display::format_file_path_for_display;
use crate::logger::Logger;
use lintric_core::models::{AnalysisResult, LineMetrics, OverallAnalysisReport};
use std::collections::hash_map::DefaultHasher;
use std::fs;
use std::hash::{Hash, Hasher};
use std::io::Write;
use std::path::{Path, PathBuf};
use syntect::highlighting::ThemeSet;
use syntect::html::css_for_theme_with_class_style;
use syntect::parsing::SyntaxSet;
use tera::{Context, Tera};

pub fn generate_html_report(
    report: &OverallAnalysisReport,
    base_paths: &[String],
    logger: &dyn Logger,
) {
    let output_dir = PathBuf::from(".lintric/output/html");
    if let Err(e) = fs::create_dir_all(&output_dir) {
        logger.error(&format!(
            "Error creating output directory {}: {}",
            output_dir.display(),
            e
        ));
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
        let display_path = format_file_path_for_display(&result.file_path, base_paths);
        let html_file_name = report_file_name(&result.file_path, &display_path);

        // Prepare data for index template
        let mut file_data = serde_json::to_value(result).unwrap();
        file_data["html_file_name"] = serde_json::to_value(&html_file_name).unwrap();
        file_data["file_path"] = serde_json::to_value(&display_path).unwrap();
        results_for_template.push(file_data);

        // Generate individual file HTML
        if let Err(e) = generate_file_html(
            &output_dir,
            result,
            &html_file_name,
            &display_path,
            &tera,
            &ps,
            theme,
        ) {
            logger.error(&format!(
                "Error generating HTML for file {}: {}",
                result.file_path, e
            ));
        }
    }
    index_context.insert("results", &results_for_template);

    let index_html_content = match tera.render("index.html", &index_context) {
        Ok(s) => s,
        Err(e) => {
            logger.error(&format!("Error rendering index.html: {e}"));
            return;
        }
    };

    let index_file_path = output_dir.join("index.html");
    if let Err(e) = write_file(&index_file_path, &index_html_content) {
        logger.error(&format!("Error writing index.html: {e}"));
    } else {
        logger.info(&format!(
            "HTML report generated at: {}",
            index_file_path.display()
        ));
    }
}

/// Build the report file name for one source file.
///
/// The readable half comes from the path as displayed, so reports are easy to
/// find. Slugging alone collides — `src/a.rs` and `src_a.rs` both read as
/// `src_a_rs` — so a digest of the full source path is appended and each source
/// keeps its own report.
fn report_file_name(source_path: &str, display_path: &str) -> String {
    format!("{}-{:08x}.html", slug(display_path), digest(source_path))
}

fn slug(path: &str) -> String {
    path.chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '_' })
        .collect::<String>()
        .split('_')
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join("_")
}

fn digest(path: &str) -> u32 {
    let mut hasher = DefaultHasher::new();
    path.hash(&mut hasher);
    hasher.finish() as u32
}

/// Bucket a line by how many lines it depends on.
///
/// The thresholds are dependency counts, not complexity scores.
fn dependency_class(total_dependencies: usize) -> &'static str {
    match total_dependencies {
        0 => "none",
        1..=5 => "low",
        6..=10 => "medium",
        _ => "high",
    }
}

/// The class lives on the wrapper so the stylesheet can colour the
/// `.metric-value` spans nested inside it.
fn metrics_html(metrics: &LineMetrics) -> String {
    format!(
        "<div class=\"metrics line-highlight-{}\">\
            TD: <span class=\"metric-value\">{}</span>\
            DDC: <span class=\"metric-value\">{:.2}</span>\
            Depth: <span class=\"metric-value\">{}</span>\
            TransD: <span class=\"metric-value\">{}</span>\
        </div>",
        dependency_class(metrics.total_dependencies),
        metrics.total_dependencies,
        metrics.dependency_distance_cost,
        metrics.depth,
        metrics.transitive_dependencies
    )
}

fn write_file(path: &Path, content: &str) -> Result<(), String> {
    let mut file = fs::File::create(path)
        .map_err(|e| format!("Error creating file {}: {}", path.display(), e))?;
    file.write_all(content.as_bytes())
        .map_err(|e| format!("Error writing to file {}: {}", path.display(), e))?;
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn generate_file_html(
    output_dir: &Path,
    result: &AnalysisResult,
    html_file_name: &str,
    display_path: &str,
    tera: &Tera,
    ps: &SyntaxSet,
    theme: &syntect::highlighting::Theme,
) -> Result<(), String> {
    let source_code = fs::read_to_string(&result.file_path)
        .map_err(|e| format!("Error reading source file {}: {}", result.file_path, e))?;

    let file_extension = Path::new(&result.file_path)
        .extension()
        .and_then(|s| s.to_str())
        .unwrap_or("txt");

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

        let metrics_str = line_metrics
            .filter(|metrics| metrics.total_dependencies > 0)
            .map(metrics_html)
            .unwrap_or_default();

        let highlighted_code_line = highlighted_lines
            .get(i)
            .unwrap_or(&String::new())
            .to_string();
        code_lines_for_template.push(serde_json::json!({
            "line_number": line_number,
            "code": highlighted_code_line,
            "metrics_str": metrics_str,
            "dependent_lines": line_metrics.map_or(vec![], |m| m.dependent_lines.clone()),
        }));
    }

    let mut file_context = Context::new();
    file_context.insert("file_path", display_path);
    file_context.insert("overall_complexity_score", &result.overall_complexity_score);
    file_context.insert("code_lines", &code_lines_for_template);
    file_context.insert("language_extension", &file_extension);
    file_context.insert("highlight_css", &css);

    let file_html_content = match tera.render("file.html", &file_context) {
        Ok(s) => s,
        Err(e) => {
            return Err(format!(
                "Error rendering file.html for {}: {}",
                result.file_path, e
            ));
        }
    };

    write_file(&output_dir.join(html_file_name), &file_html_content)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn report_file_name_distinguishes_paths_that_slug_identically() {
        assert_ne!(
            report_file_name("src/a.rs", "src/a.rs"),
            report_file_name("src_a.rs", "src_a.rs")
        );
    }

    #[test]
    fn report_file_name_collapses_repeated_separators() {
        assert!(!report_file_name("a///b.rs", "a///b.rs").contains("__"));
    }

    #[test]
    fn report_file_name_is_stable_for_the_same_path() {
        assert_eq!(
            report_file_name("src/a.rs", "src/a.rs"),
            report_file_name("src/a.rs", "src/a.rs")
        );
    }

    #[test]
    fn report_file_name_reads_as_the_displayed_path() {
        assert!(report_file_name("/abs/base/src/a.rs", "src/a.rs").starts_with("src_a_rs-"));
    }

    #[test]
    fn dependency_class_buckets_by_count() {
        assert_eq!(dependency_class(0), "none");
        assert_eq!(dependency_class(5), "low");
        assert_eq!(dependency_class(6), "medium");
        assert_eq!(dependency_class(11), "high");
    }

    #[test]
    fn metrics_html_puts_the_class_on_the_wrapper() {
        let html = metrics_html(&LineMetrics {
            line_number: 1,
            total_dependencies: 12,
            dependency_distance_cost: 1.5,
            depth: 2,
            transitive_dependencies: 3,
            dependent_lines: vec![],
        });

        assert!(
            html.contains("class=\"metrics line-highlight-high\""),
            "{html}"
        );
        assert!(
            html.contains("<span class=\"metric-value\">12</span>"),
            "{html}"
        );
    }

    #[test]
    fn stylesheet_targets_metric_values_nested_in_the_wrapper() {
        let template = include_str!("../templates/file.html");
        for level in ["low", "medium", "high"] {
            assert!(
                template.contains(&format!(".line-highlight-{level} .metric-value")),
                "file.html must style .metric-value nested inside .line-highlight-{level}"
            );
        }
    }
}
