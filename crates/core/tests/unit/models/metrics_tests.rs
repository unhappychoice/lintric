use lintric_core::models::{AnalysisResult, LineMetrics, OverallAnalysisReport};

#[test]
fn test_line_metrics_creation() {
    let metrics = LineMetrics {
        line_number: 10,
        total_dependencies: 5,
        dependency_distance_cost: 12.5,
        depth: 2,
        transitive_dependencies: 8,
        dependent_lines: vec![1, 3, 7, 15],
    };

    assert_eq!(metrics.line_number, 10);
    assert_eq!(metrics.total_dependencies, 5);
    assert_eq!(metrics.dependency_distance_cost, 12.5);
    assert_eq!(metrics.depth, 2);
    assert_eq!(metrics.transitive_dependencies, 8);
    assert_eq!(metrics.dependent_lines, vec![1, 3, 7, 15]);
}

#[test]
fn test_analysis_result_creation() {
    let line_metrics = vec![
        LineMetrics {
            line_number: 1,
            total_dependencies: 2,
            dependency_distance_cost: 5.0,
            depth: 1,
            transitive_dependencies: 3,
            dependent_lines: vec![5, 10],
        },
        LineMetrics {
            line_number: 5,
            total_dependencies: 1,
            dependency_distance_cost: 2.5,
            depth: 0,
            transitive_dependencies: 1,
            dependent_lines: vec![],
        },
    ];

    let result = AnalysisResult {
        file_path: "src/main.rs".to_string(),
        line_metrics: line_metrics.clone(),
        overall_complexity_score: 7.5,
    };

    assert_eq!(result.file_path, "src/main.rs");
    assert_eq!(result.line_metrics.len(), 2);
    assert_eq!(result.overall_complexity_score, 7.5);
}

#[test]
fn test_overall_analysis_report_creation() {
    let analysis_result = AnalysisResult {
        file_path: "test.rs".to_string(),
        line_metrics: vec![],
        overall_complexity_score: 10.0,
    };

    let report = OverallAnalysisReport {
        results: vec![analysis_result],
        total_files_analyzed: 1,
        total_overall_complexity_score: 10.0,
        average_complexity_score: 10.0,
    };

    assert_eq!(report.results.len(), 1);
    assert_eq!(report.total_files_analyzed, 1);
    assert_eq!(report.total_overall_complexity_score, 10.0);
    assert_eq!(report.average_complexity_score, 10.0);
}

#[test]
fn test_line_metrics_clone() {
    let original = LineMetrics {
        line_number: 15,
        total_dependencies: 3,
        dependency_distance_cost: 8.0,
        depth: 2,
        transitive_dependencies: 5,
        dependent_lines: vec![2, 8, 12],
    };

    let cloned = original.clone();

    // Test that clone works by checking individual fields
    assert_eq!(original.line_number, cloned.line_number);
    assert_eq!(original.total_dependencies, cloned.total_dependencies);
    assert_eq!(
        original.dependency_distance_cost,
        cloned.dependency_distance_cost
    );
    assert_eq!(original.depth, cloned.depth);
    assert_eq!(
        original.transitive_dependencies,
        cloned.transitive_dependencies
    );
    assert_eq!(original.dependent_lines.len(), cloned.dependent_lines.len());
}

#[test]
fn test_analysis_result_clone() {
    let original = AnalysisResult {
        file_path: "src/lib.rs".to_string(),
        line_metrics: vec![],
        overall_complexity_score: 5.5,
    };

    let cloned = original.clone();

    assert_eq!(original.file_path, cloned.file_path);
    assert_eq!(original.line_metrics.len(), cloned.line_metrics.len());
    assert_eq!(
        original.overall_complexity_score,
        cloned.overall_complexity_score
    );
}

#[test]
fn test_metrics_debug() {
    let metrics = LineMetrics {
        line_number: 20,
        total_dependencies: 1,
        dependency_distance_cost: 3.0,
        depth: 0,
        transitive_dependencies: 1,
        dependent_lines: vec![25],
    };

    let debug_str = format!("{:?}", metrics);
    assert!(debug_str.contains("20"));
    assert!(debug_str.contains("3.0"));
}
