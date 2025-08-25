use crate::models::{Definition, ScopeId, ScopeTree, ScopeType, Usage};
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone)]
pub struct ScopeSearchResult {
    pub definition: Definition,
    pub scope_id: ScopeId,
    pub scope_distance: usize, // Distance from the usage scope
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CaptureType {
    ByValue,
    ByReference,
    ByMutableReference,
}

#[derive(Debug, Clone)]
pub struct CaptureInfo {
    pub captured_symbol: String,
    pub capture_type: CaptureType,
    pub source_scope: ScopeId,
    pub target_scope: ScopeId,
    pub definition: Definition,
}

pub struct ClosureAnalyzer {
    captures: HashMap<ScopeId, Vec<CaptureInfo>>,
}

impl Default for ClosureAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

impl ClosureAnalyzer {
    pub fn new() -> Self {
        Self {
            captures: HashMap::new(),
        }
    }

    pub fn analyze_closure_captures(
        &mut self,
        closure_scope: ScopeId,
        scope_tree: &ScopeTree,
    ) -> &Vec<CaptureInfo> {
        if !self.captures.contains_key(&closure_scope) {
            let captures = self.find_captured_variables(closure_scope, scope_tree);
            self.captures.insert(closure_scope, captures);
        }

        self.captures.get(&closure_scope).unwrap()
    }

    fn find_captured_variables(
        &self,
        _closure_scope: ScopeId,
        _scope_tree: &ScopeTree,
    ) -> Vec<CaptureInfo> {
        let mut captures = Vec::new();

        if let Some(closure) = _scope_tree.get_scope(_closure_scope) {
            if let Some(parent_id) = closure.parent {
                // Find variables used in closure but defined in parent scopes
                for (symbol_name, definitions) in &closure.symbols {
                    for definition in definitions {
                        // Check if this symbol is captured from outer scope
                        if self.is_captured_from_outer_scope(
                            symbol_name,
                            _closure_scope,
                            parent_id,
                            _scope_tree,
                        ) {
                            captures.push(CaptureInfo {
                                captured_symbol: symbol_name.clone(),
                                capture_type: self.infer_capture_type(definition),
                                source_scope: parent_id,
                                target_scope: _closure_scope,
                                definition: definition.clone(),
                            });
                        }
                    }
                }
            }
        }

        captures
    }

    fn is_captured_from_outer_scope(
        &self,
        symbol_name: &str,
        _closure_scope: ScopeId,
        parent_scope: ScopeId,
        scope_tree: &ScopeTree,
    ) -> bool {
        // Check if symbol is defined in parent scope but used in closure
        if let Some(parent) = scope_tree.get_scope(parent_scope) {
            parent.symbols.contains_key(symbol_name)
        } else {
            false
        }
    }

    fn infer_capture_type(&self, _definition: &Definition) -> CaptureType {
        // For now, default to ByValue. In real implementation,
        // this would analyze the usage context to determine capture type
        CaptureType::ByValue
    }

    pub fn get_captures(&self, scope_id: ScopeId) -> Option<&Vec<CaptureInfo>> {
        self.captures.get(&scope_id)
    }
}

pub struct ScopeChainWalker<'a> {
    current_scope: ScopeId,
    scope_tree: &'a ScopeTree,
    visited: HashSet<ScopeId>,
}

impl<'a> ScopeChainWalker<'a> {
    pub fn new(current_scope: ScopeId, scope_tree: &'a ScopeTree) -> Self {
        Self {
            current_scope,
            scope_tree,
            visited: HashSet::new(),
        }
    }

    pub fn walk_up(&mut self) -> Option<ScopeId> {
        if let Some(scope) = self.scope_tree.get_scope(self.current_scope) {
            if let Some(parent_id) = scope.parent {
                if !self.visited.contains(&parent_id) {
                    self.visited.insert(self.current_scope);
                    self.current_scope = parent_id;
                    return Some(parent_id);
                }
            }
        }
        None
    }

