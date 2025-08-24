use crate::dependency_resolver::DependencyResolverTrait;
use crate::models::{scope::SymbolTable, Definition, Dependency, Usage};
use tree_sitter::Node;

use super::method_resolver::MethodResolver;
use super::module_resolver::ModuleResolver;

/// TypeScript-specific dependency resolver
pub struct TypeScriptDependencyResolver {
    #[allow(dead_code)]
    symbol_table: SymbolTable,
    method_resolver: MethodResolver,
    module_resolver: ModuleResolver,
}

impl TypeScriptDependencyResolver {
    pub fn new(symbol_table: SymbolTable) -> Self {
        Self {
            symbol_table,
            method_resolver: MethodResolver::new(),
            module_resolver: ModuleResolver::new(),
        }
    }

    /// TypeScript-specific field access resolution
    fn resolve_typescript_field_access(
        &self,
        usage_node: &Usage,
        definitions: &[Definition],
    ) -> Vec<Dependency> {
        self.method_resolver
            .resolve_struct_field_access(usage_node, definitions)
    }

    /// Check if definition is accessible from usage (TypeScript-specific rules)
    fn is_accessible_basic(&self, usage: &Usage, definition: &Definition) -> bool {
        // Check for hoisting rules first
        if self.is_hoisted_basic(definition) {
            return true;
        }

        // For non-hoisted definitions, check TypeScript-specific scope rules
        if !self.is_hoisted_basic(definition)
            && !self
                .module_resolver
                .are_in_same_function_scope(usage, definition)
        {
            return false;
        }

        true
    }

    fn is_hoisted_basic(&self, definition: &Definition) -> bool {
        use crate::models::DefinitionType;
        match definition.definition_type {
            // In TypeScript, function declarations and classes are hoisted
            DefinitionType::FunctionDefinition => true,
            DefinitionType::TypeDefinition => true,
            DefinitionType::InterfaceDefinition => true,
            DefinitionType::ClassDefinition => true,
            DefinitionType::EnumDefinition => true,
            DefinitionType::ModuleDefinition => true,
            _ => false,
        }
    }

    fn select_preferred_definition_typescript_aware<'a>(
        &self,
        usage_node: &Usage,
        matching_definitions: &[&'a Definition],
    ) -> Option<&'a Definition> {
        self.module_resolver
            .select_preferred_definition(usage_node, matching_definitions)
    }

    #[allow(dead_code)]
    fn find_closest_accessible_definition_basic<'a>(
        &self,
        usage: &Usage,
        definitions: &'a [Definition],
    ) -> Option<&'a Definition> {
        let matching_definitions: Vec<&Definition> = definitions
            .iter()
            .filter(|d| d.name == usage.name && self.is_accessible_basic(usage, d))
            .collect();

        if matching_definitions.is_empty() {
            return None;
        }

        let usage_line = usage.position.start_line;

        let mut best_def: &Definition = matching_definitions[0];
        let mut best_distance = if best_def.position.start_line <= usage_line {
            usage_line - best_def.position.start_line
        } else {
            usize::MAX
        };

        for &def in &matching_definitions[1..] {
            let distance = if def.position.start_line <= usage_line {
                usage_line - def.position.start_line
            } else {
                usize::MAX
            };

            if distance < best_distance
                || (distance == best_distance
                    && def.position.start_line > best_def.position.start_line)
            {
                best_def = def;
                best_distance = distance;
            }
        }

        Some(best_def)
    }
}

impl DependencyResolverTrait for TypeScriptDependencyResolver {
    fn resolve_dependencies(
        &self,
        source_code: &str,
        root_node: Node,
        usage_nodes: &[Usage],
        definitions: &[Definition],
    ) -> Result<Vec<Dependency>, String> {
        let mut all_dependencies = Vec::new();

        for usage_node in usage_nodes {
            let mut deps =
                self.resolve_single_dependency(source_code, root_node, usage_node, definitions);
            all_dependencies.append(&mut deps);
        }

        Ok(all_dependencies)
    }

    fn resolve_single_dependency(
        &self,
        _source_code: &str,
        _root_node: Node,
        usage_node: &Usage,
        definitions: &[Definition],
    ) -> Vec<Dependency> {
        let mut dependencies = Vec::new();

        // Try TypeScript-specific field access resolution
        if usage_node.kind == crate::models::UsageKind::FieldExpression {
            let field_dependencies = self.resolve_typescript_field_access(usage_node, definitions);
            if !field_dependencies.is_empty() {
                dependencies.extend(field_dependencies);
                return dependencies;
            }
        }

        // Find matching definitions with TypeScript-specific filtering
        let all_matching_definitions: Vec<&Definition> = definitions
            .iter()
            .filter(|def| def.name == usage_node.name)
            .collect();

        let matching_definitions: Vec<&Definition> = all_matching_definitions
            .into_iter()
            .filter(|def| self.is_accessible_basic(usage_node, def))
            .filter(|def| self.module_resolver.is_valid_dependency(usage_node, def))
            .collect();

        // Apply TypeScript-specific preference logic
        let preferred_definition =
            self.select_preferred_definition_typescript_aware(usage_node, &matching_definitions);

        if let Some(definition) = preferred_definition {
            let source_line = usage_node.position.start_line;
            let target_line = definition.position.start_line;

            if source_line != target_line {
                let dependency = Dependency {
                    source_line,
                    target_line,
                    symbol: usage_node.name.clone(),
                    dependency_type: self.get_dependency_type(usage_node),
                    context: self.get_context(usage_node),
                };
                dependencies.push(dependency);
            }
        }

        dependencies
    }
}
