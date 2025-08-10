use crate::collectors::common::definition_collectors::{
    find_identifiers_in_pattern,
    DefinitionCollector,
    DefinitionHandler, // Add DefinitionHandler here
};
use std::collections::HashMap;
use tree_sitter::{Node, Parser as TreeSitterParser};

pub struct TypescriptDefinitionCollector;

impl DefinitionCollector for TypescriptDefinitionCollector {
    fn collect_definitions(
        content: &str,
        is_tsx: bool,
        parser: &mut TreeSitterParser,
    ) -> Result<HashMap<String, usize>, String> {
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

        let mut kind_handlers: HashMap<&str, DefinitionHandler> = HashMap::new(); // Use DefinitionHandler here
        kind_handlers.insert("variable_declarator", Self::collect_variable_definitions);
        kind_handlers.insert("arrow_function", Self::collect_function_definitions);
        kind_handlers.insert("function", Self::collect_function_definitions);
        kind_handlers.insert("function_declaration", Self::collect_function_definitions);
        kind_handlers.insert("class_declaration", Self::collect_type_definitions);
        kind_handlers.insert("interface_declaration", Self::collect_type_definitions);
        kind_handlers.insert("type_alias_declaration", Self::collect_type_definitions);
        kind_handlers.insert("enum_declaration", Self::collect_type_definitions);
        kind_handlers.insert("import_statement", Self::collect_import_definitions);

        Self::collect_definitions_recursive(
            tree.root_node(),
            content,
            &mut definitions,
            &kind_handlers,
        );
        Ok(definitions)
    }

    fn collect_variable_definitions(
        node: Node,
        source_code: &str,
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
            find_identifiers_in_pattern(pattern_node, source_code, definitions);
        }
    }

    fn collect_function_definitions(
        node: Node,
        source_code: &str,
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
                        find_identifiers_in_pattern(pattern_node, source_code, definitions);
                    }
                }
            }
        }
    }

    fn collect_type_definitions(
        node: Node,
        source_code: &str,
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
            find_identifiers_in_pattern(pattern_node, source_code, definitions);
        }
    }

    fn collect_import_definitions(
        node: Node,
        source_code: &str,
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
}
