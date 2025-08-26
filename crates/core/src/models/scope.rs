use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::definition::{Accessibility, ScopeId};
use super::{Definition, DefinitionType, Position, Usage};

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

    pub fn lookup_symbol(&self, name: &str) -> Option<&Vec<SymbolEntry>> {
        self.symbols.get(name)
    }

    pub fn lookup_symbol_in_scope(&self, name: &str, scope_id: ScopeId) -> Vec<&Definition> {
        self.scopes.lookup_symbol_in_scope_chain(scope_id, name)
    }

    #[allow(dead_code)]
    fn is_scope_accessible(&self, from_scope: ScopeId, to_scope: ScopeId) -> bool {
        // Simple implementation: check if to_scope is a parent of from_scope
        let mut current = from_scope;
        while let Some(scope) = self.scopes.get_scope(current) {
            if current == to_scope {
                return true;
            }
            if let Some(parent) = scope.parent {
                current = parent;
            } else {
                break;
            }
        }
        false
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

// New separated structures for unified AST traversal

/// Registry for managing definitions with single responsibility
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DefinitionRegistry {
    definitions: HashMap<String, Vec<Definition>>,
    type_parameters: HashMap<String, Vec<TypeParameter>>,
}

impl DefinitionRegistry {
    pub fn new() -> Self {
        Self {
            definitions: HashMap::new(),
            type_parameters: HashMap::new(),
        }
    }

    pub fn add_definition(&mut self, name: String, definition: Definition) {
        self.definitions.entry(name).or_default().push(definition);
    }

    pub fn lookup_definition(&self, name: &str) -> Option<&Vec<Definition>> {
        self.definitions.get(name)
    }

    pub fn get_all_definitions(&self) -> &HashMap<String, Vec<Definition>> {
        &self.definitions
    }

    pub fn add_type_parameter(
        &mut self,
        name: String,
        constraint_type: Option<String>,
        default_type: Option<String>,
        scope_id: ScopeId,
    ) {
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

impl Default for DefinitionRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Registry for managing usages with single responsibility
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageRegistry {
    usages: Vec<Usage>,
    scope_indexed_usages: HashMap<ScopeId, Vec<usize>>, // Optional: for efficient lookup
}

impl UsageRegistry {
    pub fn new() -> Self {
        Self {
            usages: Vec::new(),
            scope_indexed_usages: HashMap::new(),
        }
    }

    pub fn add_usage(&mut self, usage: Usage) {
        let usage_index = self.usages.len();
        if let Some(scope_id) = usage.get_scope_id() {
            self.scope_indexed_usages
                .entry(scope_id)
                .or_default()
                .push(usage_index);
        }
        self.usages.push(usage);
    }

    pub fn get_all_usages(&self) -> &Vec<Usage> {
        &self.usages
    }

    pub fn get_usages_in_scope(&self, scope_id: ScopeId) -> Vec<&Usage> {
        self.scope_indexed_usages
            .get(&scope_id)
            .map(|indices| indices.iter().map(|&i| &self.usages[i]).collect())
            .unwrap_or_default()
    }
}

impl Default for UsageRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Coordinated context for all code analysis data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeAnalysisContext {
    pub definitions: DefinitionRegistry,
    pub usages: UsageRegistry,
    pub scopes: ScopeTree,
}

impl CodeAnalysisContext {
    pub fn new() -> Self {
        Self {
            definitions: DefinitionRegistry::new(),
            usages: UsageRegistry::new(),
            scopes: ScopeTree::new(),
        }
    }

    /// Lookup symbols using both definitions and scope chain
    pub fn lookup_symbol_in_scope(&self, scope_id: ScopeId, symbol_name: &str) -> Vec<&Definition> {
        let mut definitions = Vec::new();
        let mut current_scope_id = scope_id;

        // Walk up the scope chain
        while let Some(scope) = self.scopes.get_scope(current_scope_id) {
            // Look up in definitions registry for symbols defined in this scope
            if let Some(scope_definitions) = self.definitions.lookup_definition(symbol_name) {
                for def in scope_definitions {
                    if def.get_scope_id() == Some(current_scope_id) {
                        definitions.push(def);
                    }
                }
            }

            if let Some(parent_id) = scope.parent {
                current_scope_id = parent_id;
            } else {
                break;
            }
        }

        definitions
    }
}

impl Default for CodeAnalysisContext {
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
            scope_id: None,
            accessibility: None,
            is_hoisted: None,
        }
    }
}
