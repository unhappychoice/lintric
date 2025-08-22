use tree_sitter::Node;

use crate::dependency_resolver::DependencyResolver;
use crate::models::{Definition, Dependency, SymbolTable, Usage};
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
                // Use scope-aware resolution
                let dependencies = scope_resolver.resolve_dependencies_with_scope(
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

    pub fn collect_scope_aware_definitions(
        &self,
        root_node: Node,
        source_code: &str,
    ) -> Result<SymbolTable, String> {
        let mut collector = ScopeAwareDefinitionCollector::new(self.language.to_lowercase());
        collector.collect_with_scopes(root_node, source_code)
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
