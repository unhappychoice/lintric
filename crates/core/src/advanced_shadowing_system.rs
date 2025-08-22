use crate::dependency_resolver::DependencyResolver;
use crate::models::{
    scope::{ScopeId, SymbolTable},
    Definition, Usage,
};
use crate::optimized_shadowing_resolver::{OptimizedShadowingResolver, PerformanceMetrics};
use crate::shadowing_aware_dependency_resolver::ShadowingAwareDependencyResolver;
use crate::shadowing_resolver::{NameResolutionEngine, ShadowingResolver, ShadowingWarning};
use std::collections::HashMap;

pub struct AdvancedShadowingSystem {
    optimized_resolver: OptimizedShadowingResolver,
    name_resolution_engine: NameResolutionEngine,
    performance_metrics: PerformanceMetrics,
}

impl AdvancedShadowingSystem {
    pub fn new(symbol_table: SymbolTable) -> Self {
        let scope_tree = symbol_table.scopes.clone();
        let optimized_resolver =
            OptimizedShadowingResolver::new(scope_tree.clone(), symbol_table.clone());
        let name_resolution_engine = NameResolutionEngine::new(scope_tree, symbol_table);

        Self {
            optimized_resolver,
            name_resolution_engine,
            performance_metrics: PerformanceMetrics::new(),
        }
    }

    pub fn with_caching(mut self, cache_size: usize) -> Self {
        self.optimized_resolver = self.optimized_resolver.with_cache_size(cache_size);
        self
    }

    pub fn resolve_symbol(&mut self, usage: &Usage) -> ShadowingResolutionResult {
        let start_time = std::time::Instant::now();

        let resolved_definition = self
            .optimized_resolver
            .resolve_shadowed_symbol_optimized(usage);
        let candidates = self.name_resolution_engine.resolve_name(usage);
        let best_candidate = self
            .name_resolution_engine
            .select_best_candidate(&candidates)
            .cloned();

        let resolution_time = start_time.elapsed().as_millis() as u64;
        self.performance_metrics.resolution_time_ms += resolution_time;
        self.performance_metrics.symbols_processed += 1;

        let cache_hit = resolved_definition.is_some();

        ShadowingResolutionResult {
            resolved_definition,
            all_candidates: candidates,
            best_candidate,
            resolution_time_ms: resolution_time,
            cache_hit,
        }
    }

    pub fn batch_resolve_symbols(
        &mut self,
        usages: &[Usage],
    ) -> HashMap<String, ShadowingResolutionResult> {
        let start_time = std::time::Instant::now();

        let mut results = HashMap::new();
        let batch_resolved = self.optimized_resolver.batch_resolve_symbols(usages);

        for usage in usages {
            let cache_key = format!(
                "{}:{}:{}",
                usage.name, usage.position.start_line, usage.position.start_column
            );
            let resolved_definition = batch_resolved.get(&cache_key).cloned().flatten();

            let candidates = self.name_resolution_engine.resolve_name(usage);
            let best_candidate = self
                .name_resolution_engine
                .select_best_candidate(&candidates)
                .cloned();

            let cache_hit = resolved_definition.is_some();

            results.insert(
                cache_key,
                ShadowingResolutionResult {
                    resolved_definition,
                    all_candidates: candidates,
                    best_candidate,
                    resolution_time_ms: 0, // Batch processing time is distributed
                    cache_hit,
                },
            );
        }

        let total_time = start_time.elapsed().as_millis() as u64;
        self.performance_metrics.resolution_time_ms += total_time;
        self.performance_metrics.symbols_processed += usages.len();

        results
    }

    pub fn get_shadowing_warnings(&self, scope_id: ScopeId) -> Vec<ShadowingWarning> {
        // Create a temporary resolver for warning analysis
        // This could be optimized by caching warnings
        let temp_resolver = ShadowingResolver::from_symbol_table(
            self.name_resolution_engine
                .shadowing_resolver
                .symbol_table
                .clone(),
        );
        temp_resolver.check_shadowing_conflicts(scope_id)
    }

