use petgraph::graph::{DiGraph, NodeIndex};
use std::collections::HashMap;
use tree_sitter::Node;

pub type Dependencies = (DiGraph<usize, usize>, HashMap<usize, NodeIndex>);

pub type DependencyHandler = fn(
    Node,
    &str,
    &mut DiGraph<usize, usize>,
    &mut HashMap<usize, NodeIndex>,
    &mut HashMap<String, usize>,
);

pub trait DependencyCollector {
    fn collect_dependencies_from_root(
        root: Node,
        content: &str,
        definitions: &HashMap<String, usize>,
    ) -> Result<Dependencies, String>;

    fn kind_handlers() -> HashMap<&'static str, DependencyHandler>;

    fn handle_identifier(
        n: Node,
        source_code: &str,
        graph: &mut DiGraph<usize, usize>,
        line_nodes: &mut HashMap<usize, NodeIndex>,
        definitions: &mut HashMap<String, usize>,
    );

    fn handle_call_expression(
        n: Node,
        source_code: &str,
        graph: &mut DiGraph<usize, usize>,
        line_nodes: &mut HashMap<usize, NodeIndex>,
        definitions: &mut HashMap<String, usize>,
    );

    fn handle_field_expression(
        n: Node,
        source_code: &str,
        graph: &mut DiGraph<usize, usize>,
        line_nodes: &mut HashMap<usize, NodeIndex>,
        definitions: &mut HashMap<String, usize>,
    );

    fn handle_struct_expression(
        n: Node,
        source_code: &str,
        graph: &mut DiGraph<usize, usize>,
        line_nodes: &mut HashMap<usize, NodeIndex>,
        definitions: &mut HashMap<String, usize>,
    );

    fn collect_dependencies_recursive(
        node: Node,
        source_code: &str,
        graph: &mut DiGraph<usize, usize>,
        line_nodes: &mut HashMap<usize, NodeIndex>,
        definitions: &mut HashMap<String, usize>,
        kind_handlers: &HashMap<&str, DependencyHandler>,
    ) {
        let mut stack: Vec<Node> = Vec::new();
        stack.push(node);

        while let Some(n) = stack.pop() {
            // Ensure nodes for the line range exist (shared behavior across languages)
            let start_line = n.start_position().row + 1;
            let end_line = n.end_position().row + 1;
            for line in start_line..=end_line {
                line_nodes
                    .entry(line)
                    .or_insert_with(|| graph.add_node(line));
            }

            if let Some(handler) = kind_handlers.get(n.kind()) {
                handler(n, source_code, graph, line_nodes, definitions);
            }

            let mut cursor = n.walk();
            let mut children: Vec<Node> = Vec::new();
            for child in n.children(&mut cursor) {
                children.push(child);
            }
            for child in children.into_iter().rev() {
                stack.push(child);
            }
        }
    }
}

/// Common helper to add a dependency edge between two line nodes.
/// - Creates an edge from `from_line` to `to_line` if both nodes exist and are different.
/// - Edge weight is the absolute distance between the two lines.
pub fn add_dependency(
    from_line: usize,
    to_line: usize,
    graph: &mut DiGraph<usize, usize>,
    line_nodes: &mut HashMap<usize, NodeIndex>,
) {
    let from_node_opt = line_nodes.get(&from_line);
    let to_node_opt = line_nodes.get(&to_line);

    if let (Some(&from_node), Some(&to_node)) = (from_node_opt, to_node_opt) {
        if from_node != to_node {
            let distance = from_line.abs_diff(to_line);
            graph.add_edge(from_node, to_node, distance);
        }
    }
}
