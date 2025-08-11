use crate::collectors::common::definition_collectors::{
    find_identifiers_in_pattern, DefinitionCollector,
};
use std::collections::HashMap;
use tree_sitter::Node;

pub struct TypescriptDefinitionCollector;

impl DefinitionCollector for TypescriptDefinitionCollector {
    fn process_node<'a>(
        &self,
        node: Node<'a>,
        source_code: &'a str,
        definitions: &mut HashMap<String, usize>,
    ) {
        match node.kind() {
            "variable_declarator" => {
                self.collect_variable_definitions(node, source_code, definitions);
            }
            "arrow_function" | "function" | "function_declaration" => {
                self.collect_function_definitions(node, source_code, definitions);
            }
            "class_declaration"
            | "interface_declaration"
            | "type_alias_declaration"
            | "enum_declaration" => {
                self.collect_type_definitions(node, source_code, definitions);
            }
            "import_statement" => {
                self.collect_import_definitions(node, source_code, definitions);
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
        if let Some(pattern_node) = node.child_by_field_name("name") {
            let name = pattern_node
                .utf8_text(source_code.as_bytes())
                .unwrap()
                .trim()
                .to_string();
            definitions.insert(name.clone(), start_line);
        } else if let Some(pattern_node) = node.child_by_field_name("pattern") {
            let identifiers = find_identifiers_in_pattern(pattern_node, source_code);
            for (name, line) in identifiers {
                definitions.insert(name, line);
            }
        }
    }

    fn collect_function_definitions<'a>(
        &self,
        node: Node<'a>,
        source_code: &'a str,
        definitions: &mut HashMap<String, usize>,
    ) {
        let start_line = node.start_position().row + 1;
        if node.kind() == "function_declaration" {
            if let Some(name_node) = node.child_by_field_name("name") {
                let name = name_node
                    .utf8_text(source_code.as_bytes())
                    .unwrap()
                    .trim()
                    .to_string();
                definitions.insert(name.clone(), start_line);
            }
        }
        if let Some(parameters_node) = node.child_by_field_name("parameters") {
            let mut param_cursor = parameters_node.walk();
            for param_child in parameters_node.children(&mut param_cursor) {
                if param_child.kind() == "required_parameter"
                    || param_child.kind() == "optional_parameter"
                {
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
            definitions.insert(name.clone(), start_line);
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
        for i in 0..node.child_count() {
            let child = node.child(i);

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
                                    if let Some(identifier_node) = named_import_child.child(0) {
                                        let imported_symbol = identifier_node
                                            .utf8_text(source_code.as_bytes())
                                            .unwrap()
                                            .trim()
                                            .to_string();
                                        definitions.insert(imported_symbol.clone(), start_line);
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

    fn collect_closure_definitions<'a>(
        &self,
        _node: Node<'a>,
        _source_code: &'a str,
        _definitions: &mut HashMap<String, usize>,
    ) {
        // Arrow functions are handled by collect_variable_definitions
    }
}
