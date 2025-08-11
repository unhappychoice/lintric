use std::collections::HashMap;
use tree_sitter::Node;

pub trait DefinitionCollector: Send + Sync {
    fn collect_definitions_from_root<'a>(
        &self,
        root: Node<'a>,
        content: &'a str,
    ) -> Result<HashMap<String, usize>, String> {
        let mut definitions: HashMap<String, usize> = HashMap::new();
        let mut stack: Vec<Node<'a>> = Vec::new();
        stack.push(root);

        while let Some(node) = stack.pop() {
            self.process_node(node, content, &mut definitions);

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

    fn process_node<'a>(
        &self,
        node: Node<'a>,
        source_code: &'a str,
        definitions: &mut HashMap<String, usize>,
    );

    fn collect_variable_definitions<'a>(
        &self,
        node: Node<'a>,
        source_code: &'a str,
        definitions: &mut HashMap<String, usize>,
    );

    fn collect_function_definitions<'a>(
        &self,
        node: Node<'a>,
        source_code: &'a str,
        definitions: &mut HashMap<String, usize>,
    );

    fn collect_type_definitions<'a>(
        &self,
        node: Node<'a>,
        source_code: &'a str,
        definitions: &mut HashMap<String, usize>,
    );

    fn collect_import_definitions<'a>(
        &self,
        node: Node<'a>,
        source_code: &'a str,
        definitions: &mut HashMap<String, usize>,
    );

    fn collect_closure_definitions<'a>(
        &self,
        node: Node<'a>,
        source_code: &'a str,
        definitions: &mut HashMap<String, usize>,
    );
}

pub fn find_identifiers_in_pattern(node: Node, source_code: &str) -> Vec<(String, usize)> {
    let mut identifiers = Vec::new();
    let mut stack: Vec<Node> = Vec::new();
    stack.push(node);

    while let Some(n) = stack.pop() {
        if n.kind() == "identifier" {
            let name = n
                .utf8_text(source_code.as_bytes())
                .unwrap()
                .trim()
                .to_string();
            identifiers.push((name.clone(), n.start_position().row + 1));
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
    identifiers
}
