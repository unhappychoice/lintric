use tree_sitter::Node;

use super::{CodeAnalysisContext, Definition, Position, ScopeId, ScopeType, Usage};

/// Trait for extracting definition information from AST nodes
pub trait NodeDefinitionExtractor {
    /// Extract definitions from a node if it represents any
    fn extract_definition(&self, node: Node, scope: ScopeId, source: &str) -> Vec<Definition>;

    /// Check if this node should create a new scope
    fn creates_scope(&self, node: Node) -> Option<(ScopeType, Position)>;
}

/// Trait for extracting usage information from AST nodes
pub trait NodeUsageExtractor {
    /// Extract usage from a node if it represents one
    fn extract_usage(&self, node: Node, scope: ScopeId, source: &str) -> Option<Usage>;
}

/// Unified AST traverser with scope management
pub struct ASTScopeTraverser {
    current_scope: ScopeId,
}

impl ASTScopeTraverser {
    pub fn new() -> Self {
        Self {
            current_scope: 0, // Start with global scope
        }
    }

    /// Single traversal collecting definitions, usages, and scopes
    pub fn traverse<D, U>(
        &mut self,
        root: Node,
        source: &str,
        def_extractor: &D,
        usage_extractor: &U,
    ) -> CodeAnalysisContext
    where
        D: NodeDefinitionExtractor,
        U: NodeUsageExtractor,
    {
        let mut context = CodeAnalysisContext::new();
        self.current_scope = context.scopes.root;

        self.traverse_node(root, source, &mut context, def_extractor, usage_extractor);

        context
    }

    fn traverse_node<D, U>(
        &mut self,
        node: Node,
        source: &str,
        context: &mut CodeAnalysisContext,
        def_extractor: &D,
        usage_extractor: &U,
    ) where
        D: NodeDefinitionExtractor,
        U: NodeUsageExtractor,
    {
        // Check if this node creates a new scope FIRST
        let new_scope_id = if let Some((scope_type, position)) = def_extractor.creates_scope(node) {
            let scope_id =
                context
                    .scopes
                    .create_scope(Some(self.current_scope), scope_type, position);
            Some(scope_id)
        } else {
            None
        };

        // Update current scope if a new one was created
        let previous_scope = self.current_scope;
        if let Some(scope_id) = new_scope_id {
            self.current_scope = scope_id;
        }

        // Extract definitions from this node using the CURRENT scope (which may be the new scope)
        // This matches the old implementation where scope-creating items have their definitions in the new scope
        let definitions = def_extractor.extract_definition(node, self.current_scope, source);
        for mut definition in definitions {
            // Set scope context for the definition
            definition.set_context(
                self.current_scope,
                &super::definition::Accessibility::ScopeLocal, // Match old implementation
                false,                                         // Default, extractors can override
            );
            context
                .definitions
                .add_definition(definition.name.clone(), definition);
        }

        // Extract usage from this node
        if let Some(mut usage) = usage_extractor.extract_usage(node, self.current_scope, source) {
            // Set scope context for the usage
            usage.set_scope_id(Some(self.current_scope));
            context.usages.add_usage(usage);
        }

        // Recursively traverse children
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            self.traverse_node(child, source, context, def_extractor, usage_extractor);
        }

        // Restore previous scope
        self.current_scope = previous_scope;
    }
}

impl Default for ASTScopeTraverser {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Mock extractors for testing
    #[allow(dead_code)]
    struct MockDefinitionExtractor;
    #[allow(dead_code)]
    struct MockUsageExtractor;

    impl NodeDefinitionExtractor for MockDefinitionExtractor {
        fn extract_definition(
            &self,
            _node: Node,
            _scope: ScopeId,
            _source: &str,
        ) -> Vec<Definition> {
            vec![]
        }

        fn creates_scope(&self, _node: Node) -> Option<(ScopeType, Position)> {
            None
        }
    }

    impl NodeUsageExtractor for MockUsageExtractor {
        fn extract_usage(&self, _node: Node, _scope: ScopeId, _source: &str) -> Option<Usage> {
            None
        }
    }

    #[test]
    fn test_ast_traverser_creation() {
        let traverser = ASTScopeTraverser::new();
        assert_eq!(traverser.current_scope, 0);
    }

    #[test]
    fn test_context_creation() {
        let context = CodeAnalysisContext::new();
        assert_eq!(context.scopes.root, 0);
        assert!(context.definitions.get_all_definitions().is_empty());
        assert!(context.usages.get_all_usages().is_empty());
    }
}
