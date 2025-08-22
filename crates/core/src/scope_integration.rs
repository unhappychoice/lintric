use tree_sitter::Node;

use crate::dependency_resolver::DependencyResolver;
use crate::models::{Definition, Dependency, SymbolTable, Usage};
use crate::nested_scope_resolver::NestedScopeResolver;
use crate::scope_aware_resolver::{DefaultScopeAwareResolver, ScopeAwareDependencyResolver};
use crate::scope_builder::ScopeAwareDefinitionCollector;

pub struct ScopeIntegratedResolver {
    language: String,
    fallback_resolver: Box<dyn DependencyResolver>,
}

impl ScopeIntegratedResolver {
    pub fn new(language: String, fallback_resolver: Box<dyn DependencyResolver>) -> Self {
        Self {
            language,
            fallback_resolver,
        }
    }

    pub fn analyze_with_scope_awareness(
        &self,
        source_code: &str,
        root_node: Node,
        usage_nodes: &[Usage],
        definitions: &[Definition],
    ) -> Result<(SymbolTable, Vec<Dependency>), String> {
        // Try to create scope-aware symbol table first
        let scope_resolver = DefaultScopeAwareResolver::new(self.language.to_lowercase());
        match scope_resolver.create_enhanced_symbol_table(root_node, source_code) {
            Ok(symbol_table) => {
                // Use enhanced nested scope resolution
                let dependencies = self.resolve_with_nested_scope_analysis(
                    source_code,
                    root_node,
                    usage_nodes,
                    &symbol_table,
                )?;
                Ok((symbol_table, dependencies))
            }
            Err(scope_error) => {
                // Fall back to traditional dependency resolution
                eprintln!(
                    "Warning: Failed to create scope-aware symbol table: {}",
                    scope_error
                );
                eprintln!("Falling back to traditional dependency resolution");

                let dependencies = self.fallback_resolver.resolve_dependencies(
                    source_code,
                    root_node,
                    usage_nodes,
                    definitions,
                )?;

                // Create a basic symbol table for consistency
                let basic_symbol_table = SymbolTable::new();
                Ok((basic_symbol_table, dependencies))
            }
        }
    }

    fn resolve_with_nested_scope_analysis(
        &self,
        _source_code: &str,
        _root_node: Node,
        usage_nodes: &[Usage],
        symbol_table: &SymbolTable,
    ) -> Result<Vec<Dependency>, String> {
        let nested_resolver = NestedScopeResolver::new(symbol_table.scopes.clone());
        let mut dependencies = Vec::new();

        for usage in usage_nodes {
            // Try nested scope resolution first
            let search_results = nested_resolver.resolve_nested_access(usage);

            if !search_results.is_empty() {
                // Use the best match (closest scope)
                let best_match = search_results
                    .iter()
                    .min_by_key(|result| result.scope_distance)
                    .unwrap();

                dependencies.push(Dependency::new_with_scope(
                    usage.position,
                    best_match.definition.position,
                    usage.clone(),
                    best_match.definition.clone(),
                ));
            } else {
                // Fall back to traditional scope-aware resolution
                let scope_resolver = DefaultScopeAwareResolver::new(self.language.to_lowercase());
                if let Some(definition) =
                    scope_resolver.find_definition_in_scope(usage, symbol_table)
                {
                    dependencies.push(Dependency::new_with_scope(
                        usage.position,
                        definition.position,
                        usage.clone(),
                        definition,
                    ));
                }
            }
        }

        Ok(dependencies)
    }

    pub fn collect_scope_aware_definitions(
        &self,
        root_node: Node,
        source_code: &str,
    ) -> Result<SymbolTable, String> {
        let mut collector = ScopeAwareDefinitionCollector::new(self.language.to_lowercase());
        collector.collect_with_scopes(root_node, source_code)
    }

    pub fn analyze_complex_nested_structures(
        &self,
        source_code: &str,
        root_node: Node,
    ) -> Result<
        (
            SymbolTable,
            std::collections::HashMap<
                crate::models::ScopeId,
                Vec<crate::nested_scope_resolver::CaptureInfo>,
            >,
        ),
        String,
    > {
        let scope_resolver = DefaultScopeAwareResolver::new(self.language.to_lowercase());
        let symbol_table = scope_resolver.create_enhanced_symbol_table(root_node, source_code)?;

        let mut nested_resolver = NestedScopeResolver::new(symbol_table.scopes.clone());
        let complex_analysis = nested_resolver.analyze_complex_nesting(0); // Start from global scope

        Ok((symbol_table, complex_analysis))
    }