    pub fn find_symbol_in_chain(&self, symbol: &str) -> Option<(ScopeId, Definition)> {
        let mut walker = ScopeChainWalker::new(self.current_scope, self.scope_tree);

        while let Some(scope) = self.scope_tree.get_scope(walker.current_scope) {
            if let Some(definitions) = scope.get_symbols(symbol) {
                if let Some(definition) = definitions.first() {
                    return Some((walker.current_scope, definition.clone()));
                }
            }

            if walker.walk_up().is_none() {
                break;
            }
        }

        None
    }

    pub fn check_accessibility(&self, from_scope: ScopeId, to_scope: ScopeId) -> bool {
        if from_scope == to_scope {
            return true;
        }

        // Check if to_scope is an ancestor of from_scope (child can access parent)
        let mut current = from_scope;
        while let Some(scope) = self.scope_tree.get_scope(current) {
            if let Some(parent_id) = scope.parent {
                if parent_id == to_scope {
                    return true;
                }
                current = parent_id;
            } else {
                break;
            }
        }

        // Parent cannot access child scope directly
        false
    }
}

pub struct NestedScopeResolver {
    pub scope_tree: ScopeTree,
    pub closure_analyzer: ClosureAnalyzer,
}

impl NestedScopeResolver {
    pub fn new(scope_tree: ScopeTree) -> Self {
        Self {
            scope_tree,
            closure_analyzer: ClosureAnalyzer::new(),
        }
    }

    pub fn resolve_nested_access(&self, usage: &Usage) -> Vec<ScopeSearchResult> {
        let mut results = Vec::new();

        if let Some(usage_scope) = self.scope_tree.find_scope_at_position(&usage.position) {
            let walker = ScopeChainWalker::new(usage_scope, &self.scope_tree);

            if let Some((found_scope, definition)) = walker.find_symbol_in_chain(&usage.name) {
                let scope_distance = self.calculate_scope_distance(usage_scope, found_scope);
                results.push(ScopeSearchResult {
                    definition,
                    scope_id: found_scope,
                    scope_distance,
                });
            }

            // For now, skip closure capture analysis in read-only context
            // This would need a more sophisticated design to work with immutable borrows
        }

        results
    }

    pub fn find_in_scope_chain(&self, scope_id: ScopeId, symbol: &str) -> Option<Definition> {
        let walker = ScopeChainWalker::new(scope_id, &self.scope_tree);
        walker
            .find_symbol_in_chain(symbol)
            .map(|(_, definition)| definition)
    }

    pub fn get_accessible_scopes(&self, current_scope: ScopeId) -> Vec<ScopeId> {
        let mut accessible_scopes = Vec::new();
        let mut walker = ScopeChainWalker::new(current_scope, &self.scope_tree);

        accessible_scopes.push(current_scope);

        while let Some(parent_scope) = walker.walk_up() {
            accessible_scopes.push(parent_scope);
        }

        accessible_scopes
    }

    pub fn calculate_scope_distance(&self, from_scope: ScopeId, to_scope: ScopeId) -> usize {
        if from_scope == to_scope {
            return 0;
        }

        let mut distance = 0;
        let mut current = from_scope;

        while let Some(scope) = self.scope_tree.get_scope(current) {
            if let Some(parent_id) = scope.parent {
                distance += 1;
                if parent_id == to_scope {
                    return distance;
                }
                current = parent_id;
            } else {
                break;
            }
        }

        usize::MAX // Not found in parent chain
    }

    fn is_closure_scope(&self, scope_id: ScopeId) -> bool {
        // For now, consider function scopes as potential closures
        // In a real implementation, this would check if the function
        // is actually a closure (anonymous function, lambda, etc.)
        if let Some(scope) = self.scope_tree.get_scope(scope_id) {
            matches!(scope.scope_type, ScopeType::Function)
        } else {
            false
        }
    }

