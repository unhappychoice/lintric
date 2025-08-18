use crate::models::Usage;
use tree_sitter::Node;

pub trait UsageNodeCollector<'a>: Send + Sync {
    fn collect_usage_nodes(&self, root: Node<'a>) -> Result<Vec<Usage<'a>>, String> {
        let mut candidates = Vec::new();
        let mut stack: Vec<(Node<'a>, Option<String>)> = Vec::new();
        stack.push((root, None));

        while let Some((node, current_scope)) = stack.pop() {
            let new_scope = self.determine_scope(&node, &current_scope);

            if let Some(usage_node) = self.extract_node_if_usage(node, &new_scope) {
                candidates.push(usage_node);
            }

            let mut cursor = node.walk();
            let mut children: Vec<Node<'a>> = Vec::new();
            for child in node.children(&mut cursor) {
                children.push(child);
            }
            for child in children.into_iter().rev() {
                stack.push((child, new_scope.clone()));
            }
        }

        Ok(candidates)
    }

    fn extract_node_if_usage(
        &self,
        node: Node<'a>,
        current_scope: &Option<String>,
    ) -> Option<Usage<'a>>;

    fn determine_scope(&self, node: &Node<'a>, parent_scope: &Option<String>) -> Option<String>;
}
