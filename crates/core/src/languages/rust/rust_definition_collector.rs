use crate::definition_collectors::{find_identifier_nodes_in_node, DefinitionCollector};
use crate::models::{Definition, DefinitionType};
use tree_sitter::{Node, Query, QueryCursor};

pub struct RustDefinitionCollector<'a> {
    source_code: &'a str,
}

impl<'a> RustDefinitionCollector<'a> {
    pub fn new(source_code: &'a str) -> Self {
        Self { source_code }
    }
}

impl<'a> DefinitionCollector<'a> for RustDefinitionCollector<'a> {
    fn process_node(&self, node: Node<'a>) -> Vec<Definition> {
        match node.kind() {
            "function_item" | "function_signature_item" => self.collect_function_definitions(node),
            "let_declaration" | "for_expression" | "if_expression" | "while_expression" => {
                self.collect_variable_definitions(node)
            }
            "struct_item" | "enum_item" | "type_item" | "trait_item" | "impl_item" | "mod_item" => {
                self.collect_type_definitions(node)
            }
            "use_declaration" => self.collect_import_definitions(node),
            "closure_expression" => self.collect_closure_definitions(node),
            "const_item" | "static_item" => self.collect_variable_definitions(node),
            "macro_definition" => self.collect_macro_definitions(node),
            "match_expression" => self.collect_match_definitions(node),
            _ => Vec::new(),
        }
    }

    fn collect_variable_definitions(&self, node: Node<'a>) -> Vec<Definition> {
        match node.kind() {
            "let_declaration" => {
                if let Some(pattern_node) = node.child_by_field_name("pattern") {
                    find_identifier_nodes_in_node(pattern_node)
                        .iter()
                        .map(|node| {
                            Definition::new(
                                node,
                                self.source_code,
                                DefinitionType::VariableDefinition,
                            )
                        })
                        .collect()
                } else {
                    vec![]
                }
            }
            "for_expression" => {
                if let Some(pattern_node) = node.child_by_field_name("pattern") {
                    find_identifier_nodes_in_node(pattern_node)
                        .iter()
                        .map(|node| {
                            Definition::new(
                                node,
                                self.source_code,
                                DefinitionType::VariableDefinition,
                            )
                        })
                        .collect()
                } else {
                    vec![]
                }
            }
            "if_expression" | "while_expression" => {
                let mut definitions = vec![];
                if let Some(condition_node) = node.child_by_field_name("condition") {
                    // Handle let_condition patterns
                    if condition_node.kind() == "let_condition" {
                        // Find the pattern node (usually the first child that is a pattern)
                        let mut cursor = condition_node.walk();
                        for child in condition_node.children(&mut cursor) {
                            if matches!(
                                child.kind(),
                                "tuple_struct_pattern"
                                    | "tuple_pattern"
                                    | "struct_pattern"
                                    | "identifier"
                                    | "captured_pattern"
                            ) {
                                definitions.extend(self.collect_pattern_definitions(child));
                                break;
                            }
                        }
                    } else {
                        // Handle let_declaration in condition
                        let query_str = "(let_declaration) @let";
                        if let Ok(query) = Query::new(&tree_sitter_rust::language(), query_str) {
                            let mut cursor = QueryCursor::new();
                            for query_match in
                                cursor.matches(&query, condition_node, self.source_code.as_bytes())
                            {
                                for capture in query_match.captures {
                                    let nodes = find_identifier_nodes_in_node(capture.node)
                                        .into_iter()
                                        .map(|n| {
                                            Definition::new(
                                                &n,
                                                self.source_code,
                                                DefinitionType::VariableDefinition,
                                            )
                                        });
                                    definitions.extend(nodes);
                                }
                            }
                        }
                    }
                }
                definitions
            }
            "const_item" | "static_item" => Definition::from_naming_node(
                &node,
                self.source_code,
                DefinitionType::ConstDefinition,
            )
            .into_iter()
            .collect(),
            _ => vec![],
        }
    }

    fn collect_function_definitions(&self, node: Node<'a>) -> Vec<Definition> {
        let mut definitions = vec![];

        definitions.extend(Definition::from_naming_node(
            &node,
            self.source_code,
            DefinitionType::FunctionDefinition,
        ));

        if let Some(type_params_node) = node.child_by_field_name("type_parameters") {
            definitions.extend(self.collect_type_parameters(type_params_node));
        }

        if let Some(params_node) = node.child_by_field_name("parameters") {
            definitions.extend(self.collect_function_parameters(params_node));
        }

        definitions
    }

    fn collect_type_definitions(&self, node: Node<'a>) -> Vec<Definition> {
        let definition_type = match node.kind() {
            "struct_item" => DefinitionType::StructDefinition,
            "enum_item" => DefinitionType::EnumDefinition,
            "type_item" => DefinitionType::TypeDefinition,
            "trait_item" => DefinitionType::TypeDefinition,
            "impl_item" => return self.collect_impl_definitions(node),
            "mod_item" => DefinitionType::ModuleDefinition,
            _ => DefinitionType::Other("unknown".to_string()),
        };

        let mut definitions =
            Definition::from_naming_node(&node, self.source_code, definition_type)
                .into_iter()
                .collect::<Vec<_>>();

        if let Some(type_params_node) = node.child_by_field_name("type_parameters") {
            definitions.extend(self.collect_type_parameters(type_params_node));
        }

        // Collect struct field definitions if this is a struct
        if node.kind() == "struct_item" {
            if let Some(body_node) = node.child_by_field_name("body") {
                definitions.extend(self.collect_struct_field_definitions(body_node));
            }
        }

        definitions
    }

    fn collect_import_definitions(&self, node: Node<'a>) -> Vec<Definition> {
        let mut definitions = vec![];

        // Handle use_as_clause with aliases
        let alias_query_str = "(use_as_clause alias: (identifier) @alias) @use_as";
        if let Ok(query) = Query::new(&tree_sitter_rust::language(), alias_query_str) {
            let mut cursor = QueryCursor::new();
            for query_match in cursor.matches(&query, node, self.source_code.as_bytes()) {
                for capture in query_match.captures {
                    if capture.index == 0 {
                        definitions.push(Definition::new(
                            &capture.node,
                            self.source_code,
                            DefinitionType::ImportDefinition,
                        ));
                    }
                }
            }
        }

        // Handle regular imports like `use module::item;`
        let import_query_str = "(use_declaration argument: (scoped_use_list path: (_) list: (use_list (_)*)) @import) @use_decl";
        if let Ok(query) = Query::new(&tree_sitter_rust::language(), import_query_str) {
            let mut cursor = QueryCursor::new();
            for query_match in cursor.matches(&query, node, self.source_code.as_bytes()) {
                for capture in query_match.captures {
                    definitions.extend(self.collect_use_list_items(capture.node));
                }
            }
        }

        // Handle simple use like `use module::item;`
        let simple_query_str = "(use_declaration argument: (scoped_identifier name: (identifier) @imported_name)) @simple_use";
        if let Ok(query) = Query::new(&tree_sitter_rust::language(), simple_query_str) {
            let mut cursor = QueryCursor::new();
            for query_match in cursor.matches(&query, node, self.source_code.as_bytes()) {
                for capture in query_match.captures {
                    if capture.index == 0 {
                        definitions.push(Definition::new(
                            &capture.node,
                            self.source_code,
                            DefinitionType::ImportDefinition,
                        ));
                    }
                }
            }
        }

        definitions
    }

    fn collect_closure_definitions(&self, node: Node<'a>) -> Vec<Definition> {
        let mut definitions = vec![];
        if let Some(params_node) = node.child_by_field_name("parameters") {
            definitions.extend(self.collect_closure_parameters(params_node));
        }
        definitions
    }

    fn collect_macro_definitions(&self, node: Node<'a>) -> Vec<Definition> {
        let mut definitions = vec![];

        // Collect the macro name itself
        definitions.extend(Definition::from_naming_node(
            &node,
            self.source_code,
            DefinitionType::MacroDefinition,
        ));

        // Collect metavariables from macro rules
        definitions.extend(self.collect_macro_metavariables(node));

        definitions
    }
}

