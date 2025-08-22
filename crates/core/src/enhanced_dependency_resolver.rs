use crate::dependency_resolver::DependencyResolver;
use crate::models::{
    scope::{ScopeId, SymbolTable},
    Definition, Dependency, Usage,
};
use crate::module_resolver::ModuleResolver;
use crate::nested_scope_resolver::NestedScopeResolver;
use tree_sitter::Node;

/// Enhanced dependency resolver that combines:
/// - Base dependency resolution
/// - Scope awareness (#107)
/// - Nested scope resolution (#102/#108)  
/// - Shadowing resolution (#103)
/// - Module system integration and visibility rules (#104)
pub struct EnhancedDependencyResolver {
    symbol_table: SymbolTable,
    nested_scope_resolver: NestedScopeResolver,
    module_resolver: ModuleResolver,
    #[allow(dead_code)]
    language: String,
}

#[derive(Debug, Clone)]
pub struct ShadowingWarning {
    pub message: String,
    pub shadowing_definition: Definition,
    pub shadowed_definition: Definition,
}

#[derive(Debug, Clone)]
pub struct ResolutionCandidate {
    pub definition: Definition,
    pub scope_distance: usize,
    pub shadowing_level: usize,
    pub priority_score: f64,
}

impl ResolutionCandidate {
    pub fn new(
        definition: Definition,
        _scope_id: ScopeId,
        scope_distance: usize,
        shadowing_level: usize,
    ) -> Self {
        let priority_score = Self::calculate_priority_score(scope_distance, shadowing_level);
        Self {
            definition,
            scope_distance,
            shadowing_level,
            priority_score,
        }
    }

    fn calculate_priority_score(scope_distance: usize, shadowing_level: usize) -> f64 {
        let distance_weight = 1.0 / (scope_distance as f64 + 1.0);
        let shadowing_weight = 10.0 / (shadowing_level as f64 + 1.0);
        distance_weight + shadowing_weight
    }
}

impl EnhancedDependencyResolver {
    pub fn new(symbol_table: SymbolTable, language: String) -> Self {
        let nested_scope_resolver = NestedScopeResolver::new(symbol_table.scopes.clone());
        let module_resolver = ModuleResolver::new();

        Self {
            symbol_table,
            nested_scope_resolver,
            module_resolver,
            language,
        }
    }

    pub fn with_module_resolver(
        symbol_table: SymbolTable,
        language: String,
        module_resolver: ModuleResolver,
    ) -> Self {
        let nested_scope_resolver = NestedScopeResolver::new(symbol_table.scopes.clone());

        Self {
            symbol_table,
            nested_scope_resolver,
            module_resolver,
            language,
        }
    }

    pub fn get_module_resolver(&self) -> &ModuleResolver {
        &self.module_resolver
    }

    pub fn get_module_resolver_mut(&mut self) -> &mut ModuleResolver {
        &mut self.module_resolver
    }

    /// Resolve symbol with shadowing awareness
    pub fn resolve_shadowed_symbol(&self, usage: &Usage) -> Option<Definition> {
        // Find the scope containing this usage
        let usage_scope_id = self
            .symbol_table
            .scopes
            .find_scope_at_position(&usage.position)?;

        // Get all candidates for this symbol
        let candidates = self.resolve_name_candidates(usage, usage_scope_id);

        // Select the best candidate based on priority
        self.select_best_candidate(&candidates)
            .map(|c| c.definition.clone())
    }

    /// Get all resolution candidates for a symbol
    fn resolve_name_candidates(
        &self,
        usage: &Usage,
        scope_id: ScopeId,
    ) -> Vec<ResolutionCandidate> {
        let mut candidates = Vec::new();
        let mut current_scope_id = scope_id;
        let mut scope_distance = 0;

        // Traverse scope chain to find all possible matches
        while let Some(scope) = self.symbol_table.scopes.get_scope(current_scope_id) {
            if let Some(definitions) = scope.symbols.get(&usage.name) {
                for (index, definition) in definitions.iter().enumerate() {
                    let candidate = ResolutionCandidate::new(
                        definition.clone(),
                        current_scope_id,
                        scope_distance,
                        index, // Shadowing level within this scope
                    );
                    candidates.push(candidate);
                }
            }

            if let Some(parent_id) = scope.parent {
                current_scope_id = parent_id;
                scope_distance += 1;
            } else {
                break;
            }
        }

        // Sort by priority score (highest first)
        candidates.sort_by(|a, b| b.priority_score.partial_cmp(&a.priority_score).unwrap());
        candidates
    }

