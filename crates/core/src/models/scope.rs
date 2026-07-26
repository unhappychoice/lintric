//! The tree of scopes a file's declarations live in.
//!
//! A scope spans a range of the source, so which one a position sits in is a containment question,
//! and the chain out from it is what "visible here" means.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::definition::ScopeId;
use super::{Definition, Position};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ScopeType {
    Global,
    Function,
    Closure,
    Block,
    Module,
    Impl,
    Trait,
    Class,
    Interface,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Scope {
    pub id: ScopeId,
    pub parent: Option<ScopeId>,
    pub children: Vec<ScopeId>,
    pub scope_type: ScopeType,
    pub position: Position,
    // Temporary: keep for compatibility with old dependency resolver
    #[serde(default)]
    pub symbols: HashMap<String, Vec<Definition>>,
}

impl Scope {
    pub fn new(
        id: ScopeId,
        parent: Option<ScopeId>,
        scope_type: ScopeType,
        position: Position,
    ) -> Self {
        Self {
            id,
            parent,
            children: Vec::new(),
            scope_type,
            position,
            symbols: HashMap::new(),
        }
    }

    pub fn add_child(&mut self, child_id: ScopeId) {
        self.children.push(child_id);
    }

    // Temporary: restore for compatibility
    pub fn add_symbol(&mut self, name: String, definition: Definition) {
        self.symbols.entry(name).or_default().push(definition);
    }

    pub fn get_symbols(&self, name: &str) -> Option<&Vec<Definition>> {
        self.symbols.get(name)
    }

    pub fn contains_position(&self, position: &Position) -> bool {
        position.start_line >= self.position.start_line
            && position.start_line <= self.position.end_line
            && if position.start_line == self.position.start_line {
                position.start_column >= self.position.start_column
            } else {
                true
            }
            && if position.start_line == self.position.end_line {
                position.start_column <= self.position.end_column
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
        position: Position,
    ) -> ScopeId {
        let scope_id = self.scope_counter;
        self.scope_counter += 1;

        let scope = Scope::new(scope_id, parent_id, scope_type, position);

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

    // Temporary: restore for compatibility
}

impl Default for ScopeTree {
    fn default() -> Self {
        Self::new()
    }
}
