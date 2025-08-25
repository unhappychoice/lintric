use crate::definition_collectors::{find_identifier_nodes_in_node, DefinitionCollector};
use crate::models::{
    Accessibility, Definition, DefinitionType, Position, ScopeId, ScopeType, SymbolTable,
};
use tree_sitter::{Node, Query, QueryCursor};

pub struct RustDefinitionCollector<'a> {
    source_code: &'a str,
}

struct RustCollector<'a> {
    source_code: &'a str,
    symbol_table: SymbolTable,
    current_scope: ScopeId,
}

impl<'a> RustCollector<'a> {
    fn new(source_code: &'a str) -> Self {
        let symbol_table = SymbolTable::new();
        let current_scope = symbol_table.scopes.root;

        Self {
            source_code,
            symbol_table,
            current_scope,
        }
    }

    fn collect_from_node(&mut self, root: Node<'a>) -> Result<(), String> {
        self.visit_node(root)?;
        Ok(())
    }

    fn visit_node(&mut self, node: Node<'a>) -> Result<(), String> {
        let node_type = node.kind();
        let original_scope = self.current_scope;

        // Handle scope creation and definition collection
        self.visit_rust_node(node, node_type)?;

        // Visit children
        for i in 0..node.child_count() {
            if let Some(child) = node.child(i) {
                self.visit_node(child)?;
            }
        }

        // Restore original scope
        if matches!(
            node_type,
            "function_item"
                | "impl_item"
                | "trait_item"
                | "block"
                | "mod_item"
                | "closure_expression"
                | "for_expression"
                | "while_expression"
                | "if_expression"
                | "match_expression"
        ) {
            self.current_scope = original_scope;
        }

        Ok(())
    }

    fn visit_rust_node(&mut self, node: Node<'a>, node_type: &str) -> Result<(), String> {
        match node_type {
            "function_item" => {
                self.handle_function_scope(node)?;
            }
            "impl_item" => {
                self.handle_impl_scope(node)?;
            }
            "trait_item" => {
                self.handle_trait_scope(node)?;
            }
            "block" => {
                self.handle_block_scope(node)?;
            }
            "mod_item" => {
                self.handle_module_scope(node)?;
            }
            "closure_expression" => {
                self.handle_closure_scope(node)?;
            }
            "for_expression" | "while_expression" | "if_expression" => {
                self.handle_control_flow_scope(node)?;
            }
            "match_expression" => {
                self.handle_match_scope(node)?;
            }
            _ => {
                // Collect definitions without creating scope
                self.collect_definitions_for_node(node);
            }
        }

        Ok(())
    }

    fn handle_function_scope(&mut self, node: Node<'a>) -> Result<(), String> {
        let position = Position::from_node(&node);
        let scope_id = self.symbol_table.scopes.create_scope(
            Some(self.current_scope),
            ScopeType::Function,
            position,
        );
        self.current_scope = scope_id;

        // Collect function definition and parameters
        self.collect_function_definitions(node, scope_id);
        Ok(())
    }

    fn handle_impl_scope(&mut self, node: Node<'a>) -> Result<(), String> {
        let position = Position::from_node(&node);
        let scope_id = self.symbol_table.scopes.create_scope(
            Some(self.current_scope),
            ScopeType::Impl,
            position,
        );
        self.current_scope = scope_id;

        self.collect_impl_definitions(node, scope_id);
        Ok(())
    }

    fn handle_trait_scope(&mut self, node: Node<'a>) -> Result<(), String> {
        let position = Position::from_node(&node);
        let scope_id = self.symbol_table.scopes.create_scope(
            Some(self.current_scope),
            ScopeType::Trait,
            position,
        );
        self.current_scope = scope_id;

        self.collect_trait_definitions(node, scope_id);
        Ok(())
    }

    fn handle_block_scope(&mut self, node: Node<'a>) -> Result<(), String> {
        let position = Position::from_node(&node);
        let scope_id = self.symbol_table.scopes.create_scope(
            Some(self.current_scope),
            ScopeType::Block,
            position,
        );
        self.current_scope = scope_id;
        Ok(())
    }

    fn handle_module_scope(&mut self, node: Node<'a>) -> Result<(), String> {
        let position = Position::from_node(&node);
        let scope_id = self.symbol_table.scopes.create_scope(
            Some(self.current_scope),
            ScopeType::Module,
            position,
        );
        self.current_scope = scope_id;

        self.collect_module_definitions(node, scope_id);
        Ok(())
    }

