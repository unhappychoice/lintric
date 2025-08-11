use super::common::definition_collectors::DefinitionCollector;
use super::common::dependency_collectors::DependencyCollector;
use super::rust::rust_definition_collector::RustDefinitionCollector;
use super::rust::rust_dependency_collector::RustDependencyCollector;
use super::typescript::typescript_definition_collector::TypescriptDefinitionCollector;
use super::typescript::typescript_dependency_collector::TypescriptDependencyCollector;
use crate::models::Language;

pub fn get_definition_collector(
    language: Language,
) -> Result<Box<dyn DefinitionCollector>, String> {
    match language {
        Language::Rust => Ok(Box::new(RustDefinitionCollector)),
        Language::TypeScript => Ok(Box::new(TypescriptDefinitionCollector)),
    }
}

pub fn get_dependency_collector(
    language: Language,
) -> Result<Box<dyn DependencyCollector>, String> {
    match language {
        Language::Rust => Ok(Box::new(RustDependencyCollector)),
        Language::TypeScript => Ok(Box::new(TypescriptDependencyCollector)),
    }
}
