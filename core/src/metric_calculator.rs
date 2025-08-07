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
        let line_number = graph[node_index];
        let total_dependencies = graph.neighbors_directed(node_index, petgraph::Direction::Outgoing).count();
        let dependency_distance_cost: usize = graph.edges_directed(node_index, petgraph::Direction::Outgoing).map(|edge| *edge.weight()).sum();
        
        let depth = dfs_longest_path(&graph, node_index, &mut HashMap::new());

        let mut dfs = Dfs::new(&graph, node_index);
        let mut transitive_dependencies: usize = 0;
        while let Some(_) = dfs.next(&graph) {
            transitive_dependencies += 1;
        }
        transitive_dependencies = transitive_dependencies.saturating_sub(1);

        let line_score = (total_dependencies as f64) 
                       + (dependency_distance_cost as f64 / 10.0) 
                       + (depth as f64) 
                       + (transitive_dependencies as f64 / 5.0);
        overall_complexity_score += line_score;

        all_line_metrics.push(LineMetrics {
            line_number,
            total_dependencies,
            dependency_distance_cost,
            depth,
            transitive_dependencies,
        });
    }

    Ok(AnalysisResult {
        file_path,
        line_metrics: all_line_metrics,
        overall_complexity_score,
    })
}

fn dfs_longest_path(graph: &DiGraph<usize, usize>, start_node: NodeIndex, memo: &mut HashMap<NodeIndex, usize>) -> usize {
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