    fn handle_closure_scope(&mut self, node: Node<'a>) -> Result<(), String> {
        let position = Position::from_node(&node);
        let scope_id = self.symbol_table.scopes.create_scope(
            Some(self.current_scope),
            ScopeType::Closure,
            position,
        );
        self.current_scope = scope_id;

        self.collect_closure_definitions(node, scope_id);
        Ok(())
    }

    fn handle_control_flow_scope(&mut self, node: Node<'a>) -> Result<(), String> {
        let position = Position::from_node(&node);
        let scope_id = self.symbol_table.scopes.create_scope(
            Some(self.current_scope),
            ScopeType::Block,
            position,
        );
        self.current_scope = scope_id;

        self.collect_control_flow_definitions(node, scope_id);
        Ok(())
    }

    fn handle_match_scope(&mut self, node: Node<'a>) -> Result<(), String> {
        let position = Position::from_node(&node);
        let scope_id = self.symbol_table.scopes.create_scope(
            Some(self.current_scope),
            ScopeType::Block,
            position,
        );
        self.current_scope = scope_id;

        self.collect_match_definitions(node, scope_id);
        Ok(())
    }

    fn collect_definitions_for_node(&mut self, node: Node<'a>) {
        let definitions = match node.kind() {
            "let_declaration" => self.collect_let_definitions(node),
            "const_item" | "static_item" => self.collect_const_definitions(node),
            "struct_item" | "enum_item" | "type_item" => self.collect_type_definitions(node),
            "use_declaration" => self.collect_import_definitions(node),
            "macro_definition" => self.collect_macro_definitions(node),
            "function_signature_item" => {
                // Only collect function signatures that are NOT inside trait scopes
                // (trait function signatures are already collected by collect_trait_definitions)
                let current_scope = self.symbol_table.scopes.get_scope(self.current_scope);
                if let Some(scope) = current_scope {
                    if scope.scope_type == crate::models::ScopeType::Trait {
                        vec![] // Skip - already handled by trait processing
                    } else {
                        self.collect_function_signature_definitions(node)
                    }
                } else {
                    self.collect_function_signature_definitions(node)
                }
            }
            _ => vec![],
        };

        for mut definition in definitions {
            definition.set_context(self.current_scope, &Accessibility::ScopeLocal, false);
            self.symbol_table
                .add_enhanced_symbol(definition.name.clone(), definition);
        }
    }

    fn is_in_impl_scope(&self) -> bool {
        // Check if current scope or any parent scope is an Impl scope
        let mut scope_id = self.current_scope;
        loop {
            if let Some(scope) = self.symbol_table.scopes.get_scope(scope_id) {
                if scope.scope_type == crate::models::ScopeType::Impl {
                    return true;
                }
                if let Some(parent_id) = scope.parent {
                    scope_id = parent_id;
                } else {
                    break;
                }
            } else {
                break;
            }
        }
        false
    }

    fn collect_function_definitions(&mut self, node: Node<'a>, scope_id: ScopeId) {
        // Skip if this function is inside an impl block (it will be handled as MethodDefinition)
        if self.is_in_impl_scope() {
            return;
        }

        let mut definitions = vec![];

        // Function name
        if let Some(def) = Definition::from_naming_node(
            &node,
            self.source_code,
            DefinitionType::FunctionDefinition,
        ) {
            definitions.push(def);
        }

        // Type parameters
        if let Some(type_params_node) = node.child_by_field_name("type_parameters") {
            definitions.extend(self.collect_type_parameters(type_params_node));
        }

        // Parameters
        if let Some(params_node) = node.child_by_field_name("parameters") {
            definitions.extend(self.collect_function_parameters(params_node));
        }

        for mut def in definitions {
            def.set_context(scope_id, &Accessibility::ScopeLocal, false);
            self.symbol_table.add_enhanced_symbol(def.name.clone(), def);
        }
    }

