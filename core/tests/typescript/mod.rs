use lintric_core::{analyze_code, AnalysisResult, LineMetrics};
use crate::assert_analysis_results_match;

#[test]
fn test_analyze_code_typescript() {
    let code = "
const x = 1;
let y = x + 2;
function foo() {
    return y;
}
".trim();
    let file_path = "test.ts".to_string();
    let result = analyze_code(code, file_path.clone()).unwrap();

    let expected_result = AnalysisResult {
        file_path: file_path,
        line_metrics: vec![
            LineMetrics { line_number: 1, total_dependencies: 0, dependency_distance_cost: 0.0, depth: 0, transitive_dependencies: 0 },
            LineMetrics { line_number: 2, total_dependencies: 1, dependency_distance_cost: 0.2, depth: 1, transitive_dependencies: 1 },
            LineMetrics { line_number: 3, total_dependencies: 0, dependency_distance_cost: 0.0, depth: 0, transitive_dependencies: 0 },
            LineMetrics { line_number: 4, total_dependencies: 1, dependency_distance_cost: 0.4, depth: 2, transitive_dependencies: 2 },
            LineMetrics { line_number: 5, total_dependencies: 0, dependency_distance_cost: 0.0, depth: 0, transitive_dependencies: 0 },
        ],
        overall_complexity_score: 5.66,
    };

    assert_analysis_results_match!(result, expected_result);
}

#[test] fn test_typescript_class_method_dependency() {
    let code = r#"
class MyClass {
    constructor(public value: number){}
    greet() { console.log(this.value); }
}
let instance = new MyClass(10);
instance.greet();
"#.trim();
    let file_path = "test.ts".to_string();
    let result = analyze_code(code, file_path.clone()).unwrap();

    let expected_result = AnalysisResult {
        file_path: file_path,
        line_metrics: vec![
            LineMetrics { line_number: 1, total_dependencies: 0, dependency_distance_cost: 0.0, depth: 0, transitive_dependencies: 0 },
            LineMetrics { line_number: 2, total_dependencies: 0, dependency_distance_cost: 0.0, depth: 0, transitive_dependencies: 0 },
            LineMetrics { line_number: 3, total_dependencies: 0, dependency_distance_cost: 0.0, depth: 0, transitive_dependencies: 0 },
            LineMetrics { line_number: 4, total_dependencies: 0, dependency_distance_cost: 0.0, depth: 0, transitive_dependencies: 0 },
            LineMetrics { line_number: 5, total_dependencies: 1, dependency_distance_cost: 0.6666666666666666, depth: 1, transitive_dependencies: 1 },
            LineMetrics { line_number: 6, total_dependencies: 2, dependency_distance_cost: 0.3333333333333333, depth: 2, transitive_dependencies: 2 },
        ],
        overall_complexity_score: 6.7,
    };

    assert_analysis_results_match!(result, expected_result);
}

#[test]
fn test_typescript_import_dependency() {
    let code = r#"
import { someFunction } from './module';
const result = someFunction();
"#.trim();
    let file_path = "test.ts".to_string();
    let result = analyze_code(code, file_path.clone()).unwrap();

    let expected_result = AnalysisResult {
        file_path: file_path,
        line_metrics: vec![
            LineMetrics { line_number: 1, total_dependencies: 0, dependency_distance_cost: 0.0, depth: 0, transitive_dependencies: 0 },
            LineMetrics { line_number: 2, total_dependencies: 2, dependency_distance_cost: 1.0, depth: 1, transitive_dependencies: 1 },
        ],
        overall_complexity_score: 3.3000000000000003,
    };

    assert_analysis_results_match!(result, expected_result);
}
