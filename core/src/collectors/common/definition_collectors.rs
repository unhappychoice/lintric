use crate::models::Definition;
use tree_sitter::Node;

pub trait DefinitionCollector<'a>: Send + Sync {
    fn collect_definitions_from_root(&self, root: Node<'a>) -> Result<Vec<Definition>, String> {
        let mut definitions = vec![];
        let mut stack: Vec<(Node<'a>, Option<String>)> = Vec::new();
        stack.push((root, None));

        while let Some((node, current_scope)) = stack.pop() {
            let new_scope = self.determine_scope(&node, &current_scope);

            definitions.extend(self.process_node(node, &new_scope));

            let mut cursor = node.walk();
            let mut children: Vec<Node<'a>> = Vec::new();
            for child in node.children(&mut cursor) {
                children.push(child);
            }
            for child in children.into_iter().rev() {
                stack.push((child, new_scope.clone()));
            }
        }

        Ok(definitions)
    }

    fn process_node(&self, node: Node<'a>, current_scope: &Option<String>) -> Vec<Definition>;

    fn determine_scope(&self, node: &Node<'a>, parent_scope: &Option<String>) -> Option<String>;

    fn collect_variable_definitions(
        &self,
        node: Node<'a>,
        current_scope: &Option<String>,
    ) -> Vec<Definition>;

    fn collect_function_definitions(
        &self,
        node: Node<'a>,
        current_scope: &Option<String>,
    ) -> Vec<Definition>;

    fn collect_type_definitions(
        &self,
        node: Node<'a>,
        current_scope: &Option<String>,
    ) -> Vec<Definition>;

    fn collect_import_definitions(
        &self,
        node: Node<'a>,
        current_scope: &Option<String>,
    ) -> Vec<Definition>;

    fn collect_closure_definitions(
        &self,
        node: Node<'a>,
        current_scope: &Option<String>,
    ) -> Vec<Definition>;

    fn collect_macro_definitions(
        &self,
        node: Node<'a>,
        current_scope: &Option<String>,
    ) -> Vec<Definition>;
}

pub fn find_identifier_nodes_in_node(node: Node) -> Vec<Node> {
    let mut identifiers = vec![];
    let mut stack: Vec<Node> = vec![];
    stack.push(node);

    while let Some(n) = stack.pop() {
        if n.kind() == "identifier" {
            identifiers.push(n);
        }

        let mut cursor = n.walk();
        let mut children: Vec<Node> = vec![];
        for child in n.children(&mut cursor) {
            children.push(child);
        }
        for child in children.into_iter().rev() {
            stack.push(child);
        }
    }
    identifiers
}
