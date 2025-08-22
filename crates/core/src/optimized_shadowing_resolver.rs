use crate::models::{
    scope::{ScopeId, ScopeTree, SymbolTable},
    Definition, Position, Usage,
};
use crate::shadowing_resolver::ShadowingResolver;
use std::collections::HashMap;

#[derive(Debug, Clone)]
struct CachedResolution {
    definition: Definition,
    #[allow(dead_code)]
    scope_distance: usize,
    #[allow(dead_code)]
    timestamp: u64,
}

#[derive(Debug)]
struct ScopeIndex {
    // Map from scope id to all its symbols
    scope_symbols: HashMap<ScopeId, HashMap<String, Vec<Definition>>>,
    // Map from symbol name to all scopes containing it
    symbol_scopes: HashMap<String, Vec<ScopeId>>,
    // Map from scope id to its parent chain
    scope_chains: HashMap<ScopeId, Vec<ScopeId>>,
}

impl ScopeIndex {
    fn new() -> Self {
        Self {
            scope_symbols: HashMap::new(),
            symbol_scopes: HashMap::new(),
            scope_chains: HashMap::new(),
        }
    }

    fn build_from_symbol_table(&mut self, symbol_table: &SymbolTable) {
        // Clear existing indexes
        self.scope_symbols.clear();
        self.symbol_scopes.clear();
        self.scope_chains.clear();

        // Build scope_symbols index
        for (scope_id, scope) in &symbol_table.scopes.scopes {
            self.scope_symbols.insert(*scope_id, scope.symbols.clone());

            // Build scope chain for this scope
            let mut chain = Vec::new();
            let mut current_id = *scope_id;

            while let Some(scope) = symbol_table.scopes.get_scope(current_id) {
                chain.push(current_id);
                if let Some(parent_id) = scope.parent {
                    current_id = parent_id;
                } else {
                    break;
                }
            }

            self.scope_chains.insert(*scope_id, chain);
        }

        // Build symbol_scopes index
        for (scope_id, symbols) in &self.scope_symbols {
            for symbol_name in symbols.keys() {
                self.symbol_scopes
                    .entry(symbol_name.clone())
                    .or_default()
                    .push(*scope_id);
            }
        }
    }

    fn get_symbol_in_scope_chain(
        &self,
        scope_id: ScopeId,
        symbol_name: &str,
    ) -> Option<&Definition> {
        if let Some(chain) = self.scope_chains.get(&scope_id) {
            for &chain_scope_id in chain {
                if let Some(symbols) = self.scope_symbols.get(&chain_scope_id) {
                    if let Some(definitions) = symbols.get(symbol_name) {
                        if let Some(definition) = definitions.last() {
                            return Some(definition);
                        }
                    }
                }
            }
        }
        None
    }

    fn get_all_scopes_with_symbol(&self, symbol_name: &str) -> Vec<ScopeId> {
        self.symbol_scopes
            .get(symbol_name)
            .cloned()
            .unwrap_or_default()
    }
}

#[derive(Debug)]
pub struct OptimizedShadowingResolver {
    base_resolver: ShadowingResolver,
    resolution_cache: HashMap<String, CachedResolution>,
    scope_index: ScopeIndex,
    cache_hits: usize,
    cache_misses: usize,
    cache_timestamp: u64,
}

impl OptimizedShadowingResolver {
    pub fn new(_scope_tree: ScopeTree, symbol_table: SymbolTable) -> Self {
        let base_resolver = ShadowingResolver::from_symbol_table(symbol_table.clone());
        let mut scope_index = ScopeIndex::new();
        scope_index.build_from_symbol_table(&symbol_table);

        Self {
            base_resolver,
            resolution_cache: HashMap::new(),
            scope_index,
            cache_hits: 0,
            cache_misses: 0,
            cache_timestamp: 0,
        }
    }

    pub fn with_cache_size(self, _max_size: usize) -> Self {
        // In a production implementation, you would implement LRU cache with max size
        // For now, we'll use unlimited cache
        self
    }

    pub fn resolve_shadowed_symbol_optimized(&mut self, usage: &Usage) -> Option<Definition> {
        let cache_key = self.create_cache_key(usage);

        // Check cache first
        if let Some(cached) = self.resolution_cache.get(&cache_key) {
            self.cache_hits += 1;
            return Some(cached.definition.clone());
        }

        self.cache_misses += 1;

        // Find usage scope efficiently
        let usage_scope_id = self.find_usage_scope_optimized(&usage.position)?;

        // Use optimized lookup
        if let Some(definition) = self
            .scope_index
            .get_symbol_in_scope_chain(usage_scope_id, &usage.name)
        {
            let scope_distance =
                self.calculate_scope_distance_optimized(usage_scope_id, &definition.position);

            // Cache the result
            let cached_resolution = CachedResolution {
                definition: definition.clone(),
                scope_distance,
                timestamp: self.cache_timestamp,
            };

            self.resolution_cache.insert(cache_key, cached_resolution);
            self.cache_timestamp += 1;

            Some(definition.clone())
        } else {
            None
        }
    }

