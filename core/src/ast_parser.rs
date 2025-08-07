use petgraph::graph::{DiGraph, NodeIndex};
use std::collections::HashMap;
use tree_sitter::{Language, Node, Parser as TreeSitterParser, Tree};

pub fn parse_code(content: &str, file_path: &str) -> Result<Tree, String> {
    let mut parser = TreeSitterParser::new();

    let language = if file_path.ends_with(".rs") {
        tree_sitter_rust::language()
    } else if file_path.ends_with(".ts") {
        tree_sitter_typescript::language_typescript()
    } else if file_path.ends_with(".tsx") {
        tree_sitter_typescript::language_tsx()
    } else {
        return Err(format!("Unsupported file extension for {}. Only .rs, .ts, .tsx are supported.", file_path));
    };

    parser
        .set_language(&language)
        .map_err(|e| format!("Error loading grammar: {}", e))?;

    parser.parse(&content, None).ok_or_else(|| "Failed to parse the source code.".to_string())
}

pub fn traverse(
    node: Node,
    source_code: &str,
    graph: &mut DiGraph<usize, usize>,
    line_nodes: &mut HashMap<usize, NodeIndex>,
    definitions: &mut HashMap<String, usize>,
    language: &Language,
) {
    let start_line = node.start_position().row + 1;
    let end_line = node.end_position().row + 1;

    for line in start_line..=end_line {
        if !line_nodes.contains_key(&line) {
            let node_index = graph.add_node(line);
            line_nodes.insert(line, node_index);
        }
    }

    match node.kind() {
        "let_declaration" | "variable_declarator" | "function_declaration" | "class_declaration" => {
            // Existing definition handling
            if let Some(pattern_node) = node.child_by_field_name("pattern") {
                find_identifiers_in_pattern(pattern_node, source_code, definitions, language);
            } else { // For function_declaration and class_declaration, the identifier is often a direct child
                let mut cursor = node.walk();
                for child in node.children(&mut cursor) {
                    if child.kind() == "identifier" {
                        find_identifiers_in_pattern(child, source_code, definitions, language);
                    }
                }
            }
        }
        "identifier" => {
            // Only add dependency if this identifier is a usage, not a definition.
            // A simple heuristic: if it's not a child of a "pattern" node, it's likely a usage.
            let parent_kind = node.parent().map(|p| p.kind());
            if parent_kind != Some("pattern") {
                let name = node.utf8_text(source_code.as_bytes()).unwrap().to_string();
                if let Some(def_line) = definitions.get(&name) {
                    add_dependency(start_line, *def_line, graph, line_nodes);
                }
            }
        }
        "call_expression" => {
            // Handle the function being called
            if let Some(function_node) = node.child_by_field_name("function") {
                if function_node.kind() == "identifier" {
                    let name = function_node.utf8_text(source_code.as_bytes()).unwrap().to_string();
                    if let Some(def_line) = definitions.get(&name) {
                        add_dependency(start_line, *def_line, graph, line_nodes);
                    }
                }
            }
        }
        "field_expression" => {
            // Handle the operand (e.g., `p1` in `p1.x`)
            if let Some(operand_node) = node.child_by_field_name("operand") {
                if operand_node.kind() == "identifier" {
                    let name = operand_node.utf8_text(source_code.as_bytes()).unwrap().to_string();
                    if let Some(def_line) = definitions.get(&name) {
                        add_dependency(start_line, *def_line, graph, line_nodes);
                    }
                }
            }
        }
        _ => {}
    }

    for i in 0..node.child_count() {
        if let Some(child) = node.child(i) {
            traverse(child, source_code, graph, line_nodes, definitions, language);
        }
    }
}

pub fn find_identifiers_in_pattern(
    node: Node,
    source_code: &str,
    definitions: &mut HashMap<String, usize>,
    language: &Language,
) {
    if node.kind() == "identifier" {
        let name = node.utf8_text(source_code.as_bytes()).unwrap().to_string();
        definitions.insert(name.clone(), node.start_position().row + 1);
    }

    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        find_identifiers_in_pattern(child, source_code, definitions, language);
    }
}

// Helper function to add dependencies
fn add_dependency(
    from_line: usize,
    to_line: usize,
    graph: &mut DiGraph<usize, usize>,
    line_nodes: &mut HashMap<usize, NodeIndex>,
) {
    if let (Some(&from_node), Some(&to_node)) = (line_nodes.get(&from_line), line_nodes.get(&to_line)) {
        if from_node != to_node { // Avoid self-loops
            let distance = from_line.abs_diff(to_line);
            graph.add_edge(from_node, to_node, distance);
        }
    }
}