    /// Select the best candidate from available options
    fn select_best_candidate<'a>(
        &self,
        candidates: &'a [ResolutionCandidate],
    ) -> Option<&'a ResolutionCandidate> {
        candidates.first()
    }

    /// Check for shadowing conflicts in a scope
    pub fn check_shadowing_conflicts(&self, scope_id: ScopeId) -> Vec<ShadowingWarning> {
        let mut warnings = Vec::new();

        let Some(scope) = self.symbol_table.scopes.get_scope(scope_id) else {
            return warnings;
        };

        // Check each symbol in this scope
        for (symbol_name, definitions) in &scope.symbols {
            // Look for the same symbol in parent scopes
            if let Some(parent_definitions) = self.find_parent_symbols(scope_id, symbol_name) {
                for current_def in definitions {
                    for parent_def in &parent_definitions {
                        let warning = ShadowingWarning {
                            message: format!(
                                "Symbol '{}' at line {} shadows outer scope definition at line {}",
                                symbol_name,
                                current_def.position.start_line,
                                parent_def.position.start_line
                            ),
                            shadowing_definition: current_def.clone(),
                            shadowed_definition: parent_def.clone(),
                        };
                        warnings.push(warning);
                    }
                }
            }
        }

        warnings
    }

    /// Find symbols with the same name in parent scopes
    fn find_parent_symbols(&self, scope_id: ScopeId, symbol_name: &str) -> Option<Vec<Definition>> {
        let scope = self.symbol_table.scopes.get_scope(scope_id)?;
        let parent_id = scope.parent?;

        let mut parent_definitions = Vec::new();
        let mut current_scope_id = parent_id;

        while let Some(parent_scope) = self.symbol_table.scopes.get_scope(current_scope_id) {
            if let Some(definitions) = parent_scope.symbols.get(symbol_name) {
                parent_definitions.extend(definitions.clone());
            }

            if let Some(next_parent) = parent_scope.parent {
                current_scope_id = next_parent;
            } else {
                break;
            }
        }

        if parent_definitions.is_empty() {
            None
        } else {
            Some(parent_definitions)
        }
    }

    /// Get nested scope information using nested scope resolver
    pub fn analyze_nested_scopes(&self, scope_id: ScopeId) -> bool {
        // Placeholder for nested scope analysis - to be integrated properly
        self.nested_scope_resolver
            .scope_tree
            .get_scope(scope_id)
            .is_some()
    }

    /// Calculate scope distance between two scopes
    pub fn calculate_scope_distance(
        &self,
        from_scope: ScopeId,
        to_scope: ScopeId,
    ) -> Option<usize> {
        if from_scope == to_scope {
            return Some(0);
        }

        let mut distance = 0;
        let mut current_scope = from_scope;

        while let Some(scope) = self.symbol_table.scopes.get_scope(current_scope) {
            if current_scope == to_scope {
                return Some(distance);
            }

            if let Some(parent_id) = scope.parent {
                current_scope = parent_id;
                distance += 1;
            } else {
                break;
            }
        }

        None
    }

    /// Filter dependencies by module visibility rules
    fn filter_by_module_visibility(
        &self,
        _source_code: &str,
        _root_node: Node,
        dependencies: Vec<Dependency>,
        _definitions: &[Definition],
    ) -> Result<Vec<Dependency>, String> {
        // For now, return all dependencies without filtering
        // TODO: Implement actual module visibility checking
        Ok(dependencies)
    }
}

impl DependencyResolver for EnhancedDependencyResolver {
    fn resolve_dependencies(
        &self,
        source_code: &str,
        root_node: Node,
        usage_nodes: &[Usage],
        definitions: &[Definition],
    ) -> Result<Vec<Dependency>, String> {
        let mut all_dependencies = Vec::new();

        for usage_node in usage_nodes {
            let mut deps =
                self.resolve_single_dependency(source_code, root_node, usage_node, definitions);

            // Filter by module visibility
            deps = self.filter_by_module_visibility(source_code, root_node, deps, definitions)?;

            all_dependencies.append(&mut deps);
        }

        Ok(all_dependencies)
    }

