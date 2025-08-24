use super::rust::rust_definition_collector::RustDefinitionCollector;
use super::rust::rust_usage_node_collector::RustUsageNodeCollector;
use super::typescript::typescript_definition_collector::TypescriptDefinitionCollector;
use super::typescript::typescript_usage_node_collector::TypescriptUsageNodeCollector;
use crate::definition_collectors::DefinitionCollector;
use crate::dependency_resolver::DependencyResolverTrait;
use crate::models::Language;
use crate::scope_collector::ScopeCollector as ScopeCollectorTrait;
use crate::usage_collector::UsageCollector;

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

pub fn create_scope_collector(language: Language) -> Result<Box<dyn ScopeCollectorTrait>, String> {
    match language {
        Language::Rust => Ok(Box::new(
            super::rust::rust_scope_collector::RustScopeCollector::new(),
        )),
        Language::TypeScript | Language::TSX => Ok(Box::new(
            super::typescript::typescript_scope_collector::TypeScriptScopeCollector::new(),
        )),
    }
}

pub fn get_dependency_resolver(
    language: Language,
    symbol_table: crate::models::SymbolTable,
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