    fn collect_impl_definitions(&mut self, node: Node<'a>, scope_id: ScopeId) {
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "declaration_list" {
                let mut inner_cursor = child.walk();
                for declaration in child.children(&mut inner_cursor) {
                    if declaration.kind() == "function_item" {
                        // Collect method definition
                        if let Some(mut def) = Definition::from_naming_node(
                            &declaration,
                            self.source_code,
                            DefinitionType::MethodDefinition,
                        ) {
                            def.set_context(scope_id, &Accessibility::ScopeLocal, false);
                            self.symbol_table.add_enhanced_symbol(def.name.clone(), def);
                        }

                        // Collect method parameters
                        if let Some(params_node) = declaration.child_by_field_name("parameters") {
                            let param_definitions = self.collect_function_parameters(params_node);
                            for mut param_def in param_definitions {
                                param_def.set_context(scope_id, &Accessibility::ScopeLocal, false);
                                self.symbol_table
                                    .add_enhanced_symbol(param_def.name.clone(), param_def);
                            }
                        }
                    }
                }
            }
        }
    }

    fn collect_trait_definitions(&mut self, node: Node<'a>, scope_id: ScopeId) {
        if let Some(mut def) =
            Definition::from_naming_node(&node, self.source_code, DefinitionType::TypeDefinition)
        {
            def.set_context(scope_id, &Accessibility::ScopeLocal, false);
            self.symbol_table.add_enhanced_symbol(def.name.clone(), def);
        }

        // Collect trait methods and associated types
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "declaration_list" {
                // Process items inside the declaration list
                let mut decl_cursor = child.walk();
                for decl_child in child.children(&mut decl_cursor) {
                    match decl_child.kind() {
                        "function_signature_item" => {
                            if let Some(mut def) = Definition::from_naming_node(
                                &decl_child,
                                self.source_code,
                                DefinitionType::FunctionDefinition,
                            ) {
                                def.set_context(scope_id, &Accessibility::ScopeLocal, false);
                                self.symbol_table.add_enhanced_symbol(def.name.clone(), def);
                            }
                        }
                        "associated_type" => {
                            if let Some(mut def) = Definition::from_naming_node(
                                &decl_child,
                                self.source_code,
                                DefinitionType::TypeDefinition,
                            ) {
                                def.set_context(scope_id, &Accessibility::ScopeLocal, false);
                                self.symbol_table.add_enhanced_symbol(def.name.clone(), def);
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    fn collect_function_signature_definitions(&self, node: Node<'a>) -> Vec<Definition> {
        if let Some(def) = Definition::from_naming_node(
            &node,
            self.source_code,
            DefinitionType::FunctionDefinition,
        ) {
            vec![def]
        } else {
            vec![]
        }
    }

    fn collect_module_definitions(&mut self, node: Node<'a>, scope_id: ScopeId) {
        if let Some(mut def) =
            Definition::from_naming_node(&node, self.source_code, DefinitionType::ModuleDefinition)
        {
            def.set_context(scope_id, &Accessibility::ScopeLocal, false);
            self.symbol_table.add_enhanced_symbol(def.name.clone(), def);
        }
    }

    fn collect_closure_definitions(&mut self, node: Node<'a>, scope_id: ScopeId) {
        if let Some(params_node) = node.child_by_field_name("parameters") {
            let definitions = self.collect_closure_parameters(params_node);
            for mut def in definitions {
                def.set_context(scope_id, &Accessibility::ScopeLocal, false);
                self.symbol_table.add_enhanced_symbol(def.name.clone(), def);
            }
        }
    }

    fn collect_control_flow_definitions(&mut self, node: Node<'a>, scope_id: ScopeId) {
        let definitions = match node.kind() {
            "for_expression" => {
                let mut defs = vec![];
                if let Some(pattern_node) = node.child_by_field_name("pattern") {
                    defs.extend(find_identifier_nodes_in_node(pattern_node).iter().map(|n| {
                        Definition::new(n, self.source_code, DefinitionType::VariableDefinition)
                    }));
                }
                defs
            }
            "if_expression" | "while_expression" => self.collect_condition_definitions(node),
            _ => vec![],
        };

        for mut def in definitions {
            def.set_context(scope_id, &Accessibility::ScopeLocal, false);
            self.symbol_table.add_enhanced_symbol(def.name.clone(), def);
        }
    }

    fn collect_match_definitions(&mut self, node: Node<'a>, scope_id: ScopeId) {
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "match_block" {
                let mut match_cursor = child.walk();
                for match_child in child.children(&mut match_cursor) {
                    if match_child.kind() == "match_arm" {
                        let definitions = self.collect_match_pattern_definitions(match_child);
                        for mut def in definitions {
                            def.set_context(scope_id, &Accessibility::ScopeLocal, false);
                            self.symbol_table.add_enhanced_symbol(def.name.clone(), def);
                        }
                    }
                }
            }
        }
    }

    // Helper methods from original RustDefinitionCollector
    fn collect_let_definitions(&self, node: Node<'a>) -> Vec<Definition> {
        if let Some(pattern_node) = node.child_by_field_name("pattern") {
            find_identifier_nodes_in_node(pattern_node)
                .iter()
                .map(|node| {
                    Definition::new(node, self.source_code, DefinitionType::VariableDefinition)
                })
                .collect()
        } else {
            vec![]
        }
    }

    fn collect_const_definitions(&self, node: Node<'a>) -> Vec<Definition> {
        Definition::from_naming_node(&node, self.source_code, DefinitionType::ConstDefinition)
            .into_iter()
            .collect()
    }

    fn collect_type_definitions(&self, node: Node<'a>) -> Vec<Definition> {
        let definition_type = match node.kind() {
            "struct_item" => DefinitionType::StructDefinition,
            "enum_item" => DefinitionType::EnumDefinition,
            "type_item" => DefinitionType::TypeDefinition,
            _ => DefinitionType::Other("unknown".to_string()),
        };

        let mut definitions =
            Definition::from_naming_node(&node, self.source_code, definition_type)
                .into_iter()
                .collect::<Vec<_>>();

        if let Some(type_params_node) = node.child_by_field_name("type_parameters") {
            definitions.extend(self.collect_type_parameters(type_params_node));
        }

        if node.kind() == "struct_item" {
            if let Some(body_node) = node.child_by_field_name("body") {
                definitions.extend(self.collect_struct_field_definitions(body_node));
            }
        }

        definitions
    }

    fn collect_import_definitions(&self, node: Node<'a>) -> Vec<Definition> {
        let mut definitions = vec![];

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

        let import_query_str = "(use_declaration argument: (scoped_use_list path: (_) list: (use_list (_)*)) @import) @use_decl";
        if let Ok(query) = Query::new(&tree_sitter_rust::language(), import_query_str) {
            let mut cursor = QueryCursor::new();
            for query_match in cursor.matches(&query, node, self.source_code.as_bytes()) {
                for capture in query_match.captures {
                    definitions.extend(self.collect_use_list_items(capture.node));
                }
            }
        }

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

    fn collect_macro_definitions(&self, node: Node<'a>) -> Vec<Definition> {
        let mut definitions = vec![];

        definitions.extend(Definition::from_naming_node(
            &node,
            self.source_code,
            DefinitionType::MacroDefinition,
        ));

        definitions.extend(self.collect_macro_metavariables(node));
        definitions
    }

    fn collect_condition_definitions(&self, node: Node<'a>) -> Vec<Definition> {
        let mut definitions = vec![];
        if let Some(condition_node) = node.child_by_field_name("condition") {
            if condition_node.kind() == "let_condition" {
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

    fn collect_match_pattern_definitions(&self, node: Node<'a>) -> Vec<Definition> {
        let mut definitions = vec![];
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "match_pattern" {
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
                if let Some(first_child) = node.child(0) {
                    if first_child.kind() == "identifier" {
                        definitions.push(Definition::new(
                            &first_child,
                            self.source_code,
                            DefinitionType::VariableDefinition,
                        ));
                    }
                }
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
                    if !matches!(child.kind(), "(" | ")" | "[" | "]") {
                        definitions.extend(self.collect_pattern_definitions(child));
                    }
                }
                definitions
            }
            "tuple_struct_pattern" => {
                let mut definitions = vec![];
                let mut cursor = node.walk();
                for child in node.children(&mut cursor) {
                    if child.kind() == "identifier" {
                        if child.start_position().column
                            > node.child(0).unwrap().end_position().column
                        {
                            definitions.push(Definition::new(
                                &child,
                                self.source_code,
                                DefinitionType::VariableDefinition,
                            ));
                        }
                    } else if !matches!(child.kind(), "(" | ")") {
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
                for child in node.children(&mut cursor) {
                    definitions.extend(self.collect_metavariables_from_node(child));
                }
            }
        }

        definitions
    }

    fn collect_struct_field_definitions(&self, node: Node<'a>) -> Vec<Definition> {
        let mut definitions = vec![];

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

impl<'a> RustDefinitionCollector<'a> {
    pub fn new(source_code: &'a str) -> Self {
        Self { source_code }
    }
}

impl<'a> DefinitionCollector<'a> for RustDefinitionCollector<'a> {
    fn collect(&self, source_code: &str, root: Node<'a>) -> Result<SymbolTable, String> {
        let mut collector = RustCollector::new(source_code);
        collector.collect_from_node(root)?;
        Ok(collector.symbol_table)
    }
}
