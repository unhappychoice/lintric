//! Declarations indexed by name, alongside the scope tree they sit in.
//!
//! Kept separate from the tree because the two are indexed differently: the tree answers "what is
//! visible from here", this answers "where is everything called this".

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::definition::{Accessibility, ScopeId};
use super::scope::ScopeTree;
use super::Definition;

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
pub struct SymbolTable {
    symbols: HashMap<String, Vec<SymbolEntry>>,
    pub scopes: ScopeTree,
}

impl SymbolTable {
    pub fn new() -> Self {
        Self {
            symbols: HashMap::new(),
            scopes: ScopeTree::new(),
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
        let mut enhanced_definition = definition.clone();
        enhanced_definition.set_context(scope_id, &accessibility, is_hoisted);

        let entry = SymbolEntry::new(
            enhanced_definition.clone(),
            scope_id,
            accessibility,
            is_hoisted,
        );
        self.symbols.entry(name.clone()).or_default().push(entry);

        if let Some(scope) = self.scopes.get_scope_mut(scope_id) {
            scope.add_symbol(name, enhanced_definition);
        }
    }

    pub fn add_enhanced_symbol(&mut self, name: String, definition: Definition) {
        if let (Some(scope_id), Some(accessibility), Some(is_hoisted)) = (
            definition.get_scope_id(),
            definition.get_accessibility(),
            definition.is_hoisted(),
        ) {
            let entry = SymbolEntry::new(
                definition.clone(),
                scope_id,
                accessibility.clone(),
                is_hoisted,
            );
            self.symbols.entry(name.clone()).or_default().push(entry);

            if let Some(scope) = self.scopes.get_scope_mut(scope_id) {
                scope.add_symbol(name, definition);
            }
        } else {
            panic!("Definition must have context information set");
        }
    }
}

impl Default for SymbolTable {
    fn default() -> Self {
        Self::new()
    }
}

// New separated structures for unified AST traversal
