use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::{Definition, DefinitionType, Position};

pub type ScopeId = usize;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ScopeType {
    Global,
    Function,
    Block,
    Module,
    Impl,
    Trait,
    Class,
    Interface,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Accessibility {
    Public,
    Private,
    ScopeLocal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Scope {
    pub id: ScopeId,
    pub parent: Option<ScopeId>,
    pub children: Vec<ScopeId>,
    pub scope_type: ScopeType,
    pub symbols: HashMap<String, Vec<Definition>>,
    pub start_position: Position,
    pub end_position: Position,
}

impl Scope {
    pub fn new(
        id: ScopeId,
        parent: Option<ScopeId>,
        scope_type: ScopeType,
        start_position: Position,
        end_position: Position,
    ) -> Self {
        Self {
            id,
            parent,
            children: Vec::new(),
            scope_type,
            symbols: HashMap::new(),
            start_position,
            end_position,
        }
    }

    pub fn add_child(&mut self, child_id: ScopeId) {
        self.children.push(child_id);
    }

    pub fn add_symbol(&mut self, name: String, definition: Definition) {
        self.symbols.entry(name).or_default().push(definition);
    }

    pub fn get_symbols(&self, name: &str) -> Option<&Vec<Definition>> {
        self.symbols.get(name)
    }

    pub fn contains_position(&self, position: &Position) -> bool {
        position.start_line >= self.start_position.start_line
            && position.start_line <= self.end_position.start_line
            && if position.start_line == self.start_position.start_line {
                position.start_column >= self.start_position.start_column
            } else {
                true
            }
            && if position.start_line == self.end_position.start_line {
                position.start_column <= self.end_position.start_column
            } else {
                true
            }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScopeTree {
    pub root: ScopeId,
    pub scopes: HashMap<ScopeId, Scope>,
    scope_counter: usize,
}

impl ScopeTree {
    pub fn new() -> Self {
        let mut tree = Self {
            root: 0,
            scopes: HashMap::new(),
            scope_counter: 0,
        };

        let global_scope = Scope::new(
            0,
            None,
            ScopeType::Global,
            Position {
                start_line: 1,
                start_column: 1,
                end_line: 1,
                end_column: 1,
            },
            Position {
                start_line: usize::MAX,
                start_column: usize::MAX,
                end_line: usize::MAX,
                end_column: usize::MAX,
            },
        );
        tree.scopes.insert(0, global_scope);
        tree.scope_counter = 1;

        tree
    }

    pub fn create_scope(
        &mut self,
        parent_id: Option<ScopeId>,
        scope_type: ScopeType,
        start_position: Position,
        end_position: Position,
    ) -> ScopeId {
        let scope_id = self.scope_counter;
        self.scope_counter += 1;

        let scope = Scope::new(
            scope_id,
            parent_id,
            scope_type,
            start_position,
            end_position,
        );

        if let Some(parent_id) = parent_id {
            if let Some(parent_scope) = self.scopes.get_mut(&parent_id) {
                parent_scope.add_child(scope_id);
            }
        }

        self.scopes.insert(scope_id, scope);
        scope_id
    }

    pub fn get_scope(&self, scope_id: ScopeId) -> Option<&Scope> {
        self.scopes.get(&scope_id)
    }

    pub fn get_scope_mut(&mut self, scope_id: ScopeId) -> Option<&mut Scope> {
        self.scopes.get_mut(&scope_id)
    }

    pub fn find_scope_at_position(&self, position: &Position) -> Option<ScopeId> {
        self.find_scope_at_position_recursive(self.root, position)
    }

    fn find_scope_at_position_recursive(
        &self,
        scope_id: ScopeId,
        position: &Position,
    ) -> Option<ScopeId> {
        if let Some(scope) = self.get_scope(scope_id) {
            if scope.contains_position(position) {
                for &child_id in &scope.children {
                    if let Some(child_scope_id) =
                        self.find_scope_at_position_recursive(child_id, position)
                    {
                        return Some(child_scope_id);
                    }
                }
                return Some(scope_id);
            }
        }
        None
    }

    pub fn get_parent_scopes(&self, scope_id: ScopeId) -> Vec<ScopeId> {
        let mut parent_scopes = Vec::new();
        let mut current_scope_id = scope_id;

        while let Some(scope) = self.get_scope(current_scope_id) {
            if let Some(parent_id) = scope.parent {
                parent_scopes.push(parent_id);
                current_scope_id = parent_id;
            } else {
                break;
            }
        }

        parent_scopes
    }

    pub fn lookup_symbol_in_scope_chain(
        &self,
        scope_id: ScopeId,
        symbol_name: &str,
    ) -> Vec<&Definition> {
        let mut definitions = Vec::new();
        let mut current_scope_id = scope_id;

        while let Some(scope) = self.get_scope(current_scope_id) {
            if let Some(scope_definitions) = scope.get_symbols(symbol_name) {
                definitions.extend(scope_definitions.iter());
            }

            if let Some(parent_id) = scope.parent {
                current_scope_id = parent_id;
            } else {
                break;
            }
        }

        definitions
    }

    pub fn get_all_scopes(&self) -> Vec<&Scope> {
        self.scopes.values().collect()
    }
}

impl Default for ScopeTree {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SymbolEntry {
    pub definition: Definition,
    pub scope_id: ScopeId,
    pub accessibility: Accessibility,
    pub is_hoisted: bool,
}

impl SymbolEntry {
    pub fn new(
        definition: Definition,
        scope_id: ScopeId,
        accessibility: Accessibility,
        is_hoisted: bool,
    ) -> Self {
        Self {
            definition,
            scope_id,
            accessibility,
            is_hoisted,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypeParameter {
    pub name: String,
    pub constraint_type: Option<String>,
    pub default_type: Option<String>,
    pub scope_id: ScopeId,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SymbolTable {
    symbols: HashMap<String, Vec<SymbolEntry>>,
    pub scopes: ScopeTree,
    type_parameters: HashMap<String, Vec<TypeParameter>>,
}

impl SymbolTable {
    pub fn new() -> Self {
        Self {
            symbols: HashMap::new(),
            scopes: ScopeTree::new(),
            type_parameters: HashMap::new(),
        }
    }

    pub fn add_symbol(
        &mut self,
        name: String,
        definition: Definition,
        scope_id: ScopeId,
        accessibility: Accessibility,
        is_hoisted: bool,
    ) {
        let entry = SymbolEntry::new(definition.clone(), scope_id, accessibility, is_hoisted);
        self.symbols.entry(name.clone()).or_default().push(entry);

        if let Some(scope) = self.scopes.get_scope_mut(scope_id) {
            scope.add_symbol(name, definition);
        }
    }

    pub fn lookup_symbol(&self, name: &str) -> Option<&Vec<SymbolEntry>> {
        self.symbols.get(name)
    }

    pub fn lookup_symbol_in_scope(&self, name: &str, scope_id: ScopeId) -> Vec<&Definition> {
        self.scopes.lookup_symbol_in_scope_chain(scope_id, name)
    }

    pub fn get_symbols_in_scope(
        &self,
        scope_id: ScopeId,
    ) -> Option<&HashMap<String, Vec<Definition>>> {
        self.scopes.get_scope(scope_id).map(|scope| &scope.symbols)
    }

    pub fn get_all_symbols(&self) -> &HashMap<String, Vec<SymbolEntry>> {
        &self.symbols
    }

    pub fn add_type_parameter(
        &mut self,
        name: String,
        constraint_type: Option<String>,
        default_type: Option<String>,
    ) {
        let scope_id = self.scopes.root; // For now, add to global scope
        let type_param = TypeParameter {
            name: name.clone(),
            constraint_type,
            default_type,
            scope_id,
        };
        self.type_parameters
            .entry(name)
            .or_default()
            .push(type_param);
    }

    pub fn lookup_type_parameter(&self, name: &str) -> Option<&Vec<TypeParameter>> {
        self.type_parameters.get(name)
    }

    pub fn get_all_type_parameters(&self) -> &HashMap<String, Vec<TypeParameter>> {
        &self.type_parameters
    }
}

impl Default for SymbolTable {
    fn default() -> Self {
        Self::new()
    }
}

// Helper function to create a Definition for testing purposes
impl Definition {
    pub fn new_simple(name: String, definition_type: DefinitionType, position: Position) -> Self {
        Self {
            name,
            definition_type,
            position,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::DefinitionType;

    #[test]
    fn test_scope_creation() {
        let mut scope_tree = ScopeTree::new();

        let func_scope_id = scope_tree.create_scope(
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

        assert_eq!(func_scope_id, 1);
        assert!(scope_tree.get_scope(func_scope_id).is_some());

        let root_scope = scope_tree.get_scope(0).unwrap();
        assert!(root_scope.children.contains(&func_scope_id));
    }

    #[test]
    fn test_scope_position_lookup() {
        let mut scope_tree = ScopeTree::new();

        let func_scope_id = scope_tree.create_scope(
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
                start_column: 10,
                end_line: 10,
                end_column: 10,
            },
        );

        let position_inside = Position {
            start_line: 7,
            start_column: 5,
            end_line: 7,
            end_column: 5,
        };
        let found_scope = scope_tree.find_scope_at_position(&position_inside);
        assert_eq!(found_scope, Some(func_scope_id));

        let position_outside = Position {
            start_line: 15,
            start_column: 1,
            end_line: 15,
            end_column: 1,
        };
        let found_scope = scope_tree.find_scope_at_position(&position_outside);
        assert_eq!(found_scope, Some(0));
    }

    #[test]
    fn test_symbol_table_basic_operations() {
        let mut symbol_table = SymbolTable::new();

        let definition = Definition::new_simple(
            "test_var".to_string(),
            DefinitionType::VariableDefinition,
            Position {
                start_line: 5,
                start_column: 10,
                end_line: 5,
                end_column: 10,
            },
        );

        symbol_table.add_symbol(
            "test_var".to_string(),
            definition,
            0,
            Accessibility::ScopeLocal,
            false,
        );

        let entries = symbol_table.lookup_symbol("test_var");
        assert!(entries.is_some());
        assert_eq!(entries.unwrap().len(), 1);
    }

    #[test]
    fn test_symbol_lookup_in_scope_chain() {
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
            global_def,
            0,
            Accessibility::Public,
            false,
        );

        symbol_table.add_symbol(
            "local_var".to_string(),
            local_def,
            func_scope_id,
            Accessibility::ScopeLocal,
            false,
        );

        let global_lookup = symbol_table.lookup_symbol_in_scope("global_var", func_scope_id);
        assert_eq!(global_lookup.len(), 1);

        let local_lookup = symbol_table.lookup_symbol_in_scope("local_var", func_scope_id);
        assert_eq!(local_lookup.len(), 1);

        let nonexistent_lookup = symbol_table.lookup_symbol_in_scope("nonexistent", func_scope_id);
        assert_eq!(nonexistent_lookup.len(), 0);
    }
}
