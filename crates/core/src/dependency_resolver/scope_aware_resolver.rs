// use std::collections::HashMap; // Unused for now
use tree_sitter::Node;

use crate::dependency_resolver::scope_builder::ScopeAwareDefinitionCollector;
use crate::models::{
    Accessibility, Definition, Dependency, Position, ScopeId, ScopeType, SymbolTable, Usage,
};

pub trait ScopeAwareDependencyResolver {
    fn resolve_dependencies_with_scope(
        &self,
        source_code: &str,
        root_node: Node,
        usage_nodes: &[Usage],
        symbol_table: &SymbolTable,
    ) -> Result<Vec<Dependency>, String>;

    fn find_definition_in_scope(
        &self,
        usage: &Usage,
        symbol_table: &SymbolTable,
    ) -> Option<Definition>;

    fn resolve_scope_chain_lookup(
        &self,
        symbol_name: &str,
        scope_id: ScopeId,
        symbol_table: &SymbolTable,
    ) -> Vec<Definition>;
}

pub struct DefaultScopeAwareResolver {
    language: String,
}

impl DefaultScopeAwareResolver {
    pub fn new(language: String) -> Self {
        Self { language }
    }

    pub fn create_symbol_table(
        &self,
        root_node: Node,
        source_code: &str,
    ) -> Result<SymbolTable, String> {
        let mut collector = ScopeAwareDefinitionCollector::new(self.language.clone());
        collector.collect_with_scopes(root_node, source_code)
    }
}

impl ScopeAwareDependencyResolver for DefaultScopeAwareResolver {
    fn resolve_dependencies_with_scope(
        &self,
        _source_code: &str,
        _root_node: Node,
        usage_nodes: &[Usage],
        symbol_table: &SymbolTable,
    ) -> Result<Vec<Dependency>, String> {
        let mut dependencies = Vec::new();

        for usage in usage_nodes {
            if let Some(definition) = self.find_definition_in_scope(usage, symbol_table) {
                let dependency = Dependency::new_with_scope(
                    usage.position,
                    definition.position,
                    usage.clone(),
                    definition,
                );
                dependencies.push(dependency);
            }
        }

        Ok(dependencies)
    }

    fn find_definition_in_scope(
        &self,
        usage: &Usage,
        symbol_table: &SymbolTable,
    ) -> Option<Definition> {
        let usage_scope_id = symbol_table
            .scopes
            .find_scope_at_position(&usage.position)?;

        let definitions =
            self.resolve_scope_chain_lookup(&usage.name, usage_scope_id, symbol_table);

        definitions.into_iter().next()
    }

    fn resolve_scope_chain_lookup(
        &self,
        symbol_name: &str,
        scope_id: ScopeId,
        symbol_table: &SymbolTable,
    ) -> Vec<Definition> {
        symbol_table
            .lookup_symbol_in_scope(symbol_name, scope_id)
            .into_iter()
            .cloned()
            .collect()
    }
}

pub struct ScopeValidationError {
    pub message: String,
    pub position: Position,
    pub scope_id: ScopeId,
}

pub struct ScopeValidator {
    language: String,
}

impl ScopeValidator {
    pub fn new(language: String) -> Self {
        Self { language }
    }

    pub fn validate_scope_structure(
        &self,
        symbol_table: &SymbolTable,
    ) -> Result<Vec<ScopeValidationError>, String> {
        let mut errors = Vec::new();

        for (scope_id, scope) in &symbol_table.scopes.scopes {
            if let Some(parent_id) = scope.parent {
                if !symbol_table.scopes.scopes.contains_key(&parent_id) {
                    errors.push(ScopeValidationError {
                        message: format!(
                            "Scope {} references non-existent parent {}",
                            scope_id, parent_id
                        ),
                        position: scope.start_position,
                        scope_id: *scope_id,
                    });
                }
            }

            for &child_id in &scope.children {
                if !symbol_table.scopes.scopes.contains_key(&child_id) {
                    errors.push(ScopeValidationError {
                        message: format!(
                            "Scope {} references non-existent child {}",
                            scope_id, child_id
                        ),
                        position: scope.start_position,
                        scope_id: *scope_id,
                    });
                }
            }
        }

        self.validate_symbol_accessibility(symbol_table, &mut errors)?;

        Ok(errors)
    }

    fn validate_symbol_accessibility(
        &self,
        symbol_table: &SymbolTable,
        errors: &mut Vec<ScopeValidationError>,
    ) -> Result<(), String> {
        for (symbol_name, entries) in symbol_table.get_all_symbols() {
            for entry in entries {
                if let Some(scope) = symbol_table.scopes.get_scope(entry.scope_id) {
                    if entry.accessibility == Accessibility::Private
                        && !matches!(
                            scope.scope_type,
                            ScopeType::Module
                                | ScopeType::Impl
                                | ScopeType::Trait
                                | ScopeType::Class
                        )
                    {
                        errors.push(ScopeValidationError {
                            message: format!(
                                "Private symbol '{}' in non-module scope",
                                symbol_name
                            ),
                            position: entry.definition.position,
                            scope_id: entry.scope_id,
                        });
                    }
                }
            }
        }

        Ok(())
    }

