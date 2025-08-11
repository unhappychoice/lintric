use crate::collectors::common::definition_collectors::{
    find_identifiers_in_pattern, DefinitionCollector,
};
use std::collections::HashMap;
use tree_sitter::Node;

pub struct RustDefinitionCollector;

impl DefinitionCollector for RustDefinitionCollector {
    fn process_node<'a>(
        &self,
        node: Node<'a>,
        source_code: &'a str,
        definitions: &mut HashMap<String, usize>,
    ) {
        match node.kind() {
            "let_declaration"
            | "variable_declarator"
            | "for_expression"
            | "if_expression"
            | "while_expression" => {
                self.collect_variable_definitions(node, source_code, definitions);
            }
            "function_item" => {
                self.collect_function_definitions(node, source_code, definitions);
            }
            "struct_item" | "enum_item" | "trait_item" | "impl_item" | "type_alias" => {
                self.collect_type_definitions(node, source_code, definitions);
            }
            "use_declaration" => {
                self.collect_import_definitions(node, source_code, definitions);
            }
            "closure_expression" => {
                self.collect_closure_definitions(node, source_code, definitions);
            }
            _ => {}
        }
    }

    fn collect_variable_definitions<'a>(
        &self,
        node: Node<'a>,
        source_code: &'a str,
        definitions: &mut HashMap<String, usize>,
    ) {
        let start_line = node.start_position().row + 1;
        match node.kind() {
            "for_expression" => {
                if let Some(pattern_node) = node.child_by_field_name("pattern") {
                    let identifiers = find_identifiers_in_pattern(pattern_node, source_code);
                    for (name, line) in identifiers {
                        definitions.insert(name, line);
                    }
                }
            }
            "if_expression" | "while_expression" => {
                let mut cursor = node.walk();
                for let_condition_node in node.children(&mut cursor) {
                    if let_condition_node.kind() == "let_condition" {
                        let mut let_cursor = let_condition_node.walk();
                        for destruct_pattern_node in let_condition_node.children(&mut let_cursor) {
                            if destruct_pattern_node.kind() == "tuple_struct_pattern" {
                                let mut identifiers =
                                    find_identifiers_in_pattern(destruct_pattern_node, source_code)
                                        .into_iter();
                                // Remove the first element, which is the name of the tuple struct.
                                identifiers.next();
                                for (name, line) in identifiers {
                                    definitions.insert(name, line);
                                }
                            }
                        }
                    }
                }
            }
            "let_declaration" | "variable_declarator" => {
                if let Some(pattern_node) = node.child_by_field_name("name") {
                    let name = pattern_node
                        .utf8_text(source_code.as_bytes())
                        .unwrap()
                        .trim()
                        .to_string();
                    definitions.insert(name, start_line);
                } else if let Some(pattern_node) = node.child_by_field_name("pattern") {
                    let identifiers = find_identifiers_in_pattern(pattern_node, source_code);
                    for (name, line) in identifiers {
                        definitions.insert(name, line);
                    }
                }
            }
            _ => {}
        }
    }

    fn collect_function_definitions<'a>(
        &self,
        node: Node<'a>,
        source_code: &'a str,
        definitions: &mut HashMap<String, usize>,
    ) {
        let start_line = node.start_position().row + 1;
        if let Some(name_node) = node.child_by_field_name("name") {
            let name = name_node
                .utf8_text(source_code.as_bytes())
                .unwrap()
                .trim()
                .to_string();
            definitions.insert(name, start_line);
        }
        if let Some(parameters_node) = node.child_by_field_name("parameters") {
            let mut param_cursor = parameters_node.walk();
            for param_child in parameters_node.children(&mut param_cursor) {
                if param_child.kind() == "parameter" {
                    if let Some(pattern_node) = param_child.child_by_field_name("pattern") {
                        let identifiers = find_identifiers_in_pattern(pattern_node, source_code);
                        for (name, line) in identifiers {
                            definitions.insert(name, line);
                        }
                    }
                }
            }
        }
    }

    fn collect_type_definitions<'a>(
        &self,
        node: Node<'a>,
        source_code: &'a str,
        definitions: &mut HashMap<String, usize>,
    ) {
        let start_line = node.start_position().row + 1;
        if let Some(pattern_node) = node.child_by_field_name("name") {
            let name = pattern_node
                .utf8_text(source_code.as_bytes())
                .unwrap()
                .trim()
                .to_string();
            definitions.insert(name, start_line);
        } else if let Some(pattern_node) = node.child_by_field_name("pattern") {
            let identifiers = find_identifiers_in_pattern(pattern_node, source_code);
            for (name, line) in identifiers {
                definitions.insert(name, line);
            }
        }
    }

    fn collect_import_definitions<'a>(
        &self,
        node: Node<'a>,
        source_code: &'a str,
        definitions: &mut HashMap<String, usize>,
    ) {
        let start_line = node.start_position().row + 1;
        let mut use_cursor = node.walk();
        for use_child in node.children(&mut use_cursor) {
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
                    for clause_child_node in use_child.children(&mut clause_cursor) {
                        if clause_child_node.kind() == "identifier"
                            || clause_child_node.kind() == "scoped_identifier"
                        {
                            let name = clause_child_node
                                .utf8_text(source_code.as_bytes())
                                .unwrap()
                                .trim()
                                .to_string();
                            definitions.insert(name, start_line);
                        } else if clause_child_node.kind() == "use_as_clause" {
                            if let Some(alias_node) = clause_child_node.child_by_field_name("alias")
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

    fn collect_closure_definitions<'a>(
        &self,
        node: Node<'a>,
        source_code: &'a str,
        definitions: &mut HashMap<String, usize>,
    ) {
        if let Some(parameters_node) = node.child_by_field_name("parameters") {
            let mut param_cursor = parameters_node.walk();
            for param_child in parameters_node.children(&mut param_cursor) {
                let identifiers = find_identifiers_in_pattern(param_child, source_code);
                for (name, line) in identifiers {
                    definitions.insert(name, line);
                }
            }
        }
    }
}
