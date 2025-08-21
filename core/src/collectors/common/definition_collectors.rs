use crate::models::Definition;
use tree_sitter::Node;

pub trait DefinitionCollector<'a>: Send + Sync {
    fn collect_definitions_from_root(&self, root: Node<'a>) -> Result<Vec<Definition>, String> {
        let mut definitions = vec![];
        let mut stack: Vec<Node<'a>> = Vec::new();
        stack.push(root);

        while let Some(node) = stack.pop() {
            definitions.extend(self.process_node(node));

            let mut cursor = node.walk();
            let mut children: Vec<Node<'a>> = Vec::new();
            for child in node.children(&mut cursor) {
                children.push(child);
            }
            for child in children.into_iter().rev() {
                stack.push(child);
            }
        }

        Ok(definitions)
    }

    fn process_node(&self, node: Node<'a>) -> Vec<Definition>;

    fn collect_variable_definitions(&self, node: Node<'a>) -> Vec<Definition>;

    fn collect_function_definitions(&self, node: Node<'a>) -> Vec<Definition>;

    fn collect_type_definitions(&self, node: Node<'a>) -> Vec<Definition>;

    fn collect_import_definitions(&self, node: Node<'a>) -> Vec<Definition>;

    fn collect_closure_definitions(&self, node: Node<'a>) -> Vec<Definition>;

    fn collect_macro_definitions(&self, node: Node<'a>) -> Vec<Definition>;
}

pub fn find_identifier_nodes_in_node<'a>(node: Node<'a>) -> Vec<Node<'a>> {
    let mut identifiers = vec![];
    find_identifier_nodes_recursive(node, &mut identifiers);
    identifiers
}

fn find_identifier_nodes_recursive<'a>(node: Node<'a>, identifiers: &mut Vec<Node<'a>>) {
    match node.kind() {
        "identifier" => {
            identifiers.push(node);
        }
        "shorthand_property_identifier_pattern" => {
            // For TypeScript/TSX object destructuring patterns like { prop }
            // The shorthand_property_identifier_pattern node itself represents the identifier
            identifiers.push(node);
        }
        "object_assignment_pattern" => {
            // For TypeScript/TSX patterns like { prop = defaultValue }
            // Only extract the property name (left side), not the default value (right side)
            if let Some(left_node) = node.child_by_field_name("left") {
                find_identifier_nodes_recursive(left_node, identifiers);
            }
            // Skip the right side (default value) as it's a usage, not a definition
        }
        "assignment_pattern" => {
            // For array destructuring with defaults like [a = defaultValue]
            // Only extract the variable name (left side), not the default value (right side)
            if let Some(left_node) = node.child_by_field_name("left") {
                find_identifier_nodes_recursive(left_node, identifiers);
            }
            // Skip the right side (default value) as it's a usage, not a definition
        }
        _ => {
            // Continue traversing for other node types
            let mut cursor = node.walk();
            for child in node.children(&mut cursor) {
                find_identifier_nodes_recursive(child, identifiers);
            }
        }
    }
}
