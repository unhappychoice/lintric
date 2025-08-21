use crate::models::Usage;
use tree_sitter::Node;

pub trait UsageCollector: Send + Sync {
    fn collect_usage_nodes(&self, root: Node, source_code: &str) -> Result<Vec<Usage>, String> {
        let mut candidates = Vec::new();
        let mut stack: Vec<Node> = Vec::new();
        stack.push(root);

        while let Some(node) = stack.pop() {
            if let Some(usage_node) = self.extract_node_if_usage(node, source_code) {
                candidates.push(usage_node);
            }

            let mut cursor = node.walk();
            let mut children: Vec<Node> = Vec::new();
            for child in node.children(&mut cursor) {
                children.push(child);
            }
            for child in children.into_iter().rev() {
                stack.push(child);
            }
        }

        Ok(candidates)
    }

    fn extract_node_if_usage(&self, node: Node, source_code: &str) -> Option<Usage>;
}
