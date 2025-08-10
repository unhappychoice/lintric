use petgraph::graph::{DiGraph, NodeIndex};
use std::collections::HashMap;
use tree_sitter::{Node, Parser as TreeSitterParser};

#[allow(clippy::type_complexity)]
pub fn parse_rust_code(
    content: &str,
) -> Result<(DiGraph<usize, usize>, HashMap<usize, NodeIndex>), String> {
    let mut parser = TreeSitterParser::new();
    parser
        .set_language(&tree_sitter_rust::language())
        .map_err(|e| format!("Error loading Rust grammar: {e}"))?;

    let tree = parser
        .parse(content, None)
        .ok_or_else(|| "Failed to parse the source code.".to_string())?;

    let mut definitions: HashMap<String, usize> = HashMap::new();
    let mut graph: DiGraph<usize, usize> = DiGraph::new();
    let mut line_nodes: HashMap<usize, NodeIndex> = HashMap::new();

    collect_definitions(tree.root_node(), content, &mut definitions);
    collect_dependencies(
        tree.root_node(),
        content,
        &mut graph,
        &mut line_nodes,
        &definitions,
    );

    Ok((graph, line_nodes))
}

fn collect_definitions(node: Node, source_code: &str, definitions: &mut HashMap<String, usize>) {
    let mut stack: Vec<Node> = Vec::new();
    stack.push(node);

    while let Some(n) = stack.pop() {
        let start_line = n.start_position().row + 1;

        match n.kind() {
            "let_declaration" | "variable_declarator" => {
                if let Some(pattern_node) = n.child_by_field_name("name") {
                    // For function_item, struct_item etc.
                    let name = pattern_node
                        .utf8_text(source_code.as_bytes())
                        .unwrap()
                        .trim()
                        .to_string();
                    definitions.insert(name, start_line);
                } else if let Some(pattern_node) = n.child_by_field_name("pattern") {
                    // For let_declaration
                    find_identifiers_in_pattern(pattern_node, source_code, definitions);
                } else {
                    let mut cursor = n.walk();
                    for child in n.children(&mut cursor) {
                        if child.kind() == "identifier" {
                            find_identifiers_in_pattern(child, source_code, definitions);
                        }
                    }
                }
            }
            "function_item" => {
                if let Some(name_node) = n.child_by_field_name("name") {
                    let name = name_node
                        .utf8_text(source_code.as_bytes())
                        .unwrap()
                        .trim()
                        .to_string();
                    definitions.insert(name, start_line);
                }
                // Add parameters to definitions
                if let Some(parameters_node) = n.child_by_field_name("parameters") {
                    let mut param_cursor = parameters_node.walk();
                    for param_child in parameters_node.children(&mut param_cursor) {
                        if param_child.kind() == "parameter" {
                            if let Some(pattern_node) = param_child.child_by_field_name("pattern") {
                                find_identifiers_in_pattern(pattern_node, source_code, definitions);
                            }
                        }
                    }
                }
            }
            "struct_item" | "enum_item" | "trait_item" | "impl_item" | "type_alias" => {
                if let Some(pattern_node) = n.child_by_field_name("name") {
                    // For function_item, struct_item etc.
                    let name = pattern_node
                        .utf8_text(source_code.as_bytes())
                        .unwrap()
                        .trim()
                        .to_string();
                    definitions.insert(name, start_line);
                } else if let Some(pattern_node) = n.child_by_field_name("pattern") {
                    // For let_declaration
                    find_identifiers_in_pattern(pattern_node, source_code, definitions);
                }
            }
            "use_declaration" => {
                let mut use_cursor = n.walk();
                for use_child in n.children(&mut use_cursor) {
                    match use_child.kind() {
                        "scoped_identifier" | "identifier" => {
                            let name = use_child
                                .utf8_text(source_code.as_bytes())
                                .unwrap()
                                .trim()
                                .to_string();
                            definitions.insert(name, start_line);
                        }
                        "use_clause" => {
                            let mut clause_cursor = use_child.walk();
                            for clause_child in use_child.children(&mut clause_cursor) {
                                if clause_child.kind() == "identifier"
                                    || clause_child.kind() == "scoped_identifier"
                                {
                                    let name = clause_child
                                        .utf8_text(source_code.as_bytes())
                                        .unwrap()
                                        .trim()
                                        .to_string();
                                    definitions.insert(name, start_line);
                                } else if clause_child.kind() == "use_as_clause" {
                                    if let Some(alias_node) =
                                        clause_child.child_by_field_name("alias")
                                    {
                                        let name = alias_node
                                            .utf8_text(source_code.as_bytes())
                                            .unwrap()
                                            .trim()
                                            .to_string();
                                        definitions.insert(name, start_line);
                                    }
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }
            _ => {}
        }

        // 子ノードを積む
        let mut cursor = n.walk();
        let mut children: Vec<Node> = Vec::new();
        for child in n.children(&mut cursor) {
            children.push(child);
        }
        // DFS の順序を大きく変えないために逆順で push
        for child in children.into_iter().rev() {
            stack.push(child);
        }
    }
}

fn collect_dependencies(
    node: Node,
    source_code: &str,
    graph: &mut DiGraph<usize, usize>,
    line_nodes: &mut HashMap<usize, NodeIndex>,
    definitions: &HashMap<String, usize>,
) {
    let mut stack: Vec<Node> = Vec::new();
    stack.push(node);

    while let Some(n) = stack.pop() {
        let start_line = n.start_position().row + 1;
        let end_line = n.end_position().row + 1;

        for line in start_line..=end_line {
            line_nodes
                .entry(line)
                .or_insert_with(|| graph.add_node(line));
        }

        match n.kind() {
            "identifier" => {
                let parent_kind = n.parent().map(|p| p.kind());
                let name = n
                    .utf8_text(source_code.as_bytes())
                    .unwrap()
                    .trim()
                    .to_string();

                // Avoid re-adding definitions and parameter declarations
                let is_declaration_name = parent_kind == Some("function_item")
                    && (n.parent().unwrap().child_by_field_name("name") == Some(n))
                    || parent_kind == Some("struct_item")
                        && (n.parent().unwrap().child_by_field_name("name") == Some(n))
                    || parent_kind == Some("enum_item")
                        && (n.parent().unwrap().child_by_field_name("name") == Some(n))
                    || parent_kind == Some("trait_item")
                        && (n.parent().unwrap().child_by_field_name("name") == Some(n))
                    || parent_kind == Some("impl_item")
                        && (n.parent().unwrap().child_by_field_name("name") == Some(n))
                    || parent_kind == Some("type_alias")
                        && (n.parent().unwrap().child_by_field_name("name") == Some(n));

                let is_parameter_declaration = parent_kind == Some("parameter");

                if parent_kind != Some("pattern")
                    && !is_declaration_name
                    && !is_parameter_declaration
                // Don't add dependency if it's the parameter declaration itself
                {
                    if let Some(def_line) = definitions.get(&name) {
                        add_dependency(start_line, *def_line, graph, line_nodes);
                    }
                }
            }
            "call_expression" => {
                if let Some(function_node) = n.child_by_field_name("function") {
                    if function_node.kind() == "identifier" {
                        let name = function_node
                            .utf8_text(source_code.as_bytes())
                            .unwrap()
                            .trim()
                            .to_string();
                        if let Some(def_line) = definitions.get(&name) {
                            add_dependency(start_line, *def_line, graph, line_nodes);
                        }
                    }
                }
            }
            "field_expression" => {
                if let Some(operand_node) = n.child_by_field_name("operand") {
                    if operand_node.kind() == "identifier" {
                        let name = operand_node
                            .utf8_text(source_code.as_bytes())
                            .unwrap()
                            .trim()
                            .to_string();
                        if let Some(def_line) = definitions.get(&name) {
                            add_dependency(start_line, *def_line, graph, line_nodes);
                        }
                    }
                }
                // Add dependency to the struct definition if the field access is on a struct instance
                if let Some(type_node) = n.child_by_field_name("field") {
                    let type_name = type_node
                        .utf8_text(source_code.as_bytes())
                        .unwrap()
                        .trim()
                        .to_string();
                    if let Some(def_line) = definitions.get(&type_name) {
                        add_dependency(start_line, *def_line, graph, line_nodes);
                    }
                }
            }
            "struct_expression" => {
                if let Some(type_node) = n.child_by_field_name("type") {
                    let type_name = type_node
                        .utf8_text(source_code.as_bytes())
                        .unwrap()
                        .trim()
                        .to_string();
                    if let Some(def_line) = definitions.get(&type_name) {
                        add_dependency(start_line, *def_line, graph, line_nodes);
                    }
                }
            }
            _ => {}
        }

        // 子ノードを積む
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

fn find_identifiers_in_pattern(
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
        if from_node != to_node {
            // Avoid self-loops
            let distance = from_line.abs_diff(to_line);
            graph.add_edge(from_node, to_node, distance);
        }
    }
}
