use lintric_core::metric_calculator::calculate_metrics;
use petgraph::graph::DiGraph;

#[test]
fn test_simple_dependency() {
    let mut graph = DiGraph::<usize, usize>::new();
    let node1 = graph.add_node(1);
    let node2 = graph.add_node(2);
    graph.add_edge(node2, node1, 1);

    let code = "let a = 1;\nlet b = a;";
    let result = calculate_metrics(graph, code, "test.rs".to_string()).unwrap();

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
    let mut graph = DiGraph::<usize, usize>::new();
    let node1 = graph.add_node(1);
    let node2 = graph.add_node(2);
    let node3 = graph.add_node(3);
    graph.add_edge(node3, node1, 2);
    graph.add_edge(node3, node2, 1);

    let code = "let a = 1;\nlet b = 2;\nlet c = a + b;";
    let result = calculate_metrics(graph, code, "test.rs".to_string()).unwrap();

    let line3_metrics = result
        .line_metrics
        .iter()
        .find(|m| m.line_number == 3)
        .unwrap();
    assert_eq!(line3_metrics.total_dependencies, 2);
}

#[test]
fn test_transitive_dependencies() {
    let mut graph = DiGraph::<usize, usize>::new();
    let node1 = graph.add_node(1);
    let node2 = graph.add_node(2);
    let node3 = graph.add_node(3);
    graph.add_edge(node2, node1, 1);
    graph.add_edge(node3, node2, 1);

    let code = "let a = 1;\nlet b = a;\nlet c = b;";
    let result = calculate_metrics(graph, code, "test.rs".to_string()).unwrap();

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
    let mut graph = DiGraph::<usize, usize>::new();
    let node1 = graph.add_node(1);
    let node2 = graph.add_node(2);
    graph.add_edge(node1, node2, 1);
    graph.add_edge(node2, node1, 1);

    let code = "fn a() { b() }\nfn b() { a() }";
    let result = calculate_metrics(graph, code, "test.rs".to_string()).unwrap();

    let line1_metrics = result
        .line_metrics
        .iter()
        .find(|m| m.line_number == 1)
        .unwrap();
    assert_eq!(line1_metrics.depth, 2);
}

#[test]
fn test_no_dependencies() {
    let mut graph = DiGraph::<usize, usize>::new();
    graph.add_node(1);
    graph.add_node(2);

    let code = "let a = 1;\nlet b = 2;".trim();
    let result = calculate_metrics(graph, code, "test.rs".to_string()).unwrap();

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
