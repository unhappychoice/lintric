use petgraph::graph::{DiGraph, NodeIndex};
use std::collections::HashMap;
use tree_sitter::{Language, Node, Parser as TreeSitterParser};

pub fn parse_rust_code(
    content: &str,
) -> Result<(DiGraph<usize, usize>, HashMap<usize, NodeIndex>), String> {
    let mut parser = TreeSitterParser::new();
    parser
        .set_language(&tree_sitter_rust::language())
        .map_err(|e| format!("Error loading Rust grammar: {}", e))?;

    let tree = parser.parse(&content, None).ok_or_else(|| "Failed to parse the source code.".to_string())?;

    let mut definitions: HashMap<String, usize> = HashMap::new();
    let mut graph: DiGraph<usize, usize> = DiGraph::new();
    let mut line_nodes: HashMap<usize, NodeIndex> = HashMap::new();

    collect_definitions(tree.root_node(), content, &mut definitions, &tree_sitter_rust::language());
    collect_dependencies(tree.root_node(), content, &mut graph, &mut line_nodes, &definitions, &tree_sitter_rust::language());

    Ok((graph, line_nodes))
}

fn collect_definitions(
    node: Node,
    source_code: &str,
    definitions: &mut HashMap<String, usize>,
    language: &Language,
) {
    let start_line = node.start_position().row + 1;

    match node.kind() {
        "let_declaration" | "variable_declarator" | "function_item" | "struct_item" | "enum_item" | "trait_item" | "impl_item" | "type_alias" => {
            if let Some(pattern_node) = node.child_by_field_name("name") { // For function_item, struct_item etc.
                let name = pattern_node.utf8_text(source_code.as_bytes()).unwrap().trim().to_string();
                definitions.insert(name, start_line);
            } else if let Some(pattern_node) = node.child_by_field_name("pattern") { // For let_declaration
                find_identifiers_in_pattern(pattern_node, source_code, definitions, language);
            } else {
                let mut cursor = node.walk();
                for child in node.children(&mut cursor) {
                    if child.kind() == "identifier" {
                        find_identifiers_in_pattern(child, source_code, definitions, language);
                    }
                }
            }
        }
        _ => {}
    }

    for i in 0..node.child_count() {
        if let Some(child) = node.child(i) {
            collect_definitions(child, source_code, definitions, language);
        }
    }
}

fn collect_dependencies(
    node: Node,
    source_code: &str,
    graph: &mut DiGraph<usize, usize>,
    line_nodes: &mut HashMap<usize, NodeIndex>,
    definitions: &HashMap<String, usize>,
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
        "identifier" => {
            let parent_kind = node.parent().map(|p| p.kind());
            let name = node.utf8_text(source_code.as_bytes()).unwrap().trim().to_string();

            if parent_kind != Some("pattern") { // Avoid re-adding definitions
                if let Some(def_line) = definitions.get(&name) {
                    add_dependency(start_line, *def_line, graph, line_nodes);
                }
            }
        }
        "call_expression" => {
            if let Some(function_node) = node.child_by_field_name("function") {
                if function_node.kind() == "identifier" {
                    let name = function_node.utf8_text(source_code.as_bytes()).unwrap().trim().to_string();
                    if let Some(def_line) = definitions.get(&name) {
                        add_dependency(start_line, *def_line, graph, line_nodes);
                    }
                }
            }
        }
        "field_expression" => {
            if let Some(operand_node) = node.child_by_field_name("operand") {
                if operand_node.kind() == "identifier" {
                    let name = operand_node.utf8_text(source_code.as_bytes()).unwrap().trim().to_string();
                    if let Some(def_line) = definitions.get(&name) {
                        add_dependency(start_line, *def_line, graph, line_nodes);
                    }
                }
            }
            // Add dependency to the struct definition if the field access is on a struct instance
            if let Some(type_node) = node.child_by_field_name("field") {
                let type_name = type_node.utf8_text(source_code.as_bytes()).unwrap().trim().to_string();
                if let Some(def_line) = definitions.get(&type_name) {
                    add_dependency(start_line, *def_line, graph, line_nodes);
                }
            }
        }
        "struct_expression" => {
            if let Some(type_node) = node.child_by_field_name("type") {
                let type_name = type_node.utf8_text(source_code.as_bytes()).unwrap().trim().to_string();
                if let Some(def_line) = definitions.get(&type_name) {
                    add_dependency(start_line, *def_line, graph, line_nodes);
                }
            }
        }
        _ => {}
    }

    for i in 0..node.child_count() {
        if let Some(child) = node.child(i) {
            collect_dependencies(child, source_code, graph, line_nodes, definitions, language);
        }
    }
}

fn find_identifiers_in_pattern(
    node: Node,
    source_code: &str,
    definitions: &mut HashMap<String, usize>,
    language: &Language,
) {
    if node.kind() == "identifier" {
        let name = node.utf8_text(source_code.as_bytes()).unwrap().trim().to_string();
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
    let from_node_opt = line_nodes.get(&from_line);
    let to_node_opt = line_nodes.get(&to_line);

    if let (Some(&from_node), Some(&to_node)) = (from_node_opt, to_node_opt) {
        if from_node != to_node { // Avoid self-loops
            let distance = from_line.abs_diff(to_line);
            graph.add_edge(from_node, to_node, distance);
        }
    }
}
