use lintric_core::{analyze_code, AnalysisResult, LineMetrics};
use crate::assert_analysis_results_match;

#[test]
fn test_analyze_code_basic() {
    let code = "
let a = 1;
let b = a + 1;
".trim();
    let file_path = "test.rs".to_string();
    let result = analyze_code(code, file_path.clone()).unwrap();

    let expected_result = AnalysisResult {
        file_path: file_path,
        line_metrics: vec![
            LineMetrics { line_number: 1, total_dependencies: 0, dependency_distance_cost: 0.0, depth: 0, transitive_dependencies: 0 },
            LineMetrics { line_number: 2, total_dependencies: 1, dependency_distance_cost: 0.5, depth: 1, transitive_dependencies: 1 },
        ],
        overall_complexity_score: 2.25,
    };

    assert_analysis_results_match!(result, expected_result);
}

#[test]
fn test_rust_function_call_dependency() {
    let code = "
fn add(a: i32, b: i32) -> i32 {
    a + b
}
fn main() {
    let x = add(1, 2);
}
".trim();
    let file_path = "test.rs".to_string();
    let result = analyze_code(code, file_path.clone()).unwrap();

    let expected_result = AnalysisResult {
        file_path: file_path,
        line_metrics: vec![
            LineMetrics { line_number: 1, total_dependencies: 0, dependency_distance_cost: 0.0, depth: 0, transitive_dependencies: 0 },
            LineMetrics { line_number: 2, total_dependencies: 0, dependency_distance_cost: 0.0, depth: 0, transitive_dependencies: 0 },
            LineMetrics { line_number: 3, total_dependencies: 0, dependency_distance_cost: 0.0, depth: 0, transitive_dependencies: 0 },
            LineMetrics { line_number: 4, total_dependencies: 0, dependency_distance_cost: 0.0, depth: 0, transitive_dependencies: 0 },
            LineMetrics { line_number: 5, total_dependencies: 2, dependency_distance_cost: 1.3333333333333333, depth: 1, transitive_dependencies: 1 },
            LineMetrics { line_number: 6, total_dependencies: 0, dependency_distance_cost: 0.0, depth: 0, transitive_dependencies: 0 },
        ],
        overall_complexity_score: 3.3333333333333335,
    };

    assert_analysis_results_match!(result, expected_result);
}

#[test] fn test_rust_struct_field_access_dependency() {
    let code = r#"
struct Point { x: i32, y: i32 }
fn main() {
    let p = Point { x: 1, y: 2 };
    let val = p.x;
}
"#.trim();
    let file_path = "test.rs".to_string();
    let result = analyze_code(code, file_path.clone()).unwrap();

    let expected_result = AnalysisResult {
        file_path: file_path,
        line_metrics: vec![
            LineMetrics { line_number: 1, total_dependencies: 0, dependency_distance_cost: 0.0, depth: 0, transitive_dependencies: 0 },
            LineMetrics { line_number: 2, total_dependencies: 0, dependency_distance_cost: 0.0, depth: 0, transitive_dependencies: 0 },
            LineMetrics { line_number: 3, total_dependencies: 0, dependency_distance_cost: 0.0, depth: 0, transitive_dependencies: 0 },
            LineMetrics { line_number: 4, total_dependencies: 1, dependency_distance_cost: 0.2, depth: 1, transitive_dependencies: 1, },
            LineMetrics { line_number: 5, total_dependencies: 0, dependency_distance_cost: 0.0, depth: 0, transitive_dependencies: 0 },
        ],
        overall_complexity_score: 2.22,
    };

    assert_analysis_results_match!(result, expected_result);
}
