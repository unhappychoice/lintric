use petgraph::graph::{DiGraph, NodeIndex};
use std::collections::HashMap;
use tree_sitter::Node;

pub type Dependencies = (DiGraph<usize, usize>, HashMap<usize, NodeIndex>);

pub trait DependencyCollector: Send + Sync {
    #[allow(clippy::type_complexity)]
    fn collect_dependencies_from_root<'a>(
        &self,
        root: Node<'a>,
        content: &'a str,
        definitions: &HashMap<String, usize>,
    ) -> Result<Dependencies, String> {
        let mut graph: DiGraph<usize, usize> = DiGraph::new();
        let mut line_nodes: HashMap<usize, NodeIndex> = HashMap::new();

        for line_num in 1..=content.lines().count() {
            line_nodes
                .entry(line_num)
                .or_insert_with(|| graph.add_node(line_num));
        }

        let mut stack: Vec<Node<'a>> = Vec::new();
        stack.push(root);

        let mut defs = definitions.clone();

        while let Some(node) = stack.pop() {
            self.process_node(node, content, &mut graph, &mut line_nodes, &mut defs);

            let mut cursor = node.walk();
            let mut children: Vec<Node<'a>> = Vec::new();
            for child in node.children(&mut cursor) {
                children.push(child);
            }
            for child in children.into_iter().rev() {
                stack.push(child);
            }
        }

        Ok((graph, line_nodes))
    }

    fn process_node<'a>(
        &self,
        n: Node<'a>,
        source_code: &'a str,
        graph: &mut DiGraph<usize, usize>,
        line_nodes: &mut HashMap<usize, NodeIndex>,
        definitions: &mut HashMap<String, usize>,
    );

    fn handle_identifier<'a>(
        &self,
        n: Node<'a>,
        source_code: &'a str,
        graph: &mut DiGraph<usize, usize>,
        line_nodes: &mut HashMap<usize, NodeIndex>,
        definitions: &mut HashMap<String, usize>,
    );

    fn handle_call_expression<'a>(
        &self,
        n: Node<'a>,
        source_code: &'a str,
        graph: &mut DiGraph<usize, usize>,
        line_nodes: &mut HashMap<usize, NodeIndex>,
        definitions: &mut HashMap<String, usize>,
    );

    fn handle_field_expression<'a>(
        &self,
        n: Node<'a>,
        source_code: &'a str,
        graph: &mut DiGraph<usize, usize>,
        line_nodes: &mut HashMap<usize, NodeIndex>,
        definitions: &mut HashMap<String, usize>,
    );

    fn handle_struct_expression<'a>(
        &self,
        n: Node<'a>,
        source_code: &'a str,
        graph: &mut DiGraph<usize, usize>,
        line_nodes: &mut HashMap<usize, NodeIndex>,
        definitions: &mut HashMap<String, usize>,
    );
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
