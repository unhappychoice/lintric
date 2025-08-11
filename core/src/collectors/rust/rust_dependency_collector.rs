use crate::collectors::common::dependency_collectors::{add_dependency, DependencyCollector};
use petgraph::graph::{DiGraph, NodeIndex};
use std::collections::HashMap;
use tree_sitter::Node;

pub struct RustDependencyCollector;

impl DependencyCollector for RustDependencyCollector {
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
            "field_expression" => {
                self.handle_field_expression(n, source_code, graph, line_nodes, definitions);
            }
            "struct_expression" => {
                self.handle_struct_expression(n, source_code, graph, line_nodes, definitions);
            }
            _ => {}
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

        if parent_kind != Some("pattern") && !is_declaration_name && !is_parameter_declaration {
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

    fn handle_struct_expression<'a>(
        &self,
        n: Node<'a>,
        source_code: &'a str,
        graph: &mut DiGraph<usize, usize>,
        line_nodes: &mut HashMap<usize, NodeIndex>,
        definitions: &mut HashMap<String, usize>,
    ) {
        let start_line = n.start_position().row + 1;
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
}
