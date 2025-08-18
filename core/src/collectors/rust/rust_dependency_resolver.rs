use crate::collectors::common::dependency_resolver::{
    find_definition_in_scope, DependencyResolver,
};
use crate::models::{Definition, Dependency, DependencyType, Usage, UsageKind};
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

impl<'a> DependencyResolver<'a> for RustDependencyResolver {
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
            UsageKind::StructExpression => {
                self.resolve_struct_expression_dependency(
                    source_code,
                    usage_node,
                    definitions,
                    &mut dependencies,
                );
            }
            UsageKind::Metavariable => {
                self.resolve_metavariable_dependency(
                    source_code,
                    usage_node,
                    definitions,
                    &mut dependencies,
                );
            }
        }

        dependencies
    }
}

impl RustDependencyResolver {
    fn resolve_identifier_dependency(
        &self,
        source_code: &str,
        usage_node: &Usage,
        definitions: &[Definition],
        dependencies: &mut Vec<Dependency>,
    ) {
        let node = usage_node.node;

        let is_definition = definitions
            .iter()
            .filter(|d| d.name == node.utf8_text(source_code.as_bytes()).unwrap())
            .filter(|d| d.line_number == node.start_position().row + 1)
            .count()
            > 0;

        if is_definition {
            return;
        }

        self.add_dependency_if_needed(dependencies, source_code, usage_node, definitions);
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
                let source_line = function_node.start_position().row + 1;
                let symbol = function_node
                    .utf8_text(source_code.as_bytes())
                    .unwrap()
                    .trim()
                    .to_string();

                if let Some(def) = find_definition_in_scope(definitions, &symbol, &usage_node.scope)
                {
                    let target_line = def.line_number;
                    if source_line != target_line {
                        dependencies.push(Dependency {
                            source_line,
                            target_line,
                            symbol,
                            dependency_type: DependencyType::FunctionCall,
                            context: Some("call_expression".to_string()),
                        });
                    }
                }
            } else if function_node.kind() == "scoped_identifier" {
                self.resolve_scoped_identifier_call(
                    function_node,
                    source_code,
                    definitions,
                    dependencies,
                );
            } else if function_node.kind() == "field_expression" {
                self.resolve_method_call(function_node, source_code, definitions, dependencies);
            } else if function_node.kind() == "scoped_type_identifier" {
                self.resolve_ufcs_call(function_node, source_code, definitions, dependencies);
            }
        }

        if let Some(arguments_node) = node.child_by_field_name("arguments") {
            self.resolve_arguments(
                arguments_node,
                source_code,
                definitions,
                dependencies,
                &usage_node.scope,
            );
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
            if parent.kind() == "call_expression"
                && parent.child_by_field_name("function") == Some(node)
            {
                return;
            }
        }

        if let Some(value_node) = node.child_by_field_name("value") {
            if value_node.kind() == "identifier" {
                let source_line = value_node.start_position().row + 1;
                let symbol = value_node
                    .utf8_text(source_code.as_bytes())
                    .unwrap()
                    .trim()
                    .to_string();

                if let Some(def) = find_definition_in_scope(definitions, &symbol, &usage_node.scope)
                {
                    let target_line = def.line_number;
                    if source_line != target_line {
                        dependencies.push(Dependency {
                            source_line,
                            target_line,
                            symbol,
                            dependency_type: DependencyType::StructFieldAccess,
                            context: Some("field_access".to_string()),
                        });
                    }
                }
            }
        }
        if let Some(field_node) = node.child_by_field_name("field") {
            let source_line = field_node.start_position().row + 1;
            let symbol = field_node
                .utf8_text(source_code.as_bytes())
                .unwrap()
                .trim()
                .to_string();

            if let Some(def) = find_definition_in_scope(definitions, &symbol, &usage_node.scope) {
                let target_line = def.line_number;
                if source_line != target_line {
                    dependencies.push(Dependency {
                        source_line,
                        target_line,
                        symbol,
                        dependency_type: DependencyType::StructFieldAccess,
                        context: Some("field_access".to_string()),
                    });
                }
            }
        }
    }

    fn resolve_struct_expression_dependency(
        &self,
        source_code: &str,
        usage_node: &Usage,
        definitions: &[Definition],
        dependencies: &mut Vec<Dependency>,
    ) {
        let node = usage_node.node;

        if let Some(type_node) = node.child_by_field_name("type") {
            let type_usage_node = Usage {
                node: type_node,
                kind: UsageKind::Identifier,
                scope: usage_node.scope.clone(),
            };
            self.add_dependency_if_needed(dependencies, source_code, &type_usage_node, definitions);
        }
    }

    fn resolve_metavariable_dependency(
        &self,
        source_code: &str,
        usage_node: &Usage,
        definitions: &[Definition],
        dependencies: &mut Vec<Dependency>,
    ) {
        self.add_dependency_if_needed(dependencies, source_code, usage_node, definitions);
    }

    fn resolve_scoped_identifier_call(
        &self,
        function_node: Node,
        source_code: &str,
        definitions: &[Definition],
        dependencies: &mut Vec<Dependency>,
    ) {
        let path_node = function_node.child_by_field_name("path").unwrap();
        let path_text = path_node.utf8_text(source_code.as_bytes()).unwrap();

        let name_node = function_node.child_by_field_name("name").unwrap();
        let name_text = name_node.utf8_text(source_code.as_bytes()).unwrap();

        let f_definitions: Vec<&Definition> =
            definitions.iter().filter(|d| d.name == name_text).collect();

        if let Some(candidates) = ufcs_scope_candidates_from_path(&path_node, source_code) {
            let mut pushed = false;
            for cand in candidates {
                if pushed {
                    break;
                }
                if let Some(def) = f_definitions
                    .iter()
                    .find(|d| d.scope.as_deref().map(|s| s.starts_with(&cand)) == Some(true))
                {
                    let source_line = name_node.start_position().row + 1;
                    let target_line = def.line_number;
                    if source_line != target_line {
                        dependencies.push(Dependency {
                            source_line,
                            target_line,
                            symbol: name_text.to_string(),
                            dependency_type: DependencyType::FunctionCall,
                            context: Some("call_expression".to_string()),
                        });
                    }
                    pushed = true;
                }
            }
        } else {
            for def in f_definitions {
                if let Some(scope) = &def.scope {
                    if scope.starts_with(path_text) {
                        let source_line = name_node.start_position().row + 1;
                        let target_line = def.line_number;
                        if source_line != target_line {
                            dependencies.push(Dependency {
                                source_line,
                                target_line,
                                symbol: name_text.to_string(),
                                dependency_type: DependencyType::FunctionCall,
                                context: Some("call_expression".to_string()),
                            });
                        }
                    }
                }
            }
        }
    }

    fn resolve_method_call(
        &self,
        function_node: Node,
        source_code: &str,
        definitions: &[Definition],
        dependencies: &mut Vec<Dependency>,
    ) {
        let method_node = function_node.child_by_field_name("field").unwrap();
        let method_text = method_node.utf8_text(source_code.as_bytes()).unwrap();
        let method_def = definitions.iter().find(|d| d.name == method_text);

        if let Some(m_def) = method_def {
            let source_line = method_node.start_position().row + 1;
            let target_line = m_def.line_number;
            if source_line != target_line {
                dependencies.push(Dependency {
                    source_line,
                    target_line,
                    symbol: method_text.to_string(),
                    dependency_type: DependencyType::FunctionCall,
                    context: Some("call_expression".to_string()),
                });
            }
        }
    }

    fn resolve_ufcs_call(
        &self,
        function_node: Node,
        source_code: &str,
        definitions: &[Definition],
        dependencies: &mut Vec<Dependency>,
    ) {
        if let (Some(path_node), Some(name_node)) = (
            function_node.child_by_field_name("path"),
            function_node.child_by_field_name("name"),
        ) {
            let name_text = name_node.utf8_text(source_code.as_bytes()).unwrap();

            let f_definitions: Vec<&Definition> =
                definitions.iter().filter(|d| d.name == name_text).collect();

            if let Some(candidates) = ufcs_scope_candidates_from_path(&path_node, source_code) {
                for cand in candidates {
                    if let Some(def) = f_definitions
                        .iter()
                        .find(|d| d.scope.as_deref().map(|s| s.starts_with(&cand)) == Some(true))
                    {
                        let source_line = name_node.start_position().row + 1;
                        let target_line = def.line_number;
                        if source_line != target_line {
                            dependencies.push(Dependency {
                                source_line,
                                target_line,
                                symbol: name_text.to_string(),
                                dependency_type: DependencyType::FunctionCall,
                                context: Some("ufcs_call_expression".to_string()),
                            });
                        }
                        break;
                    }
                }
            }
        }
    }

    fn resolve_arguments(
        &self,
        arguments_node: Node,
        source_code: &str,
        definitions: &[Definition],
        dependencies: &mut Vec<Dependency>,
        current_scope: &Option<String>,
    ) {
        let mut args_cursor = arguments_node.walk();
        for arg_child in arguments_node.children(&mut args_cursor) {
            if arg_child.kind() == "identifier" {
                let arg_usage_node = Usage {
                    node: arg_child,
                    kind: UsageKind::Identifier,
                    scope: current_scope.clone(),
                };
                self.add_dependency_if_needed(
                    dependencies,
                    source_code,
                    &arg_usage_node,
                    definitions,
                );
            }
        }
    }
}

// Extract UFCS scope candidates from a path AST node like `<Type as Trait>`.
// Returns candidates in priority order: [Type, Trait].
fn ufcs_scope_candidates_from_path<'a>(
    path_node: &Node<'a>,
    source_code: &'a str,
) -> Option<Vec<String>> {
    if path_node.kind() != "bracketed_type" {
        return None;
    }

    // Find `qualified_type` inside the bracketed_type
    let mut cursor = path_node.walk();
    for child in path_node.children(&mut cursor) {
        if child.kind() == "qualified_type" {
            // Collect first two identifier-like children as [Type, Trait]
            let mut names: Vec<String> = Vec::new();
            let mut qcur = child.walk();
            for n in child.children(&mut qcur) {
                let k = n.kind();
                if k == "type_identifier" || k == "scoped_type_identifier" || k == "generic_type" {
                    if let Ok(txt) = n.utf8_text(source_code.as_bytes()) {
                        // Take the last segment of scoped type if any
                        let name = txt.split("::").last().unwrap_or(txt).to_string();
                        if !name.is_empty() {
                            names.push(name);
                        }
                    }
                }
            }
            if names.len() >= 2 {
                return Some(vec![names[0].clone(), names[1].clone()]);
            }
        }
    }
    None
}
