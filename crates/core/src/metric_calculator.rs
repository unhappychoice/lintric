use petgraph::graph::{DiGraph, NodeIndex};
use petgraph::visit::Dfs;
use std::collections::{HashMap, HashSet};

use crate::models::{AnalysisResult, IntermediateRepresentation, LineMetrics};

pub fn calculate_metrics(
    ir: &IntermediateRepresentation,
    content: &str,
) -> Result<AnalysisResult, String> {
    let graph = ir_to_graph(ir);

    let mut overall_complexity_score = 0.0;
    let mut all_line_metrics: Vec<LineMetrics> = Vec::new();

    for node_index in graph.node_indices() {
        let line_metrics = calculate_line_metrics(&graph, node_index, content);

        overall_complexity_score += (line_metrics.total_dependencies as f64)
            + (line_metrics.dependency_distance_cost as f64 / 10.0)
            + (line_metrics.depth as f64)
            + (line_metrics.transitive_dependencies as f64 / 5.0);

        all_line_metrics.push(line_metrics);
    }

    Ok(AnalysisResult {
        file_path: ir.file_path.clone(),
        line_metrics: all_line_metrics,
        overall_complexity_score,
    })
}

fn calculate_line_metrics(
    graph: &DiGraph<usize, usize>,
    node_index: NodeIndex,
    content: &str,
) -> LineMetrics {
    let line_number = graph[node_index];

    let total_dependencies = total_dependencies(graph, node_index);
    let dependency_distance_cost = dependency_distance_cost(graph, node_index, content);
    let depth = dfs_longest_path(graph, node_index, &mut HashMap::new(), &mut HashSet::new());
    let transitive_dependencies = transitive_dependencies(graph, node_index);
    let dependent_lines = get_dependent_lines(graph, node_index);

    LineMetrics {
        line_number,
        total_dependencies,
        dependency_distance_cost,
        depth,
        transitive_dependencies,
        dependent_lines,
    }
}

fn get_dependent_lines(graph: &DiGraph<usize, usize>, node_index: NodeIndex) -> Vec<usize> {
    graph
        .neighbors_directed(node_index, petgraph::Direction::Outgoing)
        .map(|neighbor_node_index| graph[neighbor_node_index])
        .collect()
}

fn total_dependencies(graph: &DiGraph<usize, usize>, node_index: NodeIndex) -> usize {
    graph
        .neighbors_directed(node_index, petgraph::Direction::Outgoing)
        .count()
}

fn dependency_distance_cost(
    graph: &DiGraph<usize, usize>,
    node_index: NodeIndex,
    content: &str,
) -> f64 {
    let line_count = content.lines().count();
    graph
        .edges_directed(node_index, petgraph::Direction::Outgoing)
        .map(|edge| (*edge.weight() as f64) / (line_count as f64))
        .sum()
}

fn transitive_dependencies(graph: &DiGraph<usize, usize>, node_index: NodeIndex) -> usize {
    let mut dfs = Dfs::new(&graph, node_index);
    let mut transitive_dependencies: usize = 0;

    while dfs.next(&graph).is_some() {
        transitive_dependencies += 1;
    }
    transitive_dependencies = transitive_dependencies.saturating_sub(1);

    transitive_dependencies
}

fn dfs_longest_path(
    graph: &DiGraph<usize, usize>,
    start_node: NodeIndex,
    memo: &mut HashMap<NodeIndex, usize>,
    on_path: &mut HashSet<NodeIndex>,
) -> usize {
    if let Some(&cached_depth) = memo.get(&start_node) {
        return cached_depth;
    }

    on_path.clear();
    let mut processed: HashSet<NodeIndex> = HashSet::new();
    let mut neighbor_idx: HashMap<NodeIndex, usize> = HashMap::new();
    let mut neighbors_map: HashMap<NodeIndex, Vec<NodeIndex>> = HashMap::new();
    let mut stack: Vec<NodeIndex> = Vec::new();

    stack.push(start_node);

    while let Some(&node) = stack.last() {
        if processed.contains(&node) {
            stack.pop();
            continue;
        }

        if let std::collections::hash_map::Entry::Vacant(e) = neighbors_map.entry(node) {
            let ns: Vec<NodeIndex> = graph.neighbors(node).collect();
            e.insert(ns);
            neighbor_idx.insert(node, 0);
            on_path.insert(node);
        }

        let idx = neighbor_idx.get_mut(&node).unwrap();
        let neighbors = neighbors_map.get(&node).unwrap();

        if *idx < neighbors.len() {
            let nb = neighbors[*idx];
            *idx += 1;

            if processed.contains(&nb) {
                continue;
            }
            if on_path.contains(&nb) {
                continue;
            }
            if memo.contains_key(&nb) {
                continue;
            }

            stack.push(nb);
        } else {
            let mut max_depth = 0usize;
            for &nb in neighbors.iter() {
                let dnb = *memo.get(&nb).unwrap_or(&0);
                let cand = 1 + dnb;
                if cand > max_depth {
                    max_depth = cand;
                }
            }
            memo.insert(node, max_depth);
            processed.insert(node);
            on_path.remove(&node);
            stack.pop();
        }
    }

    *memo.get(&start_node).unwrap_or(&0)
}

fn ir_to_graph(ir: &IntermediateRepresentation) -> DiGraph<usize, usize> {
    let mut graph: DiGraph<usize, usize> = DiGraph::new();
    let mut line_nodes: HashMap<usize, NodeIndex> = HashMap::new();

    for i in 1..=ir.analysis_metadata.total_lines {
        line_nodes.insert(i, graph.add_node(i));
    }

    // Add edges for dependencies
    for dep in &ir.dependencies {
        let source_node = line_nodes[&dep.source_line];
        let target_node = line_nodes[&dep.target_line];
        let distance = dep.source_line.abs_diff(dep.target_line);
        graph.add_edge(source_node, target_node, distance);
    }

    graph
}
