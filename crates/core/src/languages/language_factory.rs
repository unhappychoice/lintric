use super::rust::rust_node_extractors::{RustDefinitionExtractor, RustUsageExtractor};
use super::typescript::typescript_node_extractors::{
    TypeScriptDefinitionExtractor, TypeScriptUsageExtractor,
};
use crate::dependency_resolver::DependencyResolverTrait;
use crate::models::{ASTScopeTraverser, CodeAnalysisContext, Language};
use tree_sitter::Node;

/// New unified analysis using single AST traversal
pub fn analyze_code_unified<'a>(
    language: Language,
    source_code: &'a str,
    root_node: Node<'a>,
) -> Result<CodeAnalysisContext, String> {
    let mut traverser = ASTScopeTraverser::new();

    match language {
        Language::Rust => {
            let def_extractor = RustDefinitionExtractor;
            let usage_extractor = RustUsageExtractor;
            Ok(traverser.traverse(root_node, source_code, &def_extractor, &usage_extractor))
        }
        Language::TypeScript | Language::TSX => {
            let def_extractor = TypeScriptDefinitionExtractor;
            let usage_extractor = TypeScriptUsageExtractor;
            Ok(traverser.traverse(root_node, source_code, &def_extractor, &usage_extractor))
        }
    }
}

pub fn get_dependency_resolver(
    language: Language,
    context: CodeAnalysisContext,
) -> Result<Box<dyn DependencyResolverTrait>, String> {
    match language {
        Language::Rust => Ok(Box::new(
            super::rust::dependency_resolver::RustDependencyResolver::new_from_context(context),
        )),
        Language::TypeScript | Language::TSX => Ok(Box::new(
            super::typescript::dependency_resolver::TypeScriptDependencyResolver::new_from_context(
                context,
            ),
        )),
    }
}