    fn resolve_single_dependency(
        &self,
        _source_code: &str,
        _root_node: Node,
        usage_node: &Usage,
        definitions: &[Definition],
    ) -> Vec<Dependency> {
        let mut dependencies = Vec::new();

        // Try shadowing-aware resolution first
        if let Some(resolved_def) = self.resolve_shadowed_symbol(usage_node) {
            let dependency = Dependency {
                source_line: usage_node.position.start_line,
                target_line: resolved_def.position.start_line,
                symbol: usage_node.name.clone(),
                dependency_type: self.get_dependency_type(usage_node),
                context: self.get_context(usage_node),
            };
            dependencies.push(dependency);
            return dependencies;
        }

        // Fallback to simple name matching
        for definition in definitions {
            if definition.name == usage_node.name {
                let dependency = Dependency {
                    source_line: usage_node.position.start_line,
                    target_line: definition.position.start_line,
                    symbol: usage_node.name.clone(),
                    dependency_type: self.get_dependency_type(usage_node),
                    context: self.get_context(usage_node),
                };
                dependencies.push(dependency);
            }
        }

        dependencies
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{scope::ScopeType, DefinitionType, Position, UsageKind};

    fn create_test_position(line: usize, column: usize) -> Position {
        Position {
            start_line: line,
            start_column: column,
            end_line: line,
            end_column: column + 1,
        }
    }

    fn create_test_definition(name: &str, line: usize) -> Definition {
        Definition {
            name: name.to_string(),
            definition_type: DefinitionType::VariableDefinition,
            position: create_test_position(line, 1),
        }
    }

    fn create_test_usage(name: &str, line: usize) -> Usage {
        Usage {
            name: name.to_string(),
            kind: UsageKind::Identifier,
            position: create_test_position(line, 5),
        }
    }

    #[test]
    fn test_enhanced_resolver_basic_resolution() {
        let mut symbol_table = SymbolTable::new();

        let func_scope_id = symbol_table.scopes.create_scope(
            Some(0),
            ScopeType::Function,
            create_test_position(5, 1),
            create_test_position(15, 1),
        );

        let outer_def = create_test_definition("x", 1);
        let inner_def = create_test_definition("x", 8);

        symbol_table.add_symbol(
            "x".to_string(),
            outer_def,
            0,
            crate::models::scope::Accessibility::ScopeLocal,
            false,
        );
        symbol_table.add_symbol(
            "x".to_string(),
            inner_def.clone(),
            func_scope_id,
            crate::models::scope::Accessibility::ScopeLocal,
            false,
        );

        let resolver = EnhancedDependencyResolver::new(symbol_table, "rust".to_string());
        let usage = create_test_usage("x", 10);

        let resolved = resolver.resolve_shadowed_symbol(&usage);
        assert!(resolved.is_some());
        assert_eq!(resolved.unwrap().position.start_line, 8);
    }

    #[test]
    fn test_enhanced_resolver_shadowing_detection() {
        let mut symbol_table = SymbolTable::new();

        let func_scope_id = symbol_table.scopes.create_scope(
            Some(0),
            ScopeType::Function,
            create_test_position(5, 1),
            create_test_position(15, 1),
        );

        let outer_def = create_test_definition("var", 1);
        let inner_def = create_test_definition("var", 8);

        symbol_table.add_symbol(
            "var".to_string(),
            outer_def,
            0,
            crate::models::scope::Accessibility::ScopeLocal,
            false,
        );
        symbol_table.add_symbol(
            "var".to_string(),
            inner_def,
            func_scope_id,
            crate::models::scope::Accessibility::ScopeLocal,
            false,
        );

        let resolver = EnhancedDependencyResolver::new(symbol_table, "rust".to_string());
        let warnings = resolver.check_shadowing_conflicts(func_scope_id);

        assert!(!warnings.is_empty());
        assert!(warnings[0].message.contains("shadows outer scope"));
    }
}
