use crate::collectors::common::dependency_collectors::{add_dependency, DependencyCollector};
use petgraph::graph::{DiGraph, NodeIndex};
use std::collections::HashMap;
use tree_sitter::Node;

pub struct TypescriptDependencyCollector;

impl DependencyCollector for TypescriptDependencyCollector {
    fn process_node<'a>(
        &self,
        n: Node<'a>,
        source_code: &'a str,
        graph: &mut DiGraph<usize, usize>,
        line_nodes: &mut HashMap<usize, NodeIndex>,
        definitions: &mut HashMap<String, usize>,
    ) {
        match n.kind() {
            "identifier" => {
                self.handle_identifier(n, source_code, graph, line_nodes, definitions);
            }
            "call_expression" => {
                self.handle_call_expression(n, source_code, graph, line_nodes, definitions);
            }
            "property_identifier" => {
                self.handle_field_expression(n, source_code, graph, line_nodes, definitions);
            }
            _ => {
                // No equivalent to struct_expression in TypeScript, so not called here.
            }
        }
    }

    fn handle_identifier<'a>(
        &self,
        n: Node<'a>,
        source_code: &'a str,
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

    fn handle_call_expression<'a>(
        &self,
        n: Node<'a>,
        source_code: &'a str,
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

    fn handle_field_expression<'a>(
        &self,
        n: Node<'a>,
        source_code: &'a str,
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

    fn handle_struct_expression<'a>(
        &self,
        _n: Node<'a>,
        _source_code: &'a str,
        _graph: &mut DiGraph<usize, usize>,
        _line_nodes: &mut HashMap<usize, NodeIndex>,
        _definitions: &mut HashMap<String, usize>,
    ) {
        // Empty implementation as there is no equivalent to struct_expression in TypeScript.
    }
}
