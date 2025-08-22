use crate::models::{
    scope::{ScopeId, ScopeTree, SymbolTable},
    Definition, Position, Usage,
};

#[derive(Debug, Clone)]
pub struct ShadowingWarning {
    pub shadowed_definition: Definition,
    pub shadowing_definition: Definition,
    pub scope_id: ScopeId,
    pub message: String,
}

#[derive(Debug, Clone)]
pub struct ResolutionCandidate {
    pub definition: Definition,
    pub scope_distance: usize,
    pub shadowing_level: usize,
    pub priority_score: f64,
    pub scope_id: ScopeId,
}

impl ResolutionCandidate {
    pub fn new(
        definition: Definition,
        scope_distance: usize,
        shadowing_level: usize,
        scope_id: ScopeId,
    ) -> Self {
        let priority_score = Self::calculate_priority_score(scope_distance, shadowing_level);
        Self {
            definition,
            scope_distance,
            shadowing_level,
            priority_score,
            scope_id,
        }
    }

    fn calculate_priority_score(scope_distance: usize, shadowing_level: usize) -> f64 {
        let distance_weight = 1.0 / (scope_distance as f64 + 1.0);
        let shadowing_weight = 10.0 / (shadowing_level as f64 + 1.0);
        distance_weight + shadowing_weight
    }
}

#[derive(Debug)]
pub struct ShadowingResolver {
    scope_tree: ScopeTree,
    pub symbol_table: SymbolTable,
}

impl ShadowingResolver {
    pub fn new(scope_tree: ScopeTree, symbol_table: SymbolTable) -> Self {
        Self {
            scope_tree,
            symbol_table,
        }
    }

    pub fn from_symbol_table(symbol_table: SymbolTable) -> Self {
        // Use the same scope tree reference from symbol table
        let scope_tree = symbol_table.scopes.clone();
        Self {
            scope_tree,
            symbol_table,
        }
    }

    pub fn resolve_shadowed_symbol(&self, usage: &Usage) -> Option<Definition> {
        let usage_scope_id = self.find_usage_scope(&usage.position)?;
        self.find_visible_definition(usage_scope_id, &usage.name)
    }

    pub fn find_visible_definition(&self, scope_id: ScopeId, symbol: &str) -> Option<Definition> {
        // Use SymbolTable's scope tree for lookup (most specific scope first)
        let mut current_scope_id = scope_id;
        while let Some(scope) = self.symbol_table.scopes.get_scope(current_scope_id) {
            if let Some(definitions) = scope.get_symbols(symbol) {
                if let Some(definition) = definitions.last() {
                    return Some(definition.clone());
                }
            }

            if let Some(parent_id) = scope.parent {
                current_scope_id = parent_id;
            } else {
                break;
            }
        }

        None
    }

    pub fn check_shadowing_conflicts(&self, scope_id: ScopeId) -> Vec<ShadowingWarning> {
        let mut warnings = Vec::new();

        if let Some(scope) = self.scope_tree.get_scope(scope_id) {
            for (symbol_name, definitions) in &scope.symbols {
                if definitions.len() > 1 {
                    for i in 1..definitions.len() {
                        let shadowing_def = &definitions[i];
                        let shadowed_def = &definitions[i - 1];

                        let warning = ShadowingWarning {
                            shadowed_definition: shadowed_def.clone(),
                            shadowing_definition: shadowing_def.clone(),
                            scope_id,
                            message: format!(
                                "Variable '{}' shadows previous definition at line {}",
                                symbol_name, shadowed_def.position.start_line
                            ),
                        };
                        warnings.push(warning);
                    }
                }

                if let Some(parent_definitions) =
                    self.find_parent_definitions(scope_id, symbol_name)
                {
                    for definition in definitions {
                        for parent_def in &parent_definitions {
                            let warning = ShadowingWarning {
                                shadowed_definition: parent_def.clone(),
                                shadowing_definition: definition.clone(),
                                scope_id,
                                message: format!(
                                    "Variable '{}' shadows outer scope definition at line {}",
                                    symbol_name, parent_def.position.start_line
                                ),
                            };
                            warnings.push(warning);
                        }
                    }
                }
            }
        }

        warnings
    }

