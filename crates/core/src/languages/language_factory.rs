use super::rust::rust_definition_collector::RustDefinitionCollector;
use super::rust::rust_node_extractors::{RustDefinitionExtractor, RustUsageExtractor};
use super::rust::rust_usage_node_collector::RustUsageNodeCollector;
use super::typescript::typescript_definition_collector::TypescriptDefinitionCollector;
use super::typescript::typescript_node_extractors::{
    TypeScriptDefinitionExtractor, TypeScriptUsageExtractor,
};
use super::typescript::typescript_usage_node_collector::TypescriptUsageNodeCollector;
use crate::definition_collectors::DefinitionCollector;
use crate::dependency_resolver::DependencyResolverTrait;
use crate::models::{ASTScopeTraverser, CodeAnalysisContext, Language, SymbolTable};
use crate::usage_collector::UsageCollector;
use tree_sitter::Node;

pub fn get_definition_collector<'a>(
    language: Language,
    source_code: &'a str,
) -> Result<Box<dyn DefinitionCollector<'a> + 'a>, String> {
    match language {
        Language::Rust => Ok(Box::new(RustDefinitionCollector::new(source_code))),
        Language::TypeScript | Language::TSX => {
            Ok(Box::new(TypescriptDefinitionCollector::new(source_code)))
        }
    }
}

pub fn get_usage_node_collector(
    language: Language,
    source_code: &str,
) -> Result<Box<dyn UsageCollector>, String> {
    match language {
        Language::Rust => Ok(Box::new(RustUsageNodeCollector::new(source_code))),
        Language::TypeScript | Language::TSX => {
            Ok(Box::new(TypescriptUsageNodeCollector::new(source_code)))
        }
    }
}

pub fn collect_definitions_with_scopes<'a>(
    language: Language,
    source_code: &'a str,
    root_node: Node<'a>,
) -> Result<SymbolTable, String> {
    let collector = get_definition_collector(language, source_code)?;
    collector.collect(source_code, root_node)
}

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
    symbol_table: SymbolTable,
) -> Result<Box<dyn DependencyResolverTrait>, String> {
    match language {
        Language::Rust => Ok(Box::new(
            super::rust::dependency_resolver::RustDependencyResolver::new(symbol_table),
        )),
        Language::TypeScript | Language::TSX => Ok(Box::new(
            super::typescript::dependency_resolver::TypeScriptDependencyResolver::new(symbol_table),
        )),
    }
}
