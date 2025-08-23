use super::rust::rust_definition_collector::RustDefinitionCollector;
use super::rust::rust_usage_node_collector::RustUsageNodeCollector;
use super::typescript::typescript_definition_collector::TypescriptDefinitionCollector;
use super::typescript::typescript_usage_node_collector::TypescriptUsageNodeCollector;
use crate::definition_collectors::DefinitionCollector;
use crate::dependency_resolver::DependencyResolverTrait;
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

// Simple fallback resolver for scope_integration compatibility
struct SimpleFallbackResolver;

impl DependencyResolverTrait for SimpleFallbackResolver {
    fn resolve_dependencies(
        &self,
        _source_code: &str,
        _root_node: tree_sitter::Node,
        usage_nodes: &[crate::models::Usage],
        definitions: &[crate::models::Definition],
    ) -> Result<Vec<crate::models::Dependency>, String> {
        let mut dependencies = Vec::new();

        for usage in usage_nodes {
            // Simple name-based matching for fallback
            if let Some(def) = definitions.iter().find(|d| d.name == usage.name) {
                let source_line = usage.position.line_number();
                let target_line = def.line_number();

                if source_line != target_line {
                    dependencies.push(crate::models::Dependency {
                        source_line,
                        target_line,
                        symbol: usage.name.clone(),
                        dependency_type: crate::models::DependencyType::VariableUse,
                        context: Some(format!(
                            "fallback:{}:{}",
                            usage.position.start_line, usage.position.start_column
                        )),
                    });
                }
            }
        }

        Ok(dependencies)
    }

    fn resolve_single_dependency(
        &self,
        _source_code: &str,
        _root_node: tree_sitter::Node,
        _usage_node: &crate::models::Usage,
        _definitions: &[crate::models::Definition],
    ) -> Vec<crate::models::Dependency> {
        Vec::new()
    }
}

pub fn get_dependency_resolver(
    _language: Language,
) -> Result<Box<dyn DependencyResolverTrait>, String> {
    Ok(Box::new(SimpleFallbackResolver))
}