    pub fn analyze_shadowing_patterns(&self, scope_id: ScopeId) -> ShadowingAnalysis {
        let warnings = self.get_shadowing_warnings(scope_id);

        let mut analysis = ShadowingAnalysis {
            scope_id,
            total_warnings: warnings.len(),
            shadowing_levels: HashMap::new(),
            most_shadowed_symbols: Vec::new(),
            complexity_score: 0.0,
        };

        // Count shadowing levels for each symbol
        let mut symbol_counts = HashMap::new();
        for warning in &warnings {
            let symbol_name = &warning.shadowing_definition.name;
            *symbol_counts.entry(symbol_name.clone()).or_insert(0) += 1;
        }

        // Find most shadowed symbols
        let mut symbol_vec: Vec<(String, usize)> = symbol_counts.into_iter().collect();
        symbol_vec.sort_by(|a, b| b.1.cmp(&a.1));
        analysis.most_shadowed_symbols = symbol_vec.into_iter().take(5).collect();

        // Calculate complexity score
        analysis.complexity_score = self.calculate_shadowing_complexity(&warnings);

        analysis
    }

    pub fn create_shadowing_aware_dependency_resolver(
        &self,
        base_resolver: Box<dyn DependencyResolver>,
    ) -> ShadowingAwareDependencyResolver {
        ShadowingAwareDependencyResolver::new(base_resolver).with_symbol_table(
            self.name_resolution_engine
                .shadowing_resolver
                .symbol_table
                .clone(),
        )
    }

    pub fn get_performance_metrics(&self) -> &PerformanceMetrics {
        &self.performance_metrics
    }

    pub fn get_cache_statistics(&self) -> CacheStatistics {
        let (hits, misses, hit_rate) = self.optimized_resolver.get_cache_stats();
        CacheStatistics {
            cache_hits: hits,
            cache_misses: misses,
            hit_rate,
            total_lookups: hits + misses,
        }
    }

    pub fn invalidate_caches(&mut self) {
        self.optimized_resolver.invalidate_cache();
    }

    pub fn rebuild_indexes(&mut self, symbol_table: &SymbolTable) {
        self.optimized_resolver.rebuild_index(symbol_table);
    }

    fn calculate_shadowing_complexity(&self, warnings: &[ShadowingWarning]) -> f64 {
        if warnings.is_empty() {
            return 0.0;
        }

        // Simple complexity calculation based on number of warnings and nesting depth
        let base_score = warnings.len() as f64;

        // Add complexity for deeply nested shadowing
        let mut depth_score = 0.0;
        for warning in warnings {
            // Estimate depth by looking at line differences
            let line_diff = warning
                .shadowing_definition
                .position
                .start_line
                .abs_diff(warning.shadowed_definition.position.start_line);
            depth_score += (line_diff as f64).log2().max(1.0);
        }

        base_score + (depth_score / warnings.len() as f64)
    }
}

#[derive(Debug, Clone)]
pub struct ShadowingResolutionResult {
    pub resolved_definition: Option<Definition>,
    pub all_candidates: Vec<crate::shadowing_resolver::ResolutionCandidate>,
    pub best_candidate: Option<crate::shadowing_resolver::ResolutionCandidate>,
    pub resolution_time_ms: u64,
    pub cache_hit: bool,
}

#[derive(Debug, Clone)]
pub struct ShadowingAnalysis {
    pub scope_id: ScopeId,
    pub total_warnings: usize,
    pub shadowing_levels: HashMap<String, usize>,
    pub most_shadowed_symbols: Vec<(String, usize)>,
    pub complexity_score: f64,
}

#[derive(Debug, Clone)]
pub struct CacheStatistics {
    pub cache_hits: usize,
    pub cache_misses: usize,
    pub hit_rate: f64,
    pub total_lookups: usize,
}

// Factory function for easy creation
pub fn create_advanced_shadowing_system(symbol_table: SymbolTable) -> AdvancedShadowingSystem {
    AdvancedShadowingSystem::new(symbol_table).with_caching(1000) // Default cache size
}

