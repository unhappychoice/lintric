use std::collections::HashMap;
use tree_sitter::Node;

pub type DefinitionHandler = fn(Node, &str, &mut HashMap<String, usize>);

pub trait DefinitionCollector {
    fn collect_definitions_from_root(
        root: Node,
        content: &str,
    ) -> Result<HashMap<String, usize>, String>;

    fn collect_definitions_recursive(
        node: Node,
        source_code: &str,
        definitions: &mut HashMap<String, usize>,
        kind_handlers: &HashMap<&str, DefinitionHandler>,
    ) {
        let mut stack: Vec<Node> = Vec::new();
        stack.push(node);

        while let Some(n) = stack.pop() {
            if let Some(handler) = kind_handlers.get(n.kind()) {
                handler(n, source_code, definitions);
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

    fn collect_variable_definitions(
        node: Node,
        source_code: &str,
        definitions: &mut HashMap<String, usize>,
    );

    fn collect_function_definitions(
        node: Node,
        source_code: &str,
        definitions: &mut HashMap<String, usize>,
    );

    fn collect_type_definitions(
        node: Node,
        source_code: &str,
        definitions: &mut HashMap<String, usize>,
    );

    fn collect_import_definitions(
        node: Node,
        source_code: &str,
        definitions: &mut HashMap<String, usize>,
    );

    fn collect_closure_definitions(
        node: Node,
        source_code: &str,
        definitions: &mut HashMap<String, usize>,
    );
}

pub fn find_identifiers_in_pattern(
    node: Node,
    source_code: &str,
    definitions: &mut HashMap<String, usize>,
) {
    let mut stack: Vec<Node> = Vec::new();
    stack.push(node);

    while let Some(n) = stack.pop() {
        if n.kind() == "identifier" {
            let name = n
                .utf8_text(source_code.as_bytes())
                .unwrap()
                .trim()
                .to_string();
            definitions.insert(name.clone(), n.start_position().row + 1);
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
