use crate::collectors::common::dependency_collectors::{
    add_dependency, DependencyCollector, DependencyHandler,
};
use petgraph::graph::{DiGraph, NodeIndex};
use std::collections::HashMap;
use tree_sitter::Node;

pub struct TypescriptDependencyCollector;

impl DependencyCollector for TypescriptDependencyCollector {
    #[allow(clippy::type_complexity)]
    fn collect_dependencies_from_root(
        root: Node,
        content: &str,
        definitions: &HashMap<String, usize>,
    ) -> Result<(DiGraph<usize, usize>, HashMap<usize, NodeIndex>), String> {
        let mut graph: DiGraph<usize, usize> = DiGraph::new();
        let mut line_nodes: HashMap<usize, NodeIndex> = HashMap::new();

        // Add all lines to line_nodes before collecting dependencies
        for line_num in 1..=content.lines().count() {
            line_nodes
                .entry(line_num)
                .or_insert_with(|| graph.add_node(line_num));
        }

        let kind_handlers = <Self as crate::collectors::common::dependency_collectors::DependencyCollector>::kind_handlers();

        let mut defs = definitions.clone();
        <Self as crate::collectors::common::dependency_collectors::DependencyCollector>::collect_dependencies_recursive(
            root,
            content,
            &mut graph,
            &mut line_nodes,
            &mut defs,
            &kind_handlers,
        );

        Ok((graph, line_nodes))
    }

    fn kind_handlers() -> HashMap<&'static str, DependencyHandler> {
        let mut kind_handlers: HashMap<&'static str, DependencyHandler> = HashMap::new();
        kind_handlers.insert("identifier", Self::handle_identifier);
        kind_handlers.insert("call_expression", Self::handle_call_expression);
        kind_handlers.insert("property_identifier", Self::handle_field_expression);
        kind_handlers
    }

    fn handle_identifier(
        n: Node,
        source_code: &str,
        graph: &mut DiGraph<usize, usize>,
        line_nodes: &mut HashMap<usize, NodeIndex>,
        definitions: &mut HashMap<String, usize>,
    ) {
        let start_line = n.start_position().row + 1;
        let parent_kind = n.parent().map(|p| p.kind());
        let name = n
            .utf8_text(source_code.as_bytes())
            .unwrap()
            .trim()
            .to_string();

        let is_declaration_name = parent_kind == Some("function_declaration")
            && (n.parent().unwrap().child_by_field_name("name") == Some(n))
            || parent_kind == Some("class_declaration")
                && (n.parent().unwrap().child_by_field_name("name") == Some(n))
            || parent_kind == Some("interface_declaration")
                && (n.parent().unwrap().child_by_field_name("name") == Some(n))
            || parent_kind == Some("type_alias_declaration")
                && (n.parent().unwrap().child_by_field_name("name") == Some(n))
            || parent_kind == Some("enum_declaration")
                && (n.parent().unwrap().child_by_field_name("name") == Some(n));

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

    fn handle_call_expression(
        n: Node,
        source_code: &str,
        graph: &mut DiGraph<usize, usize>,
        line_nodes: &mut HashMap<usize, NodeIndex>,
        definitions: &mut HashMap<String, usize>,
    ) {
        let start_line = n.start_position().row + 1;
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
                        add_dependency(
                            arg_child.start_position().row + 1,
                            *def_line,
                            graph,
                            line_nodes,
                        );
                    }
                }
            }
        }
    }

    fn handle_field_expression(
        n: Node,
        source_code: &str,
        graph: &mut DiGraph<usize, usize>,
        line_nodes: &mut HashMap<usize, NodeIndex>,
        definitions: &mut HashMap<String, usize>,
    ) {
        let start_line = n.start_position().row + 1;
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

    fn handle_struct_expression(
        _n: Node,
        _source_code: &str,
        _graph: &mut DiGraph<usize, usize>,
        _line_nodes: &mut HashMap<usize, NodeIndex>,
        _definitions: &mut HashMap<String, usize>,
    ) {
    }
}
