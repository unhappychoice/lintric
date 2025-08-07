use petgraph::graph::{DiGraph, NodeIndex};
use petgraph::visit::Dfs;
use std::collections::HashMap;

use crate::models::{AnalysisResult, LineMetrics};

pub fn calculate_metrics(
    graph: DiGraph<usize, usize>,
    file_path: String,
) -> Result<AnalysisResult, String> {
    let mut overall_complexity_score = 0.0;
    let mut all_line_metrics: Vec<LineMetrics> = Vec::new();

    for node_index in graph.node_indices() {
        let line_metrics = calculate_line_metrics(&graph, node_index);

        overall_complexity_score += (line_metrics.total_dependencies as f64)
            + (line_metrics.dependency_distance_cost as f64 / 10.0)
            + (line_metrics.depth as f64)
            + (line_metrics.transitive_dependencies as f64 / 5.0);

        all_line_metrics.push(line_metrics);
    }

    Ok(AnalysisResult {
        file_path,
        line_metrics: all_line_metrics,
        overall_complexity_score,
    })
}

fn calculate_line_metrics(graph: &DiGraph<usize, usize>, node_index: NodeIndex) -> LineMetrics {
    let line_number = graph[node_index];

    let total_dependencies = total_dependencies(&graph, node_index);
    let dependency_distance_cost: usize = dependency_distance_cost(&graph, node_index);
    let depth = dfs_longest_path(&graph, node_index, &mut HashMap::new());
    let transitive_dependencies: usize = transitive_dependencies(&graph, node_index);

    LineMetrics {
        line_number,
        total_dependencies,
        dependency_distance_cost,
        depth,
        transitive_dependencies,
    }
}

fn total_dependencies(graph: &DiGraph<usize, usize>, node_index: NodeIndex) -> usize {
    graph.neighbors_directed(node_index, petgraph::Direction::Outgoing).count()
}

fn dependency_distance_cost(graph: &DiGraph<usize, usize>, node_index: NodeIndex) -> usize {
    graph.edges_directed(node_index, petgraph::Direction::Outgoing).map(|edge| *edge.weight()).sum()
}

fn transitive_dependencies(graph: &DiGraph<usize, usize>, node_index: NodeIndex) -> usize {
    let mut dfs = Dfs::new(&graph, node_index);
    let mut transitive_dependencies: usize = 0;

    while let Some(_) = dfs.next(&graph) {
        transitive_dependencies += 1;
    }
    transitive_dependencies = transitive_dependencies.saturating_sub(1);

    transitive_dependencies
}

fn dfs_longest_path(
    graph: &DiGraph<usize, usize>,
    start_node: NodeIndex,
    memo: &mut HashMap<NodeIndex, usize>
) -> usize {
    if let Some(&cached_depth) = memo.get(&start_node) {
        return cached_depth;
    }

    let mut max_depth = 0;
    for neighbor in graph.neighbors(start_node) {
        max_depth = max_depth.max(1 + dfs_longest_path(graph, neighbor, memo));
    }

    memo.insert(start_node, max_depth);
    max_depth
}
