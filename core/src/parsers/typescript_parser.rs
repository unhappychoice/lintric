use petgraph::graph::{DiGraph, NodeIndex};
use std::collections::HashMap;
use tree_sitter::{Node, Parser as TreeSitterParser};

#[allow(clippy::type_complexity)]
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
        .map_err(|e| format!("Error loading TypeScript/TSX grammar: {e}"))?;

    let tree = parser
        .parse(content, None)
        .ok_or_else(|| "Failed to parse the source code.".to_string())?;

    let mut definitions: HashMap<String, usize> = HashMap::new();
    let mut graph: DiGraph<usize, usize> = DiGraph::new();
    let mut line_nodes: HashMap<usize, NodeIndex> = HashMap::new();

    // Add all lines to line_nodes before collecting definitions and dependencies
    for line_num in 1..=content.lines().count() {
        line_nodes.entry(line_num).or_insert_with(|| graph.add_node(line_num));
    }

    collect_definitions(tree.root_node(), content, &mut definitions);
    collect_dependencies(
        tree.root_node(),
        content,
        &mut graph,
        &mut line_nodes,
        &mut definitions,
    );

    Ok((graph, line_nodes))
}

fn collect_definitions(node: Node, source_code: &str, definitions: &mut HashMap<String, usize>) {
    let mut stack: Vec<Node> = Vec::new();
    stack.push(node);

    while let Some(n) = stack.pop() {
        let start_line = n.start_position().row + 1;

        match n.kind() {
            "variable_declarator" => {
                if let Some(pattern_node) = n.child_by_field_name("name") {
                    let name = pattern_node
                        .utf8_text(source_code.as_bytes())
                        .unwrap()
                        .trim()
                        .to_string();
                    definitions.insert(name.clone(), start_line);
                } else if let Some(pattern_node) = n.child_by_field_name("pattern") {
                    find_identifiers_in_pattern(pattern_node, source_code, definitions);
                } else {

                }
            }
           "arrow_function" | "function" => {
               if let Some(parameters_node) = n.child_by_field_name("parameters") {
                   let mut param_cursor = parameters_node.walk();
                   for param_child in parameters_node.children(&mut param_cursor) {
                       if param_child.kind() == "required_parameter" || param_child.kind() == "optional_parameter" {
                           if let Some(pattern_node) = param_child.child_by_field_name("pattern") {
                               find_identifiers_in_pattern(pattern_node, source_code, definitions);
                           } else if let Some(identifier_node) = param_child.child(0) {
                               if identifier_node.kind() == "identifier" {
                                   let name = identifier_node
                                       .utf8_text(source_code.as_bytes())
                                       .unwrap()
                                       .trim()
                                       .to_string();
                                   definitions.insert(name.clone(), identifier_node.start_position().row + 1);
                               }
                           }
                       }
                   }
               }
           }
            "function_declaration" => {
                if let Some(name_node) = n.child_by_field_name("name") {
                    let name = name_node
                        .utf8_text(source_code.as_bytes())
                        .unwrap()
                        .trim()
                        .to_string();
                    definitions.insert(name.clone(), start_line);
                }
                // Add parameters to definitions
                if let Some(parameters_node) = n.child_by_field_name("parameters") {
                    let mut param_cursor = parameters_node.walk();
                    for param_child in parameters_node.children(&mut param_cursor) {
                        if param_child.kind() == "required_parameter" || param_child.kind() == "optional_parameter" {
                            if let Some(pattern_node) = param_child.child_by_field_name("pattern") {
                                find_identifiers_in_pattern(pattern_node, source_code, definitions);
                            } else if let Some(identifier_node) = param_child.child(0) { // Direct identifier for simple parameters
                                if identifier_node.kind() == "identifier" {
                                    let name = identifier_node
                                        .utf8_text(source_code.as_bytes())
                                        .unwrap()
                                        .trim()
                                        .to_string();
                                    definitions.insert(name.clone(), identifier_node.start_position().row + 1);
                                }
                            }
                        }
                    }
                }
            }
            "class_declaration"
            | "interface_declaration"
            | "type_alias_declaration"
            | "enum_declaration" => {
                if let Some(pattern_node) = n.child_by_field_name("name") {
                    let name = pattern_node
                        .utf8_text(source_code.as_bytes())
                        .unwrap()
                        .trim()
                        .to_string();
                    definitions.insert(name.clone(), start_line);
                } else if let Some(pattern_node) = n.child_by_field_name("pattern") {
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
            "import_statement" => {
                for i in 0..n.child_count() {
                    let child = n.child(i);

                    if let Some(child) = child {
                        if child.kind() != "import_clause" {
                            continue;
                        }

                        let mut import_clause_cursor = child.walk();
                        for import_clause_child in child.children(&mut import_clause_cursor) {
                            if import_clause_child.kind() == "from_clause" {
                                continue;
                            }
                            match import_clause_child.kind() {
                                "named_imports" => {
                                    let mut named_imports_cursor = import_clause_child.walk();
                                    for named_import_child in
                                        import_clause_child.children(&mut named_imports_cursor)
                                    {
                                        if named_import_child.kind() == "import_specifier" {
                                            if let Some(identifier_node) =
                                                named_import_child.child(0)
                                            {
                                                let imported_symbol = identifier_node
                                                    .utf8_text(source_code.as_bytes())
                                                    .unwrap()
                                                    .trim()
                                                    .to_string();
                                                definitions
                                                    .insert(imported_symbol.clone(), start_line);
                                            }
                                        }
                                    }
                                }
                                "namespace_import" => {
                                    if let Some(alias_node) =
                                        import_clause_child.child_by_field_name("alias")
                                    {
                                        let imported_symbol = alias_node
                                            .utf8_text(source_code.as_bytes())
                                            .unwrap()
                                            .trim()
                                            .to_string();
                                        definitions.insert(imported_symbol.clone(), start_line);
                                    }
                                }
                                "identifier" => {
                                    // Default import
                                    let imported_symbol = import_clause_child
                                        .utf8_text(source_code.as_bytes())
                                        .unwrap()
                                        .trim()
                                        .to_string();
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

fn collect_dependencies(
    node: Node,
    source_code: &str,
    graph: &mut DiGraph<usize, usize>,
    line_nodes: &mut HashMap<usize, NodeIndex>,
    definitions: &mut HashMap<String, usize>,
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

        // Add this block to handle arguments in call_expression
        match n.kind() {
            "identifier" => {
                let parent_kind = n.parent().map(|p| p.kind());
                let name = n
                    .utf8_text(source_code.as_bytes())
                    .unwrap()
                    .trim()
                    .to_string();

                let is_declaration_name = parent_kind == Some("function_declaration") && n.parent().unwrap().child_by_field_name("name").map_or(false, |node| node == n) ||
                    parent_kind == Some("class_declaration") && n.parent().unwrap().child_by_field_name("name").map_or(false, |node| node == n) ||
                    parent_kind == Some("interface_declaration") && n.parent().unwrap().child_by_field_name("name").map_or(false, |node| node == n) ||
                    parent_kind == Some("type_alias_declaration") && n.parent().unwrap().child_by_field_name("name").map_or(false, |node| node == n) ||
                    parent_kind == Some("enum_declaration") && n.parent().unwrap().child_by_field_name("name").map_or(false, |node| node == n);

                if parent_kind != Some("variable_declarator")
                    && parent_kind != Some("property_identifier")
                    && parent_kind != Some("arguments")
                    && !is_declaration_name
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
                // Add this block to handle arguments in call_expression
                if let Some(arguments_node) = n.child_by_field_name("arguments") {
                    let mut arg_cursor = arguments_node.walk();
                    for arg_child in arguments_node.children(&mut arg_cursor) {
                        if arg_child.kind() == "identifier" {
                            let name = arg_child
                                .utf8_text(source_code.as_bytes())
                                .unwrap()
                                .trim()
                                .to_string();
                            if let Some(def_line) = definitions.get(&name) {
                                add_dependency(arg_child.start_position().row + 1, *def_line, graph, line_nodes);
                            }
                        }
                    }
                }
            }
            "property_identifier" => {
                if let Some(parent) = n.parent() {
                    if parent.kind() == "member_expression" {
                        if let Some(object_node) = parent.child_by_field_name("object") {
                            if object_node.kind() == "identifier" {
                                let name = object_node
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
                }
            }
            _ => {}
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
