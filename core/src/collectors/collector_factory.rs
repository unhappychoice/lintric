use super::common::definition_collectors::DefinitionCollector;
use super::common::dependency_collectors::DependencyCollector;
use super::rust::rust_definition_collector::RustDefinitionCollector;
use super::rust::rust_dependency_collector::RustDependencyCollector;
use super::typescript::typescript_definition_collector::TypescriptDefinitionCollector;
use super::typescript::typescript_dependency_collector::TypescriptDependencyCollector;
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

pub fn get_dependency_collector<'a>(
    language: Language,
    source_code: &'a str,
) -> Result<Box<dyn DependencyCollector<'a> + 'a>, String> {
    match language {
        Language::Rust => Ok(Box::new(RustDependencyCollector::new(source_code))),
        Language::TypeScript => Ok(Box::new(TypescriptDependencyCollector::new(source_code))),
    }
}