    pub fn find_visible_definition_optimized(
        &self,
        scope_id: ScopeId,
        symbol: &str,
    ) -> Option<Definition> {
        self.scope_index
            .get_symbol_in_scope_chain(scope_id, symbol)
            .cloned()
    }

    pub fn get_shadowing_chain_optimized(
        &self,
        scope_id: ScopeId,
        symbol: &str,
    ) -> Vec<(ScopeId, Definition)> {
        let mut chain = Vec::new();

        if let Some(scope_chain) = self.scope_index.scope_chains.get(&scope_id) {
            for &chain_scope_id in scope_chain {
                if let Some(symbols) = self.scope_index.scope_symbols.get(&chain_scope_id) {
                    if let Some(definitions) = symbols.get(symbol) {
                        for definition in definitions {
                            chain.push((chain_scope_id, definition.clone()));
                        }
                    }
                }
            }
        }

        chain
    }

    pub fn batch_resolve_symbols(
        &mut self,
        usages: &[Usage],
    ) -> HashMap<String, Option<Definition>> {
        let mut results = HashMap::new();

        // Group usages by symbol name for efficient processing
        let mut symbol_groups: HashMap<String, Vec<&Usage>> = HashMap::new();
        for usage in usages {
            symbol_groups
                .entry(usage.name.clone())
                .or_default()
                .push(usage);
        }

        // Process each symbol group
        for (symbol_name, symbol_usages) in symbol_groups {
            // Get all scopes that contain this symbol
            let _scopes_with_symbol = self.scope_index.get_all_scopes_with_symbol(&symbol_name);

            for usage in symbol_usages {
                let cache_key = self.create_cache_key(usage);
                let resolved = self.resolve_shadowed_symbol_optimized(usage);
                results.insert(cache_key, resolved);
            }
        }

        results
    }

    pub fn invalidate_cache(&mut self) {
        self.resolution_cache.clear();
        self.cache_timestamp = 0;
    }

    pub fn rebuild_index(&mut self, symbol_table: &SymbolTable) {
        self.scope_index.build_from_symbol_table(symbol_table);
        self.invalidate_cache();
    }

    pub fn get_cache_stats(&self) -> (usize, usize, f64) {
        let total = self.cache_hits + self.cache_misses;
        let hit_rate = if total > 0 {
            self.cache_hits as f64 / total as f64
        } else {
            0.0
        };
        (self.cache_hits, self.cache_misses, hit_rate)
    }

    fn create_cache_key(&self, usage: &Usage) -> String {
        format!(
            "{}:{}:{}",
            usage.name, usage.position.start_line, usage.position.start_column
        )
    }

    fn find_usage_scope_optimized(&self, position: &Position) -> Option<ScopeId> {
        // Use base resolver's logic for now, but this could be optimized with spatial indexing
        self.base_resolver
            .symbol_table
            .scopes
            .find_scope_at_position(position)
    }

    fn calculate_scope_distance_optimized(
        &self,
        from_scope: ScopeId,
        to_position: &Position,
    ) -> usize {
        if let Some(to_scope) = self.find_usage_scope_optimized(to_position) {
            if let Some(chain) = self.scope_index.scope_chains.get(&from_scope) {
                for (distance, &scope_id) in chain.iter().enumerate() {
                    if scope_id == to_scope {
                        return distance;
                    }
                }
            }
        }
        0
    }
}

// Performance monitoring utilities
#[derive(Debug, Clone)]
pub struct PerformanceMetrics {
    pub resolution_time_ms: u64,
    pub cache_hit_rate: f64,
    pub symbols_processed: usize,
    pub scopes_traversed: usize,
}

impl Default for PerformanceMetrics {
    fn default() -> Self {
        Self::new()
    }
}

