// use std::collections::HashMap; // Unused for now
use tree_sitter::Node;

use crate::models::{
    Accessibility, Definition, DefinitionType, Position, ScopeId, ScopeTree, ScopeType, SymbolTable,
};

pub struct ScopeBuilder {
    pub scope_tree: ScopeTree,
    current_scope: ScopeId,
    language: String,
}

impl ScopeBuilder {
    pub fn new(language: String) -> Self {
        let scope_tree = ScopeTree::new();
        let current_scope = scope_tree.root;

        Self {
            scope_tree,
            current_scope,
            language,
        }
    }

    pub fn build_from_ast(
        &mut self,
        root_node: Node,
        source_code: &str,
    ) -> Result<ScopeTree, String> {
        self.visit_node(root_node, source_code)?;
        Ok(self.scope_tree.clone())
    }

    pub fn build_symbol_table_from_ast(
        &mut self,
        root_node: Node,
        source_code: &str,
    ) -> Result<SymbolTable, String> {
        let mut symbol_table = SymbolTable::new();
        symbol_table.scopes = self.build_from_ast(root_node, source_code)?;
        Ok(symbol_table)
    }

    fn visit_node(&mut self, node: Node, source_code: &str) -> Result<(), String> {
        let node_type = node.kind();

        match self.language.as_str() {
            "rust" => self.visit_rust_node(node, source_code, node_type)?,
            "typescript" | "tsx" => self.visit_typescript_node(node, source_code, node_type)?,
            _ => return Err(format!("Unsupported language: {}", self.language)),
        }

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

    fn visit_typescript_node(
        &mut self,
        node: Node,
        source_code: &str,
        node_type: &str,
    ) -> Result<(), String> {
        match node_type {
            "function_declaration"
            | "function_expression"
            | "arrow_function"
            | "method_definition"
            | "generator_function_declaration" => {
                self.enter_scope_for_node(node, ScopeType::Function, source_code)?;
            }
            "class_declaration" => {
                self.enter_scope_for_node(node, ScopeType::Class, source_code)?;
            }
            "interface_declaration" => {
                self.enter_scope_for_node(node, ScopeType::Interface, source_code)?;
            }
            "statement_block" => {
                self.enter_scope_for_node(node, ScopeType::Block, source_code)?;
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
    ) -> Result<ScopeId, String> {
        let start_pos = self.node_position_start(node, source_code)?;
        let end_pos = self.node_position_end(node, source_code)?;

        let scope_id = self.enter_scope(scope_type, start_pos);

        self.exit_scope(end_pos);

        Ok(scope_id)
    }

    fn enter_scope(&mut self, scope_type: ScopeType, start_pos: Position) -> ScopeId {
        let parent_scope = Some(self.current_scope);
        let scope_id = self.scope_tree.create_scope(
            parent_scope,
            scope_type,
            start_pos,
            Position {
                start_line: usize::MAX,
                start_column: usize::MAX,
                end_line: usize::MAX,
                end_column: usize::MAX,
            }, // Will be updated in exit_scope
        );

        self.current_scope = scope_id;
        scope_id
    }

    fn exit_scope(&mut self, end_pos: Position) {
        if let Some(scope) = self.scope_tree.get_scope_mut(self.current_scope) {
            scope.end_position = end_pos;

            if let Some(parent_id) = scope.parent {
                self.current_scope = parent_id;
            }
        }
    }

    fn node_position_start(&self, node: Node, _source_code: &str) -> Result<Position, String> {
        Ok(Position::from_node(&node))
    }

    fn node_position_end(&self, node: Node, _source_code: &str) -> Result<Position, String> {
        Ok(Position::from_node(&node))
    }
}

pub struct ScopeAwareDefinitionCollector {
    scope_builder: ScopeBuilder,
    symbol_table: SymbolTable,
}

impl ScopeAwareDefinitionCollector {
    pub fn new(language: String) -> Self {
        Self {
            scope_builder: ScopeBuilder::new(language),
            symbol_table: SymbolTable::new(),
        }
    }

    pub fn collect_with_scopes(
        &mut self,
        root_node: Node,
        source_code: &str,
    ) -> Result<SymbolTable, String> {
        self.symbol_table = self
            .scope_builder
            .build_symbol_table_from_ast(root_node, source_code)?;
        self.collect_definitions_with_scope(root_node, source_code)?;
        Ok(self.symbol_table.clone())
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

    fn is_node_within(&self, child: Node, parent: Node) -> bool {
        let child_start = child.start_byte();
        let child_end = child.end_byte();
        let parent_start = parent.start_byte();
        let parent_end = parent.end_byte();

        child_start >= parent_start && child_end <= parent_end
    }

    fn is_imported_identifier(&self, node: Node) -> bool {
        // Check if this identifier is the imported name in a use declaration
        // For example, in `use module::MyStruct` or `use module::{Item1, Item2}`
        if let Some(parent) = node.parent() {
            match parent.kind() {
                "scoped_identifier" => {
                    // In `use module::Item`, Item is the imported identifier
                    node == parent.child_by_field_name("name").unwrap_or(node)
                }
                "use_list" => {
                    // In `use module::{Item1, Item2}`, each item is imported
                    true
                }
                "use_as_clause" => {
                    // In `use module::Item as Alias`, the alias is imported
                    node == parent.child_by_field_name("alias").unwrap_or(node)
                }
                _ => false,
            }
        } else {
            false
        }
    }

    fn is_definition_context(&self, node: Node) -> bool {
        if let Some(parent) = node.parent() {
            match parent.kind() {
                "let_declaration" => {
                    // Only consider identifiers in the pattern field as definitions
                    if let Some(pattern_node) = parent.child_by_field_name("pattern") {
                        self.is_node_within(node, pattern_node)
                    } else {
                        false
                    }
                }
                "variable_declaration" | "function_item" | "function_declaration" | "mod_item" => {
                    true
                }
                "use_declaration" => {
                    // For use_declaration, check if this identifier is being imported
                    self.is_imported_identifier(node)
                }
                "scoped_identifier" => {
                    // Check if this identifier is part of an import like `use module::Item`
                    if let Some(grandparent) = parent.parent() {
                        grandparent.kind() == "use_declaration"
                    } else {
                        false
                    }
                }
                "use_as_clause" => {
                    // Handle alias in `use module as alias`
                    node.kind() == "identifier"
                        && node == parent.child_by_field_name("alias").unwrap_or(node)
                }
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
                "scoped_identifier" => {
                    // For use declarations like `use module::Item`
                    if let Some(grandparent) = parent.parent() {
                        if grandparent.kind() == "use_declaration" {
                            return DefinitionType::ImportDefinition;
                        }
                    }
                    DefinitionType::VariableDefinition
                }
                "use_as_clause" => DefinitionType::ImportDefinition,
                "use_list" => DefinitionType::ImportDefinition,
                _ => DefinitionType::VariableDefinition,
            }
        } else {
            DefinitionType::VariableDefinition
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tree_sitter::{Language, Parser};

    extern "C" {
        fn tree_sitter_rust() -> Language;
    }

    #[test]
    fn test_rust_scope_building() {
        let mut parser = Parser::new();
        unsafe {
            parser.set_language(&tree_sitter_rust()).unwrap();
        }

        let source_code = r#"
fn main() {
    let x = 10;
    {
        let y = 20;
    }
}
        "#;

        let tree = parser.parse(source_code, None).unwrap();
        let mut builder = ScopeBuilder::new("rust".to_string());

        let scope_tree = builder
            .build_from_ast(tree.root_node(), source_code)
            .unwrap();

        assert!(scope_tree.scopes.len() > 1); // Should have more than just global scope

        let function_scopes: Vec<_> = scope_tree
            .scopes
            .values()
            .filter(|s| matches!(s.scope_type, ScopeType::Function))
            .collect();
        assert!(!function_scopes.is_empty());

        let block_scopes: Vec<_> = scope_tree
            .scopes
            .values()
            .filter(|s| matches!(s.scope_type, ScopeType::Block))
            .collect();
        assert!(!block_scopes.is_empty());
    }

    #[test]
    fn test_scope_position_detection() {
        let mut builder = ScopeBuilder::new("rust".to_string());
        let scope_tree = &mut builder.scope_tree;

        let func_scope_id = scope_tree.create_scope(
            Some(0),
            ScopeType::Function,
            Position {
                start_line: 2,
                start_column: 1,
                end_line: 2,
                end_column: 1,
            },
            Position {
                start_line: 6,
                start_column: 1,
                end_line: 6,
                end_column: 1,
            },
        );

        let position_in_function = Position {
            start_line: 3,
            start_column: 5,
            end_line: 3,
            end_column: 5,
        };
        let found_scope = scope_tree.find_scope_at_position(&position_in_function);
        assert_eq!(found_scope, Some(func_scope_id));

        let position_outside = Position {
            start_line: 10,
            start_column: 1,
            end_line: 10,
            end_column: 1,
        };
        let found_scope = scope_tree.find_scope_at_position(&position_outside);
        assert_eq!(found_scope, Some(0)); // Should be global scope
    }
}
