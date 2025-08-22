use crate::dependency_resolver::DependencyResolver;
use crate::models::{Definition, Dependency, Usage, UsageKind};
use tree_sitter::Node;

pub struct RustDependencyResolver;

impl RustDependencyResolver {
    pub fn new() -> Self {
        Self
    }
}

impl Default for RustDependencyResolver {
    fn default() -> Self {
        Self::new()
    }
}

impl DependencyResolver for RustDependencyResolver {
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
        source_code: &str,
        root_node: Node,
        usage_node: &Usage,
        definitions: &[Definition],
    ) -> Vec<Dependency> {
        let mut dependencies = Vec::new();

        match usage_node.kind {
            UsageKind::FieldExpression => {
                // Handle field access like p.x
                dependencies.extend(self.resolve_field_access_dependency(
                    source_code,
                    root_node,
                    usage_node,
                    definitions,
                ));
            }
            UsageKind::CallExpression => {
                // Handle call expressions like add(1, 2)
                dependencies.extend(self.resolve_call_expression_dependency(
                    source_code,
                    root_node,
                    usage_node,
                    definitions,
                ));
            }
            _ => {
                // Find the most appropriate definition (closest accessible one)
                if let Some(def) = self.find_closest_accessible_definition(usage_node, definitions)
                {
                    let source_line = usage_node.position.line_number();
                    let target_line = def.line_number();

                    // Don't create self-referential dependencies
                    if source_line != target_line {
                        dependencies.push(Dependency {
                            source_line,
                            target_line,
                            symbol: usage_node.name.clone(),
                            dependency_type: self.get_dependency_type(usage_node),
                            context: self.get_context(usage_node),
                        });
                    }
                }
            }
        }

        dependencies
    }
}

impl RustDependencyResolver {
    fn is_accessible(&self, usage: &Usage, definition: &Definition) -> bool {
        // Check for hoisting rules first
        if self.is_hoisted(definition) {
            return true; // Hoisted definitions are always accessible
        }

        // Basic position check: definition must come before usage
        if definition.position.start_line < usage.position.start_line {
            return true;
        }

        if definition.position.start_line == usage.position.start_line {
            return definition.position.start_column < usage.position.start_column;
        }

        // Definition comes after usage - not accessible for basic forward reference check
        false
    }

    fn is_hoisted(&self, definition: &Definition) -> bool {
        use crate::models::DefinitionType;
        match definition.definition_type {
            // In Rust, function definitions are accessible from anywhere in the same scope
            DefinitionType::FunctionDefinition => true,
            // Other items that are hoisted in Rust
            DefinitionType::StructDefinition => true,
            DefinitionType::EnumDefinition => true,
            DefinitionType::TypeDefinition => true,
            DefinitionType::ModuleDefinition => true,
            DefinitionType::MacroDefinition => true,
            _ => false,
        }
    }

    fn find_closest_accessible_definition<'a>(
        &self,
        usage: &Usage,
        definitions: &'a [Definition],
    ) -> Option<&'a Definition> {
        let matching_definitions: Vec<&Definition> = definitions
            .iter()
            .filter(|d| d.name == usage.name && self.is_accessible(usage, d))
            .collect();

        if matching_definitions.is_empty() {
            return None;
        }

        // For hoisted definitions, prefer the one declared in the same scope level
        // For non-hoisted definitions, prefer the closest one that comes before the usage
        let usage_line = usage.position.start_line;

        // Sort by preference: closest before usage line, then by line number
        let mut best_def: &Definition = matching_definitions[0];
        let mut best_distance = if best_def.position.start_line <= usage_line {
            usage_line - best_def.position.start_line
        } else {
            usize::MAX // Hoisted definitions that come after usage
        };

        for &def in &matching_definitions[1..] {
            let distance = if def.position.start_line <= usage_line {
                usage_line - def.position.start_line
            } else {
                usize::MAX // Hoisted definitions that come after usage
            };

            // Prefer smaller distance (closer definitions)
            // For same distance, prefer the one that comes later (more recent in scope)
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

    fn find_closest_accessible_from_list<'a>(
        &self,
        usage: &Usage,
        definitions: &[&'a Definition],
    ) -> Option<&'a Definition> {
        let matching_definitions: Vec<&Definition> = definitions
            .iter()
            .filter(|&&d| self.is_accessible(usage, d))
            .cloned()
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

    fn resolve_call_expression_dependency(
        &self,
        _source_code: &str,
        _root_node: Node,
        usage_node: &Usage,
        definitions: &[Definition],
    ) -> Vec<Dependency> {
        let mut dependencies = Vec::new();

        // The usage_node.name should now contain only the function name (extracted during usage collection)
        let function_name = &usage_node.name;

        // Look for function definition or import with this name
        let function_definitions: Vec<&Definition> = definitions
            .iter()
            .filter(|d| {
                d.name == *function_name
                    && matches!(
                        d.definition_type,
                        crate::models::DefinitionType::FunctionDefinition
                            | crate::models::DefinitionType::ImportDefinition
                    )
            })
            .collect();

        if let Some(function_def) =
            self.find_closest_accessible_from_list(usage_node, &function_definitions)
        {
            let source_line = usage_node.position.line_number();
            let target_line = function_def.line_number();

            // Don't create self-referential dependencies
            if source_line != target_line {
                dependencies.push(Dependency {
                    source_line,
                    target_line,
                    symbol: function_name.to_string(),
                    dependency_type: self.get_dependency_type(usage_node),
                    context: self.get_context(usage_node),
                });
            }
        }

        dependencies
    }

    fn resolve_field_access_dependency(
        &self,
        _source_code: &str,
        _root_node: Node,
        usage_node: &Usage,
        definitions: &[Definition],
    ) -> Vec<Dependency> {
        let mut dependencies = Vec::new();

        // The usage_node name is "p.x", extract field name from it
        if let Some(dot_pos) = usage_node.name.rfind('.') {
            let field_name = &usage_node.name[dot_pos + 1..];

            // Look for struct field definition with this name
            let field_definitions: Vec<&Definition> = definitions
                .iter()
                .filter(|d| {
                    d.name == field_name
                        && matches!(
                            d.definition_type,
                            crate::models::DefinitionType::StructFieldDefinition
                        )
                })
                .collect();

            if let Some(field_def) =
                self.find_closest_accessible_from_list(usage_node, &field_definitions)
            {
                dependencies.push(Dependency {
                    source_line: usage_node.position.line_number(),
                    target_line: field_def.line_number(),
                    symbol: field_name.to_string(),
                    dependency_type: crate::models::DependencyType::StructFieldAccess,
                    context: Some("field_access".to_string()),
                });
            }
        }

        dependencies
    }
}
