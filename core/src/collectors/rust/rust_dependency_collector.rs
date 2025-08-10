use crate::collectors::common::dependency_collectors::{
    add_dependency, DependencyCollector, DependencyHandler,
};
use petgraph::graph::{DiGraph, NodeIndex};
use std::collections::HashMap;
use tree_sitter::Node;

pub struct RustDependencyCollector;

impl DependencyCollector for RustDependencyCollector {
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
        kind_handlers.insert("field_expression", Self::handle_field_expression);
        kind_handlers.insert("struct_expression", Self::handle_struct_expression);
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
    }

    fn handle_field_expression(
        n: Node,
        source_code: &str,
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

    fn handle_struct_expression(
        n: Node,
        source_code: &str,
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