    pub fn validate_variable_hoisting(
        &self,
        symbol_table: &SymbolTable,
    ) -> Result<Vec<ScopeValidationError>, String> {
        let mut errors = Vec::new();

        match self.language.as_str() {
            "javascript" | "typescript" | "tsx" => {
                self.validate_js_ts_hoisting(symbol_table, &mut errors)?;
            }
            "rust" => {
                self.validate_rust_forward_references(symbol_table, &mut errors)?;
            }
            _ => {} // No specific hoisting rules for other languages yet
        }

        Ok(errors)
    }

    #[allow(clippy::ptr_arg)]
    fn validate_js_ts_hoisting(
        &self,
        _symbol_table: &SymbolTable,
        _errors: &mut Vec<ScopeValidationError>,
    ) -> Result<(), String> {
        for (symbol_name, entries) in _symbol_table.get_all_symbols() {
            for entry in entries {
                if entry.is_hoisted {
                    continue; // Hoisted variables are valid
                }

                let scope_id = entry.scope_id;
                if let Some(scope) = _symbol_table.scopes.get_scope(scope_id) {
                    for (other_name, other_entries) in &scope.symbols {
                        if symbol_name == other_name {
                            continue;
                        }

                        for other_def in other_entries {
                            if other_def.position.start_line < entry.definition.position.start_line
                            {
                                _errors.push(ScopeValidationError {
                                    message: format!(
                                        "Variable '{}' used before declaration",
                                        symbol_name
                                    ),
                                    position: other_def.position,
                                    scope_id,
                                });
                            }
                        }
                    }
                }
            }
        }

        Ok(())
    }

    #[allow(clippy::ptr_arg)]
    fn validate_rust_forward_references(
        &self,
        _symbol_table: &SymbolTable,
        _errors: &mut Vec<ScopeValidationError>,
    ) -> Result<(), String> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{DefinitionType, UsageKind};

    #[test]
    fn test_scope_aware_definition_lookup() {
        let mut symbol_table = SymbolTable::new();

        let func_scope_id = symbol_table.scopes.create_scope(
            Some(0),
            ScopeType::Function,
            Position {
                start_line: 5,
                start_column: 1,
                end_line: 5,
                end_column: 1,
            },
            Position {
                start_line: 10,
                start_column: 1,
                end_line: 10,
                end_column: 1,
            },
        );

        let global_def = Definition::new_simple(
            "global_var".to_string(),
            DefinitionType::VariableDefinition,
            Position {
                start_line: 1,
                start_column: 1,
                end_line: 1,
                end_column: 1,
            },
        );

        let local_def = Definition::new_simple(
            "local_var".to_string(),
            DefinitionType::VariableDefinition,
            Position {
                start_line: 6,
                start_column: 5,
                end_line: 6,
                end_column: 5,
            },
        );

        symbol_table.add_symbol(
            "global_var".to_string(),
            global_def.clone(),
            0,
            Accessibility::Public,
            false,
        );

        symbol_table.add_symbol(
            "local_var".to_string(),
            local_def.clone(),
            func_scope_id,
            Accessibility::ScopeLocal,
            false,
        );

        let resolver = DefaultScopeAwareResolver::new("rust".to_string());

        let global_usage = Usage::new_simple(
            "global_var".to_string(),
            Position {
                start_line: 7,
                start_column: 10,
                end_line: 7,
                end_column: 10,
            },
            UsageKind::Read,
        );

        let found_def = resolver.find_definition_in_scope(&global_usage, &symbol_table);
        assert!(found_def.is_some());
        assert_eq!(found_def.unwrap().name, "global_var");

        let local_usage = Usage::new_simple(
            "local_var".to_string(),
            Position {
                start_line: 8,
                start_column: 5,
                end_line: 8,
                end_column: 5,
            },
            UsageKind::Read,
        );

        let found_def = resolver.find_definition_in_scope(&local_usage, &symbol_table);
        assert!(found_def.is_some());
        assert_eq!(found_def.unwrap().name, "local_var");
    }

    #[test]
    fn test_scope_validation() {
        let mut symbol_table = SymbolTable::new();

        let _func_scope_id = symbol_table.scopes.create_scope(
            Some(0),
            ScopeType::Function,
            Position {
                start_line: 5,
                start_column: 1,
                end_line: 5,
                end_column: 1,
            },
            Position {
                start_line: 10,
                start_column: 1,
                end_line: 10,
                end_column: 1,
            },
        );

        let validator = ScopeValidator::new("rust".to_string());
        let errors = validator.validate_scope_structure(&symbol_table).unwrap();

        assert_eq!(errors.len(), 0); // Should have no validation errors for a well-formed scope tree
    }
}
