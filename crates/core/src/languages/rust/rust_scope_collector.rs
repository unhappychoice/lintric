use tree_sitter::Node;

use super::dependency_resolver::nested_scope_resolver::NestedScopeResolver;
use crate::models::{
    Accessibility, Definition, DefinitionType, Position, ScopeId, ScopeTree, ScopeType, SymbolTable,
};
use crate::scope_collector::ScopeCollector as ScopeCollectorTrait;

pub struct RustScopeCollector {
    pub scope_tree: ScopeTree,
    current_scope: ScopeId,
    symbol_table: SymbolTable,
}

impl Default for RustScopeCollector {
    fn default() -> Self {
        Self::new()
    }
}

impl RustScopeCollector {
    pub fn new() -> Self {
        let scope_tree = ScopeTree::new();
        let current_scope = scope_tree.root;

        Self {
            scope_tree,
            current_scope,
            symbol_table: SymbolTable::new(),
        }
    }

    fn visit_node(&mut self, node: Node, source_code: &str) -> Result<(), String> {
        let node_type = node.kind();

        self.visit_rust_node(node, source_code, node_type)?;

        for i in 0..node.child_count() {
            if let Some(child) = node.child(i) {
                self.visit_node(child, source_code)?;
            }
        }

        Ok(())
    }

    fn visit_rust_node(
        &mut self,
        node: Node,
        source_code: &str,
        node_type: &str,
    ) -> Result<(), String> {
        match node_type {
            "function_item" | "impl_item" | "trait_item" => {
                let scope_type = match node_type {
                    "function_item" => ScopeType::Function,
                    "impl_item" => ScopeType::Impl,
                    "trait_item" => ScopeType::Trait,
                    _ => ScopeType::Function,
                };
                self.enter_scope_for_node(node, scope_type, source_code)?;
            }
            "block" => {
                self.enter_scope_for_node(node, ScopeType::Block, source_code)?;
            }
            "mod_item" => {
                self.enter_scope_for_node(node, ScopeType::Module, source_code)?;
            }
            _ => {}
        }

        Ok(())
    }

    fn enter_scope_for_node(
        &mut self,
        node: Node,
        scope_type: ScopeType,
        source_code: &str,
    ) -> Result<(), String> {
        let start_position = self.extract_position_from_node(node, source_code)?;
        let end_position = self.extract_end_position_from_node(node, source_code)?;

        let scope_id = self.scope_tree.create_scope(
            Some(self.current_scope),
            scope_type,
            start_position,
            end_position,
        );

        self.current_scope = scope_id;
        Ok(())
    }

    fn extract_position_from_node(
        &self,
        node: Node,
        _source_code: &str,
    ) -> Result<Position, String> {
        Ok(Position::from_node(&node))
    }

    fn extract_end_position_from_node(
        &self,
        node: Node,
        _source_code: &str,
    ) -> Result<Position, String> {
        Ok(Position::from_node(&node))
    }

    fn collect_definitions_with_scope(
        &mut self,
        node: Node,
        source_code: &str,
    ) -> Result<(), String> {
        let node_type = node.kind();
        let position = Position::from_node(&node);

        if let Some(scope_id) = self.symbol_table.scopes.find_scope_at_position(&position) {
            if node_type == "identifier" && self.is_definition_context(node) {
                if let Ok(name) = node.utf8_text(source_code.as_bytes()) {
                    let definition = Definition::new_simple(
                        name.to_string(),
                        self.infer_definition_type(node),
                        position,
                    );

                    self.symbol_table.add_symbol(
                        name.to_string(),
                        definition,
                        scope_id,
                        Accessibility::ScopeLocal,
                        false,
                    );
                }
            }
        }

        for i in 0..node.child_count() {
            if let Some(child) = node.child(i) {
                self.collect_definitions_with_scope(child, source_code)?;
            }
        }

        Ok(())
    }

    fn is_definition_context(&self, node: Node) -> bool {
        if let Some(parent) = node.parent() {
            match parent.kind() {
                "let_declaration" => {
                    if let Some(pattern_node) = parent.child_by_field_name("pattern") {
                        self.is_node_within(node, pattern_node)
                    } else {
                        false
                    }
                }
                "variable_declaration" | "function_item" | "function_declaration" | "mod_item" => {
                    true
                }
                "use_declaration" => self.is_imported_identifier(node),
                _ => false,
            }
        } else {
            false
        }
    }

    fn is_node_within(&self, child: Node, parent: Node) -> bool {
        let child_start = child.start_position().row;
        let child_end = child.end_position().row;
        let parent_start = parent.start_position().row;
        let parent_end = parent.end_position().row;

        child_start >= parent_start && child_end <= parent_end
    }

    fn is_imported_identifier(&self, node: Node) -> bool {
        if let Some(parent) = node.parent() {
            match parent.kind() {
                "scoped_identifier" => node == parent.child_by_field_name("name").unwrap_or(node),
                "use_list" => true,
                "use_as_clause" => node == parent.child_by_field_name("alias").unwrap_or(node),
                _ => false,
            }
        } else {
            false
        }
    }

    fn infer_definition_type(&self, node: Node) -> DefinitionType {
        if let Some(parent) = node.parent() {
            match parent.kind() {
                "function_item" | "function_declaration" => DefinitionType::FunctionDefinition,
                "let_declaration" | "variable_declaration" => DefinitionType::VariableDefinition,
                "struct_item" | "class_declaration" => DefinitionType::TypeDefinition,
                "mod_item" => DefinitionType::ModuleDefinition,
                _ => DefinitionType::VariableDefinition,
            }
        } else {
            DefinitionType::VariableDefinition
        }
    }

    pub fn analyze_complex_nested_structures_with_captures(
        &self,
        source_code: &str,
        root_node: Node,
    ) -> Result<
        (
            SymbolTable,
            std::collections::HashMap<
                crate::models::ScopeId,
                Vec<
                    crate::languages::rust::dependency_resolver::nested_scope_resolver::CaptureInfo,
                >,
            >,
        ),
        String,
    > {
        let scope_resolver = RustScopeCollector::new();
        let symbol_table = scope_resolver.collect(source_code, root_node, &[], &[])?;

        let mut nested_resolver = NestedScopeResolver::new(symbol_table.scopes.clone());
        let complex_analysis = nested_resolver.analyze_complex_nesting(0); // Start from global scope

        Ok((symbol_table, complex_analysis))
    }
}

impl ScopeCollectorTrait for RustScopeCollector {
    fn scopes(&mut self, root_node: Node, source_code: &str) -> Result<ScopeTree, String> {
        self.visit_node(root_node, source_code)?;
        Ok(self.scope_tree.clone())
    }

    fn collect(
        &self,
        source_code: &str,
        root_node: Node,
        _usage_nodes: &[crate::models::Usage],
        _definitions: &[crate::models::Definition],
    ) -> Result<SymbolTable, String> {
        let mut new_self = RustScopeCollector::new();
        new_self.visit_node(root_node, source_code)?;
        new_self.symbol_table.scopes = new_self.scope_tree.clone();
        new_self.collect_definitions_with_scope(root_node, source_code)?;
        Ok(new_self.symbol_table.clone())
    }
}
