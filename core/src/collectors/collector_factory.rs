use super::common::definition_collectors::DefinitionCollector;
use super::common::dependency_resolver::DependencyResolver;
use super::common::usage_node_collector::UsageNodeCollector;
use super::rust::rust_definition_collector::RustDefinitionCollector;
use super::rust::rust_dependency_resolver::RustDependencyResolver;
use super::rust::rust_usage_node_collector::RustUsageNodeCollector;
use super::typescript::typescript_definition_collector::TypescriptDefinitionCollector;
use super::typescript::typescript_dependency_resolver::TypescriptDependencyResolver;
use super::typescript::typescript_usage_node_collector::TypescriptUsageNodeCollector;
use crate::models::Language;

pub fn get_definition_collector<'a>(
    language: Language,
    source_code: &'a str,
) -> Result<Box<dyn DefinitionCollector<'a> + 'a>, String> {
    match language {
        Language::Rust => Ok(Box::new(RustDefinitionCollector::new(source_code))),
        Language::TypeScript => Ok(Box::new(TypescriptDefinitionCollector::new(source_code))),
    }
}

pub fn get_usage_node_collector<'a>(
    language: Language,
    source_code: &'a str,
) -> Result<Box<dyn UsageNodeCollector<'a> + 'a>, String> {
    match language {
        Language::Rust => Ok(Box::new(RustUsageNodeCollector::new(source_code))),
        Language::TypeScript => Ok(Box::new(TypescriptUsageNodeCollector::new(source_code))),
    }
}

pub fn get_dependency_resolver<'a>(
    language: Language,
) -> Result<Box<dyn DependencyResolver<'a> + 'a>, String> {
    match language {
        Language::Rust => Ok(Box::new(RustDependencyResolver::new())),
        Language::TypeScript => Ok(Box::new(TypescriptDependencyResolver::new())),
    }
}