    pub fn analyze_complex_nesting(
        &mut self,
        root_scope: ScopeId,
    ) -> HashMap<ScopeId, Vec<CaptureInfo>> {
        let mut all_captures = HashMap::new();

        self.analyze_scope_recursively(root_scope, &mut all_captures);

        all_captures
    }

    fn analyze_scope_recursively(
        &mut self,
        scope_id: ScopeId,
        captures: &mut HashMap<ScopeId, Vec<CaptureInfo>>,
    ) {
        let child_scopes = if let Some(scope) = self.scope_tree.get_scope(scope_id) {
            // Analyze current scope for captures
            if self.is_closure_scope(scope_id) {
                let scope_tree = self.scope_tree.clone();
                let scope_captures = self
                    .closure_analyzer
                    .analyze_closure_captures(scope_id, &scope_tree);
                captures.insert(scope_id, scope_captures.clone());
            }

            scope.children.clone()
        } else {
            Vec::new()
        };

        // Recursively analyze child scopes
        for child_id in child_scopes {
            self.analyze_scope_recursively(child_id, captures);
        }
    }

    pub fn validate_nested_access(&self, usage: &Usage, definition: &Definition) -> bool {
        if let Some(usage_scope) = self.scope_tree.find_scope_at_position(&usage.position) {
            if let Some(def_scope) = self.scope_tree.find_scope_at_position(&definition.position) {
                let walker = ScopeChainWalker::new(usage_scope, &self.scope_tree);
                return walker.check_accessibility(usage_scope, def_scope);
            }
        }
        false
    }
}

// Additional scope utility functions from RustHelpers
use crate::models::{scope::SymbolTable, Position};

pub struct ScopeUtilities;

impl ScopeUtilities {
    /// Check if two nodes are in the same function scope
    pub fn are_in_same_function_scope(
        symbol_table: &SymbolTable,
        usage: &Usage,
        definition: &Definition,
    ) -> bool {
        // Find the enclosing function scope for both usage and definition
        let usage_function_scope =
            Self::find_enclosing_function_scope(symbol_table, &usage.position);
        let definition_function_scope =
            Self::find_enclosing_function_scope(symbol_table, &definition.position);

        match (usage_function_scope, definition_function_scope) {
            (Some(usage_scope), Some(def_scope)) => usage_scope == def_scope,
            (None, None) => true, // Both are at module level
            _ => false,           // One is in a function, the other isn't
        }
    }

    /// Check if usage_scope can access definition_scope
    pub fn is_scope_accessible(
        symbol_table: &SymbolTable,
        usage_scope: ScopeId,
        def_scope: ScopeId,
    ) -> bool {
        // Same scope is always accessible
        if usage_scope == def_scope {
            return true;
        }

        // Check if def_scope is an ancestor of usage_scope
        let mut current_scope = usage_scope;
        while let Some(scope) = symbol_table.scopes.get_scope(current_scope) {
            if let Some(parent_id) = scope.parent {
                if parent_id == def_scope {
                    return true;
                }
                current_scope = parent_id;
            } else {
                break;
            }
        }

        false
    }

    pub fn find_enclosing_function_scope(
        symbol_table: &SymbolTable,
        position: &Position,
    ) -> Option<ScopeId> {
        // Find the scope that contains this position
        if let Some(scope_id) = symbol_table.scopes.find_scope_at_position(position) {
            let mut current_scope_id = scope_id;

            // Walk up the scope chain to find a function scope
            while let Some(scope) = symbol_table.scopes.get_scope(current_scope_id) {
                if matches!(scope.scope_type, ScopeType::Function | ScopeType::Closure) {
                    return Some(current_scope_id);
                }

                if let Some(parent_id) = scope.parent {
                    current_scope_id = parent_id;
                } else {
                    break;
                }
            }
        }

        None
    }
}
