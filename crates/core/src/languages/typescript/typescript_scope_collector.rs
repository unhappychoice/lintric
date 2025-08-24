use tree_sitter::Node;

use crate::models::{Position, ScopeId, ScopeTree, ScopeType, SymbolTable};
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
        _definitions: &[crate::models::Definition],
    ) -> Result<SymbolTable, String> {
        let mut new_self = TypeScriptScopeCollector::new();
        let mut symbol_table = SymbolTable::new();
        new_self.visit_node(root_node, source_code)?;
        symbol_table.scopes = new_self.scope_tree.clone();
        Ok(symbol_table)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scope_collector::ScopeCollector;
    use tree_sitter::{Language, Parser};

    extern "C" {
        fn tree_sitter_typescript() -> Language;
    }

    #[test]
    fn test_typescript_scope_building() {
        let mut parser = Parser::new();
        unsafe {
            parser.set_language(&tree_sitter_typescript()).unwrap();
        }

        let source_code = r#"
function main() {
    let x = 10;
    {
        let y = 20;
    }
}
        "#;

        let tree = parser.parse(source_code, None).unwrap();
        let mut builder = TypeScriptScopeCollector::new();

        let scope_tree = builder.scopes(tree.root_node(), source_code).unwrap();

        assert!(scope_tree.scopes.len() > 1); // Should have more than just global scope
    }

    #[test]
    fn test_scope_position_detection() {
        let mut builder = TypeScriptScopeCollector::new();
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
                start_line: 5,
                start_column: 1,
                end_line: 5,
                end_column: 1,
            },
        );

        let test_position = Position {
            start_line: 3,
            start_column: 5,
            end_line: 3,
            end_column: 10,
        };

        let found_scope = scope_tree.find_scope_at_position(&test_position);
        assert_eq!(found_scope, Some(func_scope_id));
    }

    #[test]
    fn test_scope_collector_fallback() {
        let collector = TypeScriptScopeCollector::new();

        let source_code = "function test() { let x = 1; return x; }";
        let mut parser = Parser::new();
        parser
            .set_language(unsafe { &tree_sitter_typescript() })
            .unwrap();
        let tree = parser.parse(source_code, None).unwrap();

        let result = ScopeCollector::collect(&collector, source_code, tree.root_node(), &[], &[]);

        assert!(result.is_ok());
    }

    #[test]
    fn test_create_scope_collector() {
        let _collector = TypeScriptScopeCollector::new();
    }
}
