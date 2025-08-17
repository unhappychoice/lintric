use crate::collectors::common::dependency_collectors::DependencyCollector;
use crate::models::{Definition, Dependency, DependencyType};
use tree_sitter::Node;

pub struct RustDependencyCollector<'a> {
    source_code: &'a str,
}

impl<'a> RustDependencyCollector<'a> {
    pub fn new(source_code: &'a str) -> Self {
        Self { source_code }
    }
}

impl<'a> DependencyCollector<'a> for RustDependencyCollector<'a> {
    fn process_node(
        &self,
        node: Node<'a>,
        definitions: &[Definition],
        current_scope: &Option<String>,
    ) -> Vec<Dependency> {
        let mut dependencies = Vec::new();
        match node.kind() {
            "identifier" => {
                self.handle_identifier(node, &mut dependencies, definitions, current_scope);
            }
            "call_expression" => {
                self.handle_call_expression(node, &mut dependencies, definitions, current_scope);
            }
            "field_expression" => {
                self.handle_field_expression(node, &mut dependencies, definitions, current_scope);
            }
            "struct_expression" => {
                self.handle_struct_expression(node, &mut dependencies, definitions, current_scope);
            }
            "metavariable" => {
                self.handle_metavariable(node, &mut dependencies, definitions, current_scope);
            }
            _ => {}
        }
        dependencies
    }

    fn determine_scope(&self, node: &Node<'a>, parent_scope: &Option<String>) -> Option<String> {
        let new_scope_name = match node.kind() {
            "function_item" | "struct_item" | "enum_item" | "trait_item" => {
                node.child_by_field_name("name").map(|n| {
                    n.utf8_text(self.source_code.as_bytes())
                        .unwrap()
                        .trim()
                        .to_string()
                })
            }
            "impl_item" => node.child_by_field_name("type").map(|n| {
                n.utf8_text(self.source_code.as_bytes())
                    .unwrap()
                    .trim()
                    .to_string()
            }),
            _ => None,
        };

        if let Some(name) = new_scope_name {
            Some(
                parent_scope
                    .as_ref()
                    .map_or(name.clone(), |p| format!("{p}.{name}")),
            )
        } else {
            parent_scope.clone()
        }
    }

    fn handle_identifier(
        &self,
        node: Node<'a>,
        dependencies: &mut Vec<Dependency>,
        definitions: &[Definition],
        current_scope: &Option<String>,
    ) {
        let is_definition = definitions
            .iter()
            .filter(|d| d.name == node.utf8_text(self.source_code.as_bytes()).unwrap())
            .filter(|d| d.line_number == node.start_position().row + 1)
            .count()
            > 0;

        if is_definition {
            return;
        }

        let parent_kind = node.parent().map(|p| p.kind()).map(|k| k.to_string());

        let dependency_type = match &parent_kind {
            Some(parent_kind) if parent_kind == "call_expression" => DependencyType::FunctionCall,
            Some(parent_kind) if parent_kind == "macro_invocation" => {
                DependencyType::MacroInvocation
            }
            _ => DependencyType::VariableUse,
        };

        self.add_dependency_if_needed(
            dependencies,
            node,
            self.source_code,
            definitions,
            current_scope,
            dependency_type,
            parent_kind,
        );
    }

    fn handle_call_expression(
        &self,
        node: Node<'a>,
        dependencies: &mut Vec<Dependency>,
        definitions: &[Definition],
        current_scope: &Option<String>,
    ) {
        if let Some(function_node) = node.child_by_field_name("function") {
            if function_node.kind() == "identifier" {
                self.add_dependency_if_needed(
                    dependencies,
                    function_node,
                    self.source_code,
                    definitions,
                    current_scope,
                    DependencyType::FunctionCall,
                    Some("call_expression".to_string()),
                );
            } else if function_node.kind() == "scoped_identifier" {
                let path_node = function_node.child_by_field_name("path").unwrap();
                let path_text = path_node.utf8_text(self.source_code.as_bytes()).unwrap();

                let name_node = function_node.child_by_field_name("name").unwrap();
                let name_text = name_node.utf8_text(self.source_code.as_bytes()).unwrap();

                // Match definitions by path. If UFCS, consider both <Type as Trait> sides via AST.
                let f_definitions: Vec<&Definition> =
                    definitions.iter().filter(|d| d.name == name_text).collect();

                if let Some(candidates) =
                    ufcs_scope_candidates_from_path(&path_node, self.source_code)
                {
                    // Prefer impl for Type, then fallback to Trait
                    let mut pushed = false;
                    for cand in candidates {
                        if pushed {
                            break;
                        }
                        if let Some(def) = f_definitions.iter().find(|d| {
                            d.scope.as_deref().map(|s| s.starts_with(&cand)) == Some(true)
                        }) {
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
            } else if function_node.kind() == "field_expression" {
                let method_node = function_node.child_by_field_name("field").unwrap();
                let method_text = method_node.utf8_text(self.source_code.as_bytes()).unwrap();
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
            } else if function_node.kind() == "scoped_type_identifier" {
                // Handle UFCS: <Type as Trait>::function()
                // This node typically has a "path" and a "name" field, similar to scoped_identifier
                if let (Some(path_node), Some(name_node)) = (
                    function_node.child_by_field_name("path"),
                    function_node.child_by_field_name("name"),
                ) {
                    let name_text = name_node.utf8_text(self.source_code.as_bytes()).unwrap();

                    let f_definitions: Vec<&Definition> =
                        definitions.iter().filter(|d| d.name == name_text).collect();

                    if let Some(candidates) =
                        ufcs_scope_candidates_from_path(&path_node, self.source_code)
                    {
                        // Prefer impl for Type, then fallback to Trait
                        for cand in candidates {
                            if let Some(def) = f_definitions.iter().find(|d| {
                                d.scope.as_deref().map(|s| s.starts_with(&cand)) == Some(true)
                            }) {
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
        }

        if let Some(arguments_node) = node.child_by_field_name("arguments") {
            let mut args_cursor = arguments_node.walk();
            for arg_child in arguments_node.children(&mut args_cursor) {
                if arg_child.kind() == "identifier" {
                    self.add_dependency_if_needed(
                        dependencies,
                        arg_child,
                        self.source_code,
                        definitions,
                        current_scope,
                        DependencyType::VariableUse,
                        Some("arguments".to_string()),
                    );
                }
            }
        }
    }

    fn handle_field_expression(
        &self,
        node: Node<'a>,
        dependencies: &mut Vec<Dependency>,
        definitions: &[Definition],
        current_scope: &Option<String>,
    ) {
        if let Some(parent) = node.parent() {
            if parent.kind() == "call_expression"
                && parent.child_by_field_name("function") == Some(node)
            {
                // This field expression is the function part of a method call,
                // so we let handle_call_expression deal with it.
                return;
            }
        }

        if let Some(value_node) = node.child_by_field_name("value") {
            if value_node.kind() == "identifier" {
                self.add_dependency_if_needed(
                    dependencies,
                    value_node,
                    self.source_code,
                    definitions,
                    current_scope,
                    DependencyType::StructFieldAccess,
                    Some("field_access".to_string()),
                );
            }
        }
        if let Some(type_node) = node.child_by_field_name("field") {
            self.add_dependency_if_needed(
                dependencies,
                type_node,
                self.source_code,
                definitions,
                current_scope,
                DependencyType::StructFieldAccess,
                Some("field_access".to_string()),
            );
        }
    }

    fn handle_struct_expression(
        &self,
        node: Node<'a>,
        dependencies: &mut Vec<Dependency>,
        definitions: &[Definition],
        current_scope: &Option<String>,
    ) {
        if let Some(type_node) = node.child_by_field_name("type") {
            self.add_dependency_if_needed(
                dependencies,
                type_node,
                self.source_code,
                definitions,
                current_scope,
                DependencyType::TypeReference,
                Some("struct_instantiation".to_string()),
            );
        }
    }

    fn handle_metavariable(
        &self,
        node: Node<'a>,
        dependencies: &mut Vec<Dependency>,
        definitions: &[Definition],
        current_scope: &Option<String>,
    ) {
        self.add_dependency_if_needed(
            dependencies,
            node,
            self.source_code,
            definitions,
            current_scope,
            DependencyType::VariableUse,
            Some("metavariable_use".to_string()),
        );
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