    pub fn get_shadowing_chain(
        &self,
        scope_id: ScopeId,
        symbol: &str,
    ) -> Vec<(ScopeId, Definition)> {
        let mut chain = Vec::new();
        let mut current_scope_id = scope_id;

        while let Some(scope) = self.symbol_table.scopes.get_scope(current_scope_id) {
            if let Some(definitions) = scope.get_symbols(symbol) {
                for definition in definitions {
                    chain.push((current_scope_id, definition.clone()));
                }
            }

            if let Some(parent_id) = scope.parent {
                current_scope_id = parent_id;
            } else {
                break;
            }
        }

        // Also check symbol table entries for comprehensive chain
        if let Some(symbol_entries) = self.symbol_table.lookup_symbol(symbol) {
            for entry in symbol_entries {
                // Only include if not already in chain
                let already_exists = chain.iter().any(|(_, def)| {
                    def.name == entry.definition.name
                        && def.position.start_line == entry.definition.position.start_line
                        && def.position.start_column == entry.definition.position.start_column
                });

                if !already_exists {
                    chain.push((entry.scope_id, entry.definition.clone()));
                }
            }
        }

        chain
    }

    pub fn is_shadowed(&self, definition: &Definition, at_scope: ScopeId) -> bool {
        let def_scope_id = self.find_definition_scope(&definition.position);
        if def_scope_id.is_none() {
            return false;
        }

        let def_scope_id = def_scope_id.unwrap();

        if def_scope_id == at_scope {
            return false;
        }

        let chain = self.get_shadowing_chain(at_scope, &definition.name);

        for (scope_id, _) in chain {
            if scope_id != def_scope_id && self.is_descendant_scope(scope_id, def_scope_id) {
                return true;
            }
        }

        false
    }

    fn find_usage_scope(&self, position: &Position) -> Option<ScopeId> {
        self.symbol_table.scopes.find_scope_at_position(position)
    }

    fn find_definition_scope(&self, position: &Position) -> Option<ScopeId> {
        self.scope_tree.find_scope_at_position(position)
    }

    fn find_parent_definitions(
        &self,
        scope_id: ScopeId,
        symbol_name: &str,
    ) -> Option<Vec<Definition>> {
        if let Some(scope) = self.scope_tree.get_scope(scope_id) {
            if let Some(parent_id) = scope.parent {
                let parent_definitions = self
                    .scope_tree
                    .lookup_symbol_in_scope_chain(parent_id, symbol_name);
                if !parent_definitions.is_empty() {
                    return Some(parent_definitions.into_iter().cloned().collect());
                }
            }
        }
        None
    }

    fn is_descendant_scope(
        &self,
        potential_descendant: ScopeId,
        potential_ancestor: ScopeId,
    ) -> bool {
        let mut current_scope_id = potential_descendant;

        while let Some(scope) = self.scope_tree.get_scope(current_scope_id) {
            if let Some(parent_id) = scope.parent {
                if parent_id == potential_ancestor {
                    return true;
                }
                current_scope_id = parent_id;
            } else {
                break;
            }
        }

        false
    }
}

#[derive(Debug)]
pub struct PriorityCalculator;

impl PriorityCalculator {
    pub fn calculate_resolution_priority(
        &self,
        candidate: &ResolutionCandidate,
        usage_scope_id: ScopeId,
    ) -> f64 {
        let mut priority = candidate.priority_score;

        if candidate.scope_id == usage_scope_id {
            priority += 5.0;
        }

        priority -= candidate.scope_distance as f64 * 0.1;
        priority -= candidate.shadowing_level as f64 * 0.5;

        priority
    }
}

#[derive(Debug)]
pub struct NameResolutionEngine {
    pub shadowing_resolver: ShadowingResolver,
    priority_calculator: PriorityCalculator,
}

impl NameResolutionEngine {
    pub fn new(scope_tree: ScopeTree, symbol_table: SymbolTable) -> Self {
        Self {
            shadowing_resolver: ShadowingResolver::new(scope_tree, symbol_table),
            priority_calculator: PriorityCalculator,
        }
    }