impl PerformanceMetrics {
    pub fn new() -> Self {
        Self {
            resolution_time_ms: 0,
            cache_hit_rate: 0.0,
            symbols_processed: 0,
            scopes_traversed: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{
        scope::{Accessibility, ScopeType},
        usage::UsageKind,
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

    fn create_test_usage(name: &str, line: usize) -> Usage {
        Usage {
            name: name.to_string(),
            kind: UsageKind::Identifier,
            position: create_test_position(line, 5),
        }
    }

    #[test]
    fn test_scope_index_building() {
        let mut symbol_table = SymbolTable::new();

        let func_scope_id = symbol_table.scopes.create_scope(
            Some(0),
            ScopeType::Function,
            create_test_position(5, 1),
            create_test_position(15, 1),
        );

        let def1 = create_test_definition("var", 1);
        let def2 = create_test_definition("var", 8);

        symbol_table.add_symbol("var".to_string(), def1, 0, Accessibility::ScopeLocal, false);
        symbol_table.add_symbol(
            "var".to_string(),
            def2,
            func_scope_id,
            Accessibility::ScopeLocal,
            false,
        );

        let mut index = ScopeIndex::new();
        index.build_from_symbol_table(&symbol_table);

        // Test scope chain
        assert!(index.scope_chains.contains_key(&func_scope_id));
        assert!(index.scope_chains.contains_key(&0));

        // Test symbol scopes
        assert!(index.symbol_scopes.contains_key("var"));
        let var_scopes = index.symbol_scopes.get("var").unwrap();
        assert!(var_scopes.contains(&0));
        assert!(var_scopes.contains(&func_scope_id));
    }

    #[test]
    fn test_optimized_resolution_with_caching() {
        let mut symbol_table = SymbolTable::new();

        let func_scope_id = symbol_table.scopes.create_scope(
            Some(0),
            ScopeType::Function,
            create_test_position(5, 1),
            create_test_position(15, 1),
        );

        let outer_def = create_test_definition("cached_var", 1);
        let inner_def = create_test_definition("cached_var", 8);

        symbol_table.add_symbol(
            "cached_var".to_string(),
            outer_def,
            0,
            Accessibility::ScopeLocal,
            false,
        );
        symbol_table.add_symbol(
            "cached_var".to_string(),
            inner_def.clone(),
            func_scope_id,
            Accessibility::ScopeLocal,
            false,
        );

        let mut resolver =
            OptimizedShadowingResolver::new(symbol_table.scopes.clone(), symbol_table);

        let usage = create_test_usage("cached_var", 10);

        // First resolution - should be cache miss
        let result1 = resolver.resolve_shadowed_symbol_optimized(&usage);
        let (hits1, misses1, _) = resolver.get_cache_stats();
        assert_eq!(hits1, 0);
        assert_eq!(misses1, 1);

        // Second resolution - should be cache hit
        let result2 = resolver.resolve_shadowed_symbol_optimized(&usage);
        let (hits2, misses2, hit_rate) = resolver.get_cache_stats();
        assert_eq!(hits2, 1);
        assert_eq!(misses2, 1);
        assert!(hit_rate > 0.0);

        // Results should be identical
        assert_eq!(result1, result2);
    }

    #[test]
    fn test_batch_symbol_resolution() {
        let mut symbol_table = SymbolTable::new();

        let _func_scope_id = symbol_table.scopes.create_scope(
            Some(0),
            ScopeType::Function,
            create_test_position(5, 1),
            create_test_position(25, 1),
        );

        // Add multiple symbols
        for i in 0..5 {
            let def_name = format!("batch_var_{}", i);
            let def = create_test_definition(&def_name, i + 1);
            symbol_table.add_symbol(def_name, def, 0, Accessibility::ScopeLocal, false);
        }

        let mut resolver =
            OptimizedShadowingResolver::new(symbol_table.scopes.clone(), symbol_table);

        // Create batch of usages
        let usages: Vec<Usage> = (0..5)
            .map(|i| create_test_usage(&format!("batch_var_{}", i), 10))
            .collect();

        let results = resolver.batch_resolve_symbols(&usages);
        assert_eq!(results.len(), 5);

        // All should be resolved
        for (_, result) in results {
            assert!(result.is_some());
        }
    }

    #[test]
    fn test_cache_invalidation() {
        let mut symbol_table = SymbolTable::new();
        let def = create_test_definition("invalidate_test", 1);
        symbol_table.add_symbol(
            "invalidate_test".to_string(),
            def,
            0,
            Accessibility::ScopeLocal,
            false,
        );

        let mut resolver =
            OptimizedShadowingResolver::new(symbol_table.scopes.clone(), symbol_table.clone());

        let usage = create_test_usage("invalidate_test", 5);

        // Populate cache
        resolver.resolve_shadowed_symbol_optimized(&usage);
        let (hits_before, misses_before, _) = resolver.get_cache_stats();

        // Invalidate cache
        resolver.invalidate_cache();

        // Check stats after invalidation (stats are preserved)
        let (hits_after, misses_after, _) = resolver.get_cache_stats();
        assert_eq!(hits_after, hits_before);
        assert_eq!(misses_after, misses_before);

        // Resolve again - should be cache miss since cache was cleared
        resolver.resolve_shadowed_symbol_optimized(&usage);
        let (hits_final, misses_final, _) = resolver.get_cache_stats();

        // Cache was invalidated, so second resolution should be a miss
        assert_eq!(hits_final, hits_after); // No new cache hits
        assert_eq!(misses_final, misses_after + 1); // One additional cache miss
    }
}
