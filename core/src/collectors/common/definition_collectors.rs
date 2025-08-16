use crate::models::Definition;
use tree_sitter::Node;

pub trait DefinitionCollector: Send + Sync {
    fn collect_definitions_from_root<'a>(
        &self,
        root: Node<'a>,
        content: &'a str,
    ) -> Result<Vec<Definition>, String> {
        let mut definitions: Vec<Definition> = Vec::new();
        let mut stack: Vec<(Node<'a>, Option<String>)> = Vec::new();
        stack.push((root, None));

        while let Some((node, current_scope)) = stack.pop() {
            let new_scope = self.determine_scope(&node, content, &current_scope);

            self.process_node(node, content, &mut definitions, &new_scope);

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

    fn process_node<'a>(
        &self,
        node: Node<'a>,
        source_code: &'a str,
        definitions: &mut Vec<Definition>,
        current_scope: &Option<String>,
    );

    fn determine_scope<'a>(
        &self,
        node: &Node<'a>,
        source_code: &'a str,
        parent_scope: &Option<String>,
    ) -> Option<String>;

    fn collect_variable_definitions<'a>(
        &self,
        node: Node<'a>,
        source_code: &'a str,
        definitions: &mut Vec<Definition>,
        current_scope: &Option<String>,
    );

    fn collect_function_definitions<'a>(
        &self,
        node: Node<'a>,
        source_code: &'a str,
        definitions: &mut Vec<Definition>,
        current_scope: &Option<String>,
    );

    fn collect_type_definitions<'a>(
        &self,
        node: Node<'a>,
        source_code: &'a str,
        definitions: &mut Vec<Definition>,
        current_scope: &Option<String>,
    );

    fn collect_import_definitions<'a>(
        &self,
        node: Node<'a>,
        source_code: &'a str,
        definitions: &mut Vec<Definition>,
        current_scope: &Option<String>,
    );

    fn collect_closure_definitions<'a>(
        &self,
        node: Node<'a>,
        source_code: &'a str,
        definitions: &mut Vec<Definition>,
        current_scope: &Option<String>,
    );

    fn collect_macro_definitions<'a>(
        &self,
        node: Node<'a>,
        source_code: &'a str,
        definitions: &mut Vec<Definition>,
        current_scope: &Option<String>,
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