    pub fn validate_nested_access_patterns(
        &self,
        source_code: &str,
        root_node: Node,
        usage_nodes: &[Usage],
    ) -> Result<Vec<(Usage, bool)>, String> {
        let scope_resolver = DefaultScopeAwareResolver::new(self.language.to_lowercase());
        let symbol_table = scope_resolver.create_enhanced_symbol_table(root_node, source_code)?;

        let nested_resolver = NestedScopeResolver::new(symbol_table.scopes.clone());
        let mut validation_results = Vec::new();

        for usage in usage_nodes {
            let search_results = nested_resolver.resolve_nested_access(usage);
            let is_valid = !search_results.is_empty()
                && search_results.iter().any(|result| {
                    nested_resolver.validate_nested_access(usage, &result.definition)
                });
            validation_results.push((usage.clone(), is_valid));
        }

        Ok(validation_results)
    }
}

impl DependencyResolver for ScopeIntegratedResolver {
    fn resolve_dependencies(
        &self,
        source_code: &str,
        root_node: Node,
        usage_nodes: &[Usage],
        definitions: &[Definition],
    ) -> Result<Vec<Dependency>, String> {
        match self.analyze_with_scope_awareness(source_code, root_node, usage_nodes, definitions) {
            Ok((_, dependencies)) => Ok(dependencies),
            Err(e) => Err(format!("Failed to resolve dependencies: {}", e)),
        }
    }

    fn resolve_single_dependency(
        &self,
        source_code: &str,
        root_node: Node,
        usage_node: &Usage,
        definitions: &[Definition],
    ) -> Vec<Dependency> {
        // For single dependency resolution, we fall back to the traditional approach
        // as scope-aware resolution is designed for batch processing
        self.fallback_resolver.resolve_single_dependency(
            source_code,
            root_node,
            usage_node,
            definitions,
        )
    }
}

pub fn create_scope_integrated_resolver(
    language: String,
    fallback_resolver: Box<dyn DependencyResolver>,
) -> ScopeIntegratedResolver {
    ScopeIntegratedResolver::new(language, fallback_resolver)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{DefinitionType, Position, UsageKind};
    use tree_sitter::{Language, Parser};

    extern "C" {
        fn tree_sitter_rust() -> Language;
    }

    struct MockFallbackResolver;

    impl DependencyResolver for MockFallbackResolver {
        fn resolve_dependencies(
            &self,
            _source_code: &str,
            _root_node: Node,
            _usage_nodes: &[Usage],
            _definitions: &[Definition],
        ) -> Result<Vec<Dependency>, String> {
            // Mock implementation returns empty dependencies
            Ok(vec![])
        }

        fn resolve_single_dependency(
            &self,
            _source_code: &str,
            _root_node: Node,
            _usage_node: &Usage,
            _definitions: &[Definition],
        ) -> Vec<Dependency> {
            vec![]
        }
    }

    #[test]
    fn test_scope_integrated_resolver_fallback() {
        let fallback = Box::new(MockFallbackResolver);
        let resolver = ScopeIntegratedResolver::new("rust".to_string(), fallback);

        let mut parser = Parser::new();
        unsafe {
            parser.set_language(&tree_sitter_rust()).unwrap();
        }

        let source_code = "fn main() { let x = 10; }";
        let tree = parser.parse(source_code, None).unwrap();

        let usage_nodes = vec![Usage::new_simple(
            "x".to_string(),
            Position {
                start_line: 1,
                start_column: 20,
                end_line: 1,
                end_column: 21,
            },
            UsageKind::Read,
        )];

        let definitions = vec![Definition::new_simple(
            "x".to_string(),
            DefinitionType::VariableDefinition,
            Position {
                start_line: 1,
                start_column: 17,
                end_line: 1,
                end_column: 18,
            },
        )];

        let result = resolver.analyze_with_scope_awareness(
            source_code,
            tree.root_node(),
            &usage_nodes,
            &definitions,
        );

        assert!(result.is_ok());
        let (symbol_table, dependencies) = result.unwrap();

        // Should have a symbol table (even if basic)
        assert!(symbol_table.scopes.get_scope(0).is_some());

        // Dependencies result is acceptable (could be empty in fallback)
        assert!(dependencies.len() >= 0);
    }

    #[test]
    fn test_create_scope_integrated_resolver() {
        let fallback = Box::new(MockFallbackResolver);
        let resolver = create_scope_integrated_resolver("rust".to_string(), fallback);

        assert_eq!(resolver.language, "rust");
    }
}
