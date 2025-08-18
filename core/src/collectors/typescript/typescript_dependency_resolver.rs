use crate::collectors::common::dependency_resolver::{
    find_definition_in_scope, DependencyResolver,
};
use crate::models::{Definition, Dependency, DependencyType, Usage, UsageKind};

pub struct TypescriptDependencyResolver;

impl TypescriptDependencyResolver {
    pub fn new() -> Self {
        Self
    }
}

impl Default for TypescriptDependencyResolver {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> DependencyResolver<'a> for TypescriptDependencyResolver {
    fn resolve_dependencies(
        &self,
        source_code: &'a str,
        usage_nodes: &[Usage<'a>],
        definitions: &[Definition],
    ) -> Result<Vec<Dependency>, String> {
        let mut all_dependencies = Vec::new();

        for usage_node in usage_nodes {
            let mut deps = self.resolve_single_dependency(source_code, usage_node, definitions);
            all_dependencies.append(&mut deps);
        }

        Ok(all_dependencies)
    }

    fn resolve_single_dependency(
        &self,
        source_code: &'a str,
        usage_node: &Usage<'a>,
        definitions: &[Definition],
    ) -> Vec<Dependency> {
        let mut dependencies = Vec::new();

        match usage_node.kind {
            UsageKind::Identifier => {
                self.resolve_identifier_dependency(
                    source_code,
                    usage_node,
                    definitions,
                    &mut dependencies,
                );
            }
            UsageKind::CallExpression => {
                self.resolve_call_expression_dependency(
                    source_code,
                    usage_node,
                    definitions,
                    &mut dependencies,
                );
            }
            UsageKind::FieldExpression => {
                self.resolve_field_expression_dependency(
                    source_code,
                    usage_node,
                    definitions,
                    &mut dependencies,
                );
            }
            _ => {} // TypeScript doesn't have struct expressions or metavariables
        }

        dependencies
    }
}

impl TypescriptDependencyResolver {
    fn resolve_identifier_dependency(
        &self,
        source_code: &str,
        usage_node: &Usage,
        definitions: &[Definition],
        dependencies: &mut Vec<Dependency>,
    ) {
        let node = usage_node.node;
        let parent_kind = node.parent().map(|p| p.kind());

        let is_declaration_name = parent_kind == Some("function_declaration")
            && (node.parent().unwrap().child_by_field_name("name") == Some(node))
            || parent_kind == Some("class_declaration")
                && (node.parent().unwrap().child_by_field_name("name") == Some(node))
            || parent_kind == Some("interface_declaration")
                && (node.parent().unwrap().child_by_field_name("name") == Some(node));

        if parent_kind != Some("variable_declarator") && !is_declaration_name {
            self.add_dependency_if_needed(dependencies, source_code, usage_node, definitions);
        }
    }

    fn resolve_call_expression_dependency(
        &self,
        source_code: &str,
        usage_node: &Usage,
        definitions: &[Definition],
        dependencies: &mut Vec<Dependency>,
    ) {
        let node = usage_node.node;

        if let Some(function_node) = node.child_by_field_name("function") {
            if function_node.kind() == "identifier" {
                let func_usage_node = Usage {
                    node: function_node,
                    kind: UsageKind::Identifier,
                    scope: usage_node.scope.clone(),
                };
                self.add_dependency_if_needed(
                    dependencies,
                    source_code,
                    &func_usage_node,
                    definitions,
                );
            }
        }

        if let Some(arguments_node) = node.child_by_field_name("arguments") {
            let mut arg_cursor = arguments_node.walk();
            for arg_child in arguments_node.children(&mut arg_cursor) {
                if arg_child.kind() == "identifier" {
                    let source_line = arg_child.start_position().row + 1;
                    let symbol = arg_child
                        .utf8_text(source_code.as_bytes())
                        .unwrap()
                        .trim()
                        .to_string();

                    if let Some(def) =
                        find_definition_in_scope(definitions, &symbol, &usage_node.scope)
                    {
                        let target_line = def.line_number;
                        if source_line != target_line {
                            dependencies.push(Dependency {
                                source_line,
                                target_line,
                                symbol,
                                dependency_type: DependencyType::VariableUse,
                                context: Some("variable_use".to_string()),
                            });
                        }
                    }
                }
            }
        }
    }

    fn resolve_field_expression_dependency(
        &self,
        source_code: &str,
        usage_node: &Usage,
        definitions: &[Definition],
        dependencies: &mut Vec<Dependency>,
    ) {
        let node = usage_node.node;

        if let Some(parent) = node.parent() {
            if parent.kind() == "member_expression" {
                if let Some(object_node) = parent.child_by_field_name("object") {
                    if object_node.kind() == "identifier" {
                        let source_line = object_node.start_position().row + 1;
                        let symbol = object_node
                            .utf8_text(source_code.as_bytes())
                            .unwrap()
                            .trim()
                            .to_string();

                        if let Some(def) =
                            find_definition_in_scope(definitions, &symbol, &usage_node.scope)
                        {
                            let target_line = def.line_number;
                            if source_line != target_line {
                                dependencies.push(Dependency {
                                    source_line,
                                    target_line,
                                    symbol,
                                    dependency_type: DependencyType::StructFieldAccess,
                                    context: Some("member_access".to_string()),
                                });
                            }
                        }
                    }
                }
            }
        }
    }
}
