use petgraph::graph::{DiGraph, NodeIndex};
use std::collections::HashMap;
use tree_sitter::{Language, Node, Parser as TreeSitterParser};

pub fn parse_typescript_code(
    content: &str,
    is_tsx: bool,
) -> Result<(DiGraph<usize, usize>, HashMap<usize, NodeIndex>), String> {
    let mut parser = TreeSitterParser::new();
    let language = if is_tsx {
        tree_sitter_typescript::language_tsx()
    } else {
        tree_sitter_typescript::language_typescript()
    };
    parser
        .set_language(&language)
        .map_err(|e| format!("Error loading TypeScript/TSX grammar: {}", e))?;

    let tree = parser.parse(&content, None).ok_or_else(|| "Failed to parse the source code.".to_string())?;

    let mut definitions: HashMap<String, usize> = HashMap::new();
    let mut graph: DiGraph<usize, usize> = DiGraph::new();
    let mut line_nodes: HashMap<usize, NodeIndex> = HashMap::new();

    collect_definitions(tree.root_node(), content, &mut definitions, &language);
    collect_dependencies(tree.root_node(), content, &mut graph, &mut line_nodes, &definitions, &language);

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
        "variable_declarator" | "function_declaration" | "class_declaration" | "interface_declaration" | "type_alias_declaration" | "enum_declaration" => {
            if let Some(pattern_node) = node.child_by_field_name("name") {
                let name = pattern_node.utf8_text(source_code.as_bytes()).unwrap().trim().to_string();
                definitions.insert(name.clone(), start_line);
            } else if let Some(pattern_node) = node.child_by_field_name("pattern") {
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
        "import_statement" => {
            for i in 0..node.child_count() {
                let child = node.child(i);

                if let Some(child) = child {
                    if child.kind() != "import_clause" { continue; }

                    let mut import_clause_cursor = child.walk();
                    for import_clause_child in child.children(&mut import_clause_cursor) {
                        if import_clause_child.kind() == "from_clause" { continue; }
                        match import_clause_child.kind() {
                            "named_imports" => {
                                let mut named_imports_cursor = import_clause_child.walk();
                                for named_import_child in import_clause_child.children(&mut named_imports_cursor) {
                                    if named_import_child.kind() == "import_specifier" {
                                        if let Some(identifier_node) = named_import_child.child(0) {
                                            let imported_symbol = identifier_node.utf8_text(source_code.as_bytes()).unwrap().trim().to_string();
                                            definitions.insert(imported_symbol.clone(), start_line);
                                        }
                                    }
                                }
                            }
                            "namespace_import" => {
                                if let Some(alias_node) = import_clause_child.child_by_field_name("alias") {
                                    let imported_symbol = alias_node.utf8_text(source_code.as_bytes()).unwrap().trim().to_string();
                                    definitions.insert(imported_symbol.clone(), start_line);
                                }
                            }
                            "identifier" => { // Default import
                                let imported_symbol = import_clause_child.utf8_text(source_code.as_bytes()).unwrap().trim().to_string();
                                definitions.insert(imported_symbol.clone(), start_line);
                            }
                            _ => {}
                        }
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

            if parent_kind != Some("variable_declarator") && parent_kind != Some("property_identifier") { // Avoid re-adding definitions or property access
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
        "property_identifier" => {
            if let Some(parent) = node.parent() {
                if parent.kind() == "member_expression" {
                    if let Some(object_node) = parent.child_by_field_name("object") {
                        if object_node.kind() == "identifier" {
                            let name = object_node.utf8_text(source_code.as_bytes()).unwrap().trim().to_string();
                            if let Some(def_line) = definitions.get(&name) {
                                add_dependency(start_line, *def_line, graph, line_nodes);
                            }
                        }
                    }
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