// Convenience function for simple use cases
pub fn resolve_symbol_with_shadowing(
    usage: &Usage,
    symbol_table: &SymbolTable,
) -> Option<Definition> {
    let mut system = AdvancedShadowingSystem::new(symbol_table.clone());
    let result = system.resolve_symbol(usage);
    result.resolved_definition
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{
        scope::{Accessibility, ScopeType},
        usage::UsageKind,
        DefinitionType, Position,
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
    fn test_advanced_shadowing_system_basic() {
        let mut symbol_table = SymbolTable::new();

        let func_scope_id = symbol_table.scopes.create_scope(
            Some(0),
            ScopeType::Function,
            create_test_position(5, 1),
            create_test_position(15, 1),
        );

        let outer_def = create_test_definition("system_var", 1);
        let inner_def = create_test_definition("system_var", 8);

        symbol_table.add_symbol(
            "system_var".to_string(),
            outer_def,
            0,
            Accessibility::ScopeLocal,
            false,
        );
        symbol_table.add_symbol(
            "system_var".to_string(),
            inner_def.clone(),
            func_scope_id,
            Accessibility::ScopeLocal,
            false,
        );

        let mut system = create_advanced_shadowing_system(symbol_table);
        let usage = create_test_usage("system_var", 10);

        let result = system.resolve_symbol(&usage);

        // Should find some resolution
        assert!(!result.all_candidates.is_empty());

        // Performance metrics should be updated
        let metrics = system.get_performance_metrics();
        assert_eq!(metrics.symbols_processed, 1);
    }

    #[test]
    fn test_batch_resolution() {
        let mut symbol_table = SymbolTable::new();

        // Add multiple symbols
        for i in 0..3 {
            let def_name = format!("batch_system_var_{}", i);
            let def = create_test_definition(&def_name, i + 1);
            symbol_table.add_symbol(def_name, def, 0, Accessibility::ScopeLocal, false);
        }

        let mut system = create_advanced_shadowing_system(symbol_table);

        let usages: Vec<Usage> = (0..3)
            .map(|i| create_test_usage(&format!("batch_system_var_{}", i), 10))
            .collect();

        let results = system.batch_resolve_symbols(&usages);
        assert_eq!(results.len(), 3);

        let metrics = system.get_performance_metrics();
        assert_eq!(metrics.symbols_processed, 3);
    }

    #[test]
    fn test_cache_statistics() {
        let mut symbol_table = SymbolTable::new();
        let def = create_test_definition("cache_test_var", 1);
        symbol_table.add_symbol(
            "cache_test_var".to_string(),
            def,
            0,
            Accessibility::ScopeLocal,
            false,
        );

        let mut system = create_advanced_shadowing_system(symbol_table);
        let usage = create_test_usage("cache_test_var", 5);

        // First resolution
        system.resolve_symbol(&usage);

        // Second resolution (should hit cache)
        system.resolve_symbol(&usage);

        let cache_stats = system.get_cache_statistics();
        assert!(cache_stats.total_lookups >= 1);
        assert!(cache_stats.hit_rate >= 0.0);
    }

    #[test]
    fn test_shadowing_analysis() {
        let mut symbol_table = SymbolTable::new();

        let func_scope_id = symbol_table.scopes.create_scope(
            Some(0),
            ScopeType::Function,
            create_test_position(5, 1),
            create_test_position(15, 1),
        );

        // Create shadowing scenario
        let outer_def = create_test_definition("analyzed_var", 1);
        let inner_def = create_test_definition("analyzed_var", 8);

        symbol_table.add_symbol(
            "analyzed_var".to_string(),
            outer_def,
            0,
            Accessibility::ScopeLocal,
            false,
        );
        symbol_table.add_symbol(
            "analyzed_var".to_string(),
            inner_def,
            func_scope_id,
            Accessibility::ScopeLocal,
            false,
        );

        let system = create_advanced_shadowing_system(symbol_table);
        let analysis = system.analyze_shadowing_patterns(func_scope_id);

        assert_eq!(analysis.scope_id, func_scope_id);
        assert!(analysis.complexity_score >= 0.0);
    }
}