impl<'a> RustDefinitionCollector<'a> {
    fn collect_match_definitions(&self, node: Node<'a>) -> Vec<Definition> {
        let mut definitions = vec![];
        // Look for match_block and then match_arms
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "match_block" {
                let mut match_cursor = child.walk();
                for match_child in child.children(&mut match_cursor) {
                    if match_child.kind() == "match_arm" {
                        definitions.extend(self.collect_match_pattern_definitions(match_child));
                    }
                }
            }
        }
        definitions
    }

    fn collect_match_pattern_definitions(&self, node: Node<'a>) -> Vec<Definition> {
        let mut definitions = vec![];
        // Look for match_pattern child in match_arm
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "match_pattern" {
                // Process all children of match_pattern
                let mut pattern_cursor = child.walk();
                for pattern_child in child.children(&mut pattern_cursor) {
                    definitions.extend(self.collect_pattern_definitions(pattern_child));
                }
            }
        }
        definitions
    }

    fn collect_pattern_definitions(&self, node: Node<'a>) -> Vec<Definition> {
        match node.kind() {
            "identifier" => {
                // In match patterns, standalone identifiers can be either:
                // 1. Variable bindings (definitions) - lowercase identifiers
                // 2. Constructor references (usages) - uppercase identifiers
                let name = node.utf8_text(self.source_code.as_bytes()).unwrap_or("");
                if name
                    .chars()
                    .next()
                    .is_some_and(|c| c.is_lowercase() || c == '_')
                {
                    vec![Definition::new(
                        &node,
                        self.source_code,
                        DefinitionType::VariableDefinition,
                    )]
                } else {
                    vec![]
                }
            }
            "captured_pattern" => {
                let mut definitions = vec![];
                // First child is usually the binding identifier
                if let Some(first_child) = node.child(0) {
                    if first_child.kind() == "identifier" {
                        definitions.push(Definition::new(
                            &first_child,
                            self.source_code,
                            DefinitionType::VariableDefinition,
                        ));
                    }
                }
                // Also collect from the rest of the pattern
                let mut cursor = node.walk();
                for child in node.children(&mut cursor).skip(1) {
                    definitions.extend(self.collect_pattern_definitions(child));
                }
                definitions
            }
            "tuple_pattern" | "struct_pattern" | "slice_pattern" => {
                let mut definitions = vec![];
                let mut cursor = node.walk();
                for child in node.children(&mut cursor) {
                    if child.kind() != "("
                        && child.kind() != ")"
                        && child.kind() != "["
                        && child.kind() != "]"
                    {
                        definitions.extend(self.collect_pattern_definitions(child));
                    }
                }
                definitions
            }
            "tuple_struct_pattern" => {
                let mut definitions = vec![];
                let mut cursor = node.walk();
                for child in node.children(&mut cursor) {
                    // Skip the first identifier (constructor name) and punctuation
                    if child.kind() == "identifier" {
                        // Check if this is not the first child (constructor name)
                        if child.start_position().column
                            > node.child(0).unwrap().end_position().column
                        {
                            definitions.push(Definition::new(
                                &child,
                                self.source_code,
                                DefinitionType::VariableDefinition,
                            ));
                        }
                    } else if child.kind() != "(" && child.kind() != ")" {
                        definitions.extend(self.collect_pattern_definitions(child));
                    }
                }
                definitions
            }
            _ => vec![],
        }
    }

    fn collect_type_parameters(&self, node: Node<'a>) -> Vec<Definition> {
        let mut definitions = vec![];
        let mut cursor = node.walk();

        for child in node.children(&mut cursor) {
            match child.kind() {
                "constrained_type_parameter" => {
                    // Only collect the first type_identifier as a type parameter definition
                    // The ones in trait_bounds are usages
                    if let Some(first_child) = child.child(0) {
                        if first_child.kind() == "type_identifier" {
                            definitions.push(Definition::new(
                                &first_child,
                                self.source_code,
                                DefinitionType::TypeDefinition,
                            ));
                        }
                    }
                }
                "type_parameter" => {
                    if let Some(type_id) = child.child_by_field_name("name") {
                        definitions.push(Definition::new(
                            &type_id,
                            self.source_code,
                            DefinitionType::TypeDefinition,
                        ));
                    }
                }
                _ => {
                    // For other simple type parameters
                    if child.kind() == "type_identifier" {
                        definitions.push(Definition::new(
                            &child,
                            self.source_code,
                            DefinitionType::TypeDefinition,
                        ));
                    }
                }
            }
        }
        definitions
    }

    fn collect_function_parameters(&self, node: Node<'a>) -> Vec<Definition> {
        let query_str = "(parameter pattern: (identifier) @param) @parameter";
        let mut definitions = vec![];
        if let Ok(query) = Query::new(&tree_sitter_rust::language(), query_str) {
            let mut cursor = QueryCursor::new();
            for query_match in cursor.matches(&query, node, self.source_code.as_bytes()) {
                for capture in query_match.captures {
                    if capture.index == 0 {
                        definitions.push(Definition::new(
                            &capture.node,
                            self.source_code,
                            DefinitionType::VariableDefinition,
                        ));
                    }
                }
            }
        }
        definitions
    }

    fn collect_impl_definitions(&self, node: Node<'a>) -> Vec<Definition> {
        let mut definitions = vec![];
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "declaration_list" {
                let mut inner_cursor = child.walk();
                for declaration in child.children(&mut inner_cursor) {
                    if declaration.kind() == "function_item" {
                        // Collect as method definition instead of function definition
                        definitions.extend(Definition::from_naming_node(
                            &declaration,
                            self.source_code,
                            DefinitionType::MethodDefinition,
                        ));

                        // Also collect function parameters
                        if let Some(params_node) = declaration.child_by_field_name("parameters") {
                            definitions.extend(self.collect_function_parameters(params_node));
                        }

                        // Collect type parameters if any
                        if let Some(type_params_node) =
                            declaration.child_by_field_name("type_parameters")
                        {
                            definitions.extend(self.collect_type_parameters(type_params_node));
                        }
                    }
                }
            }
        }
        definitions
    }

    fn collect_closure_parameters(&self, node: Node<'a>) -> Vec<Definition> {
        let mut definitions = vec![];
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "identifier" {
                definitions.push(Definition::new(
                    &child,
                    self.source_code,
                    DefinitionType::VariableDefinition,
                ));
            }
        }
        definitions
    }

    fn collect_use_list_items(&self, node: Node<'a>) -> Vec<Definition> {
        let mut definitions = vec![];
        let mut cursor = node.walk();

        for child in node.children(&mut cursor) {
            match child.kind() {
                "identifier" => {
                    definitions.push(Definition::new(
                        &child,
                        self.source_code,
                        DefinitionType::ImportDefinition,
                    ));
                }
                "use_list" => {
                    definitions.extend(self.collect_use_list_items(child));
                }
                _ => {}
            }
        }

        definitions
    }

    fn collect_macro_metavariables(&self, node: Node<'a>) -> Vec<Definition> {
        let mut definitions = vec![];
        let mut cursor = node.walk();

        // Recursively search for token_binding_pattern nodes
        for child in node.children(&mut cursor) {
            definitions.extend(self.collect_metavariables_from_node(child));
        }

        definitions
    }

    fn collect_metavariables_from_node(&self, node: Node<'a>) -> Vec<Definition> {
        let mut definitions = vec![];
        let mut cursor = node.walk();

        match node.kind() {
            "token_binding_pattern" => {
                // Look for metavariable child
                for child in node.children(&mut cursor) {
                    if child.kind() == "metavariable" {
                        definitions.push(Definition::new(
                            &child,
                            self.source_code,
                            DefinitionType::MacroVariableDefinition,
                        ));
                        break;
                    }
                }
            }
            _ => {
                // Recursively search in children
                for child in node.children(&mut cursor) {
                    definitions.extend(self.collect_metavariables_from_node(child));
                }
            }
        }

        definitions
    }

    fn collect_struct_field_definitions(&self, node: Node<'a>) -> Vec<Definition> {
        let mut definitions = vec![];

        // For field_declaration_list, look for field_declaration children
        if node.kind() == "field_declaration_list" {
            let mut cursor = node.walk();
            for child in node.children(&mut cursor) {
                if child.kind() == "field_declaration" {
                    if let Some(name_node) = child.child_by_field_name("name") {
                        definitions.push(Definition::new(
                            &name_node,
                            self.source_code,
                            DefinitionType::StructFieldDefinition,
                        ));
                    }
                }
            }
        }

        definitions
    }
}
