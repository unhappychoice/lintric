use lintric_core::metric_calculator::calculate_metrics;
use lintric_core::models::{Dependency, DependencyType, IntermediateRepresentation};

#[test]
fn test_simple_dependency() {
    let dependencies = vec![Dependency {
        source_line: 2,
        target_line: 1,
        symbol: "a".to_string(),
        dependency_type: DependencyType::VariableUse,
        context: None,
    }];
    let ir = IntermediateRepresentation::new(
        "test.rs".to_string(),
        vec![],
        dependencies,
        vec![],
        "Rust".to_string(),
        2,
    );

    let code = "let a = 1;\nlet b = a;";
    let result = calculate_metrics(&ir, code).unwrap();

    assert_eq!(result.line_metrics.len(), 2);

    let line1_metrics = result
        .line_metrics
        .iter()
        .find(|m| m.line_number == 1)
        .unwrap();
    assert_eq!(line1_metrics.total_dependencies, 0);
    assert_eq!(line1_metrics.dependency_distance_cost, 0.0);
    assert_eq!(line1_metrics.depth, 0);
    assert_eq!(line1_metrics.transitive_dependencies, 0);

    let line2_metrics = result
        .line_metrics
        .iter()
        .find(|m| m.line_number == 2)
        .unwrap();
    assert_eq!(line2_metrics.total_dependencies, 1);
    assert_eq!(line2_metrics.dependency_distance_cost, 0.5);
    assert_eq!(line2_metrics.depth, 1);
    assert_eq!(line2_metrics.transitive_dependencies, 1);
}

#[test]
fn test_multiple_dependencies() {
    let dependencies = vec![
        Dependency {
            source_line: 3,
            target_line: 1,
            symbol: "a".to_string(),
            dependency_type: DependencyType::VariableUse,
            context: None,
        },
        Dependency {
            source_line: 3,
            target_line: 2,
            symbol: "b".to_string(),
            dependency_type: DependencyType::VariableUse,
            context: None,
        },
    ];
    let ir = IntermediateRepresentation::new(
        "test.rs".to_string(),
        vec![],
        dependencies,
        vec![],
        "Rust".to_string(),
        3,
    );

    let code = "let a = 1;\nlet b = 2;\nlet c = a + b;";
    let result = calculate_metrics(&ir, code).unwrap();

    let line3_metrics = result
        .line_metrics
        .iter()
        .find(|m| m.line_number == 3)
        .unwrap();
    assert_eq!(line3_metrics.total_dependencies, 2);
}

#[test]
fn test_transitive_dependencies() {
    let dependencies = vec![
        Dependency {
            source_line: 2,
            target_line: 1,
            symbol: "a".to_string(),
            dependency_type: DependencyType::VariableUse,
            context: None,
        },
        Dependency {
            source_line: 3,
            target_line: 2,
            symbol: "b".to_string(),
            dependency_type: DependencyType::VariableUse,
            context: None,
        },
    ];
    let ir = IntermediateRepresentation::new(
        "test.rs".to_string(),
        vec![],
        dependencies,
        vec![],
        "Rust".to_string(),
        3,
    );

    let code = "let a = 1;\nlet b = a;\nlet c = b;";
    let result = calculate_metrics(&ir, code).unwrap();

    let line3_metrics = result
        .line_metrics
        .iter()
        .find(|m| m.line_number == 3)
        .unwrap();
    assert_eq!(line3_metrics.depth, 2);
    assert_eq!(line3_metrics.transitive_dependencies, 2);
}

#[test]
fn test_cyclic_dependencies() {
    let dependencies = vec![
        Dependency {
            source_line: 1,
            target_line: 2,
            symbol: "b".to_string(),
            dependency_type: DependencyType::FunctionCall,
            context: None,
        },
        Dependency {
            source_line: 2,
            target_line: 1,
            symbol: "a".to_string(),
            dependency_type: DependencyType::FunctionCall,
            context: None,
        },
    ];
    let ir = IntermediateRepresentation::new(
        "test.rs".to_string(),
        vec![],
        dependencies,
        vec![],
        "Rust".to_string(),
        2,
    );

    let code = "fn a() { b() }\nfn b() { a() }";
    let result = calculate_metrics(&ir, code).unwrap();

    let line1_metrics = result
        .line_metrics
        .iter()
        .find(|m| m.line_number == 1)
        .unwrap();
    assert_eq!(line1_metrics.depth, 2);
}

#[test]
fn test_no_dependencies() {
    let ir = IntermediateRepresentation::new(
        "test.rs".to_string(),
        vec![],
        vec![],
        vec![],
        "Rust".to_string(),
        2,
    );

    let code = "let a = 1;\nlet b = 2;";
    let result = calculate_metrics(&ir, code).unwrap();

    let line1_metrics = result
        .line_metrics
        .iter()
        .find(|m| m.line_number == 1)
        .unwrap();
    assert_eq!(line1_metrics.total_dependencies, 0);

    let line2_metrics = result
        .line_metrics
        .iter()
        .find(|m| m.line_number == 2)
        .unwrap();
    assert_eq!(line2_metrics.total_dependencies, 0);
}
