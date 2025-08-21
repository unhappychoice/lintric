use super::rust::rust_definition_collector::RustDefinitionCollector;
use super::rust::rust_dependency_resolver::RustDependencyResolver;
use super::rust::rust_usage_node_collector::RustUsageNodeCollector;
use super::typescript::typescript_definition_collector::TypescriptDefinitionCollector;
use super::typescript::typescript_dependency_resolver::TypescriptDependencyResolver;
use super::typescript::typescript_usage_node_collector::TypescriptUsageNodeCollector;
use crate::definition_collectors::DefinitionCollector;
use crate::dependency_resolver::DependencyResolver;
use crate::models::Language;
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

pub fn get_dependency_resolver(language: Language) -> Result<Box<dyn DependencyResolver>, String> {
    match language {
        Language::Rust => Ok(Box::new(RustDependencyResolver::new())),
        Language::TypeScript | Language::TSX => Ok(Box::new(TypescriptDependencyResolver::new())),
    }
}