    pub fn resolve_name(&self, usage: &Usage) -> Vec<ResolutionCandidate> {
        let usage_scope_id = self.shadowing_resolver.find_usage_scope(&usage.position);
        if usage_scope_id.is_none() {
            return Vec::new();
        }
        let usage_scope_id = usage_scope_id.unwrap();

        let chain = self
            .shadowing_resolver
            .get_shadowing_chain(usage_scope_id, &usage.name);
        let mut candidates = Vec::new();

        for (scope_distance, (scope_id, definition)) in chain.into_iter().enumerate() {
            let shadowing_level = self.calculate_shadowing_level(scope_id, &usage.name);
            let candidate =
                ResolutionCandidate::new(definition, scope_distance, shadowing_level, scope_id);
            candidates.push(candidate);
        }

        candidates.sort_by(|a, b| {
            let priority_a = self
                .priority_calculator
                .calculate_resolution_priority(a, usage_scope_id);
            let priority_b = self
                .priority_calculator
                .calculate_resolution_priority(b, usage_scope_id);
            priority_b
                .partial_cmp(&priority_a)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        candidates
    }

    pub fn select_best_candidate<'a>(
        &self,
        candidates: &'a [ResolutionCandidate],
    ) -> Option<&'a ResolutionCandidate> {
        candidates.first()
    }

    fn calculate_shadowing_level(&self, scope_id: ScopeId, symbol: &str) -> usize {
        let chain = self
            .shadowing_resolver
            .get_shadowing_chain(scope_id, symbol);
        chain.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{
        scope::{Accessibility, ScopeType},
        DefinitionType,
    };

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

    #[test]
    fn test_shadowing_resolver_basic() {
        let mut symbol_table = SymbolTable::new();

        let func_scope_id = symbol_table.scopes.create_scope(
            Some(0),
            ScopeType::Function,
            create_test_position(5, 1),
            create_test_position(10, 10),
        );

        let outer_def = create_test_definition("x", 1);
        let inner_def = create_test_definition("x", 6);

        symbol_table.add_symbol(
            "x".to_string(),
            outer_def,
            0,
            Accessibility::ScopeLocal,
            false,
        );

        symbol_table.add_symbol(
            "x".to_string(),
            inner_def.clone(),
            func_scope_id,
            Accessibility::ScopeLocal,
            false,
        );

        let resolver = ShadowingResolver::from_symbol_table(symbol_table);
        let usage = Usage::new_simple(
            "x".to_string(),
            create_test_position(7, 5),
            crate::models::usage::UsageKind::Identifier,
        );

        let resolved = resolver.resolve_shadowed_symbol(&usage);
        assert!(resolved.is_some());
        assert_eq!(resolved.unwrap().position.start_line, 6);
    }

    #[test]
    fn test_shadowing_warnings() {
        let mut symbol_table = SymbolTable::new();

        let func_scope_id = symbol_table.scopes.create_scope(
            Some(0),
            ScopeType::Function,
            create_test_position(5, 1),
            create_test_position(10, 1),
        );

        let outer_def = create_test_definition("var", 1);
        let inner_def = create_test_definition("var", 6);

        symbol_table.add_symbol(
            "var".to_string(),
            outer_def,
            0,
            Accessibility::ScopeLocal,
            false,
        );

        symbol_table.add_symbol(
            "var".to_string(),
            inner_def,
            func_scope_id,
            Accessibility::ScopeLocal,
            false,
        );

        let resolver = ShadowingResolver::from_symbol_table(symbol_table);
        let warnings = resolver.check_shadowing_conflicts(func_scope_id);

        assert!(!warnings.is_empty());
        assert!(warnings[0].message.contains("shadows outer scope"));
    }

    #[test]
    fn test_name_resolution_engine() {
        let mut symbol_table = SymbolTable::new();

        let func_scope_id = symbol_table.scopes.create_scope(
            Some(0),
            ScopeType::Function,
            create_test_position(5, 1),
            create_test_position(10, 1),
        );

        let outer_def = create_test_definition("test_var", 1);
        let inner_def = create_test_definition("test_var", 6);

        symbol_table.add_symbol(
            "test_var".to_string(),
            outer_def,
            0,
            Accessibility::ScopeLocal,
            false,
        );

        symbol_table.add_symbol(
            "test_var".to_string(),
            inner_def.clone(),
            func_scope_id,
            Accessibility::ScopeLocal,
            false,
        );

        let engine = NameResolutionEngine::new(symbol_table.scopes.clone(), symbol_table);

        let usage = Usage::new_simple(
            "test_var".to_string(),
            create_test_position(7, 5),
            crate::models::usage::UsageKind::Identifier,
        );

        let candidates = engine.resolve_name(&usage);
        assert!(!candidates.is_empty());

        let best_candidate = engine.select_best_candidate(&candidates);
        assert!(best_candidate.is_some());
        assert_eq!(best_candidate.unwrap().definition.position.start_line, 6);
    }
}
