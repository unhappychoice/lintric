use tree_sitter::Node;

use crate::models::{Accessibility, Position, ScopeId, ScopeTree, ScopeType, SymbolTable};
use crate::scope_collector::ScopeCollector as ScopeCollectorTrait;

pub struct TypeScriptScopeCollector {
    pub scope_tree: ScopeTree,
    current_scope: ScopeId,
}

impl Default for TypeScriptScopeCollector {
    fn default() -> Self {
        Self::new()
    }
}

impl TypeScriptScopeCollector {
    pub fn new() -> Self {
        let scope_tree = ScopeTree::new();
        let current_scope = scope_tree.root;

        Self {
            scope_tree,
            current_scope,
        }
    }

    fn visit_node(&mut self, node: Node, source_code: &str) -> Result<(), String> {
        let node_type = node.kind();

        self.visit_typescript_node(node, source_code, node_type)?;

        for i in 0..node.child_count() {
            if let Some(child) = node.child(i) {
                self.visit_node(child, source_code)?;
            }
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
            "block_statement" => {
                self.enter_scope_for_node(node, ScopeType::Block, source_code)?;
            }
            "module_declaration" => {
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
        let start = node.start_position();
        let end = node.end_position();

        let start_pos = Position {
            start_line: start.row + 1,
            start_column: start.column,
            end_line: start.row + 1,
            end_column: start.column,
        };
        let end_pos = Position {
            start_line: end.row + 1,
            start_column: end.column,
            end_line: end.row + 1,
            end_column: end.column,
        };

        let scope_id =
            self.scope_tree
                .create_scope(Some(self.current_scope), scope_type, start_pos, end_pos);

        let old_scope = self.current_scope;
        self.current_scope = scope_id;

        // Visit children in the new scope
        for i in 0..node.child_count() {
            if let Some(child) = node.child(i) {
                self.visit_node(child, source_code)?;
            }
        }

        // Restore previous scope
        self.current_scope = old_scope;

        Ok(())
    }
}

impl ScopeCollectorTrait for TypeScriptScopeCollector {
    fn scopes(&mut self, root_node: Node, source_code: &str) -> Result<ScopeTree, String> {
        self.visit_node(root_node, source_code)?;
        Ok(self.scope_tree.clone())
    }

    fn collect(
        &self,
        source_code: &str,
        root_node: Node,
        _usage_nodes: &[crate::models::Usage],
        definitions: &[crate::models::Definition],
    ) -> Result<SymbolTable, String> {
        let mut new_self = TypeScriptScopeCollector::new();
        let mut symbol_table = SymbolTable::new();
        new_self.visit_node(root_node, source_code)?;
        symbol_table.scopes = new_self.scope_tree.clone();

        // Add definitions from definition collector (like imports) to symbol table
        for definition in definitions {
            // Find the appropriate scope for this definition
            let scope_id = symbol_table
                .scopes
                .find_scope_at_position(&definition.position)
                .unwrap_or_default();

            symbol_table.add_symbol(
                definition.name.clone(),
                definition.clone(),
                scope_id,
                Accessibility::ScopeLocal,
                false,
            );
        }

        Ok(symbol_table)
    }
}
