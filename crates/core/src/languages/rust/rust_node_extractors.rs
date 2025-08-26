use tree_sitter::Node;

use crate::models::{
    ast_traverser::{NodeDefinitionExtractor, NodeUsageExtractor},
    Definition, DefinitionType, Position, ScopeId, ScopeType, Usage, UsageKind,
};

/// Rust-specific definition extractor
pub struct RustDefinitionExtractor;

impl NodeDefinitionExtractor for RustDefinitionExtractor {
    fn extract_definition(&self, node: Node, scope: ScopeId, source: &str) -> Vec<Definition> {
        match node.kind() {
            // Scope-creating items: definitions go to PARENT scope
            "function_item" => {
                // Skip if this function is inside an impl block (it will be handled as MethodDefinition)
                if self.is_in_impl_scope(node) {
                    self.extract_method_definition(node, scope, source)
                        .into_iter()
                        .collect()
                } else {
                    self.extract_function_definition(node, scope, source)
                        .into_iter()
                        .collect()
                }
            }
            "struct_item" => self
                .extract_struct_definition(node, scope, source)
                .into_iter()
                .collect(),
            "union_item" => self
                .extract_union_definition(node, scope, source)
                .into_iter()
                .collect(),
            "enum_item" => self
                .extract_enum_definition(node, scope, source)
                .into_iter()
                .collect(),
            "trait_item" => self
                .extract_trait_definition(node, scope, source)
                .into_iter()
                .collect(),
            "impl_item" => vec![], // impl items don't create definitions themselves
            "mod_item" => self
                .extract_module_definition(node, scope, source)
                .into_iter()
                .collect(),

            // Non-scope-creating items: definitions go to CURRENT scope
            "let_declaration" => self.extract_let_definition(node, scope, source),
            "const_item" => self
                .extract_const_definition(node, scope, source)
                .into_iter()
                .collect(),
            "static_item" => self
                .extract_static_definition(node, scope, source)
                .into_iter()
                .collect(),
            "type_item" => self
                .extract_type_alias_definition(node, scope, source)
                .into_iter()
                .collect(),
            "parameter" => self
                .extract_parameter_definition(node, scope, source)
                .into_iter()
                .collect(),
            "function_signature_item" => self
                .extract_function_signature_definition(node, scope, source)
                .into_iter()
                .collect(),
            "associated_type" => self
                .extract_associated_type_definition(node, scope, source)
                .into_iter()
                .collect(),
            "field_declaration" => self
                .extract_field_definition(node, scope, source)
                .into_iter()
                .collect(),
            "use_declaration" => self.extract_import_definition(node, scope, source),
            "macro_definition" => self
                .extract_macro_definition(node, scope, source)
                .into_iter()
                .collect(),
            "metavariable" => self
                .extract_metavariable_definition(node, scope, source)
                .into_iter()
                .collect(),
            "type_parameter" => self
                .extract_type_parameter_definition(node, scope, source)
                .into_iter()
                .collect(),
            "constrained_type_parameter" => self
                .extract_constrained_type_parameter_definition(node, scope, source)
                .into_iter()
                .collect(),
            "type_parameters" => self.extract_type_parameters_definitions(node, scope, source),
            "identifier" => {
                // Check if this identifier is in a definition context
                if let Some(parent) = node.parent() {
                    match parent.kind() {
                        "closure_parameters" => {
                            return self
                                .extract_closure_parameter_definition(node, scope, source)
                                .into_iter()
                                .collect();
                        }
                        _ => {
                            // Check if this is a pattern binding in if let, for, while let, etc.
                            if self.is_pattern_binding(node) {
                                return self
                                    .extract_pattern_binding_definition(node, scope, source)
                                    .into_iter()
                                    .collect();
                            }
                            // Also check direct for_expression pattern
                            if self.is_for_loop_pattern(node) {
                                return self
                                    .extract_pattern_binding_definition(node, scope, source)
                                    .into_iter()
                                    .collect();
                            }
                        }
                    }
                }
                vec![]
            }
            _ => vec![],
        }
    }

    fn creates_scope(&self, node: Node) -> Option<(ScopeType, Position)> {
        let scope_type = match node.kind() {
            "function_item" => ScopeType::Function,
            "impl_item" => ScopeType::Impl,
            "trait_item" => ScopeType::Trait,
            "struct_item" => ScopeType::Block, // Structs create block-like scopes for their fields
            "union_item" => ScopeType::Block,  // Unions create block-like scopes for their fields
            "enum_item" => ScopeType::Block,   // Enums create block-like scopes for their variants
            "block" => ScopeType::Block,
            "mod_item" => ScopeType::Module,
            "closure_expression" => ScopeType::Closure,
            "for_expression" | "while_expression" | "if_expression" | "match_expression" => {
                ScopeType::Block
            }
            _ => return None,
        };

        Some((scope_type, Position::from_node(&node)))
    }
}

impl RustDefinitionExtractor {
    fn is_in_impl_scope(&self, node: Node) -> bool {
        // Check if this function is directly inside an impl_item
        let mut current = node.parent();
        while let Some(parent) = current {
            match parent.kind() {
                "impl_item" => return true,
                "source_file" => return false,
                _ => current = parent.parent(),
            }
        }
        false
    }

    fn extract_function_definition(
        &self,
        node: Node,
        scope: ScopeId,
        source: &str,
    ) -> Option<Definition> {
        let name = self.find_child_by_field_name(node, "name")?;
        let name_text = name.utf8_text(source.as_bytes()).ok()?;

        Some(Definition {
            name: name_text.to_string(),
            definition_type: DefinitionType::FunctionDefinition,
            position: Position::from_node(&name),
            scope_id: Some(scope),
            accessibility: None, // Will be set by ASTScopeTraverser to ScopeLocal
            is_hoisted: Some(false),
        })
    }

    fn extract_method_definition(
        &self,
        node: Node,
        scope: ScopeId,
        source: &str,
    ) -> Option<Definition> {
        let name = self.find_child_by_field_name(node, "name")?;
        let name_text = name.utf8_text(source.as_bytes()).ok()?;

        Some(Definition {
            name: name_text.to_string(),
            definition_type: DefinitionType::MethodDefinition,
            position: Position::from_node(&name),
            scope_id: Some(scope),
            accessibility: None, // Will be set by ASTScopeTraverser to ScopeLocal
            is_hoisted: Some(false),
        })
    }

    fn extract_let_definition(&self, node: Node, scope: ScopeId, source: &str) -> Vec<Definition> {
        use crate::definition_collectors::find_identifier_nodes_in_node;

        let pattern = match self.find_child_by_field_name(node, "pattern") {
            Some(p) => p,
            None => return vec![],
        };

        // Use find_identifier_nodes_in_node to handle complex patterns like tuple_pattern
        find_identifier_nodes_in_node(pattern)
            .into_iter()
            .filter_map(|identifier_node| {
                let name_text = identifier_node.utf8_text(source.as_bytes()).ok()?;
                Some(Definition {
                    name: name_text.to_string(),
                    definition_type: DefinitionType::VariableDefinition,
                    position: Position::from_node(&identifier_node),
                    scope_id: Some(scope),
                    accessibility: None, // Will be set by ASTScopeTraverser to ScopeLocal
                    is_hoisted: Some(false),
                })
            })
            .collect()
    }

    fn extract_function_signature_definition(
        &self,
        node: Node,
        scope: ScopeId,
        source: &str,
    ) -> Option<Definition> {
        let name = self.find_child_by_field_name(node, "name")?;
        let name_text = name.utf8_text(source.as_bytes()).ok()?;

        Some(Definition {
            name: name_text.to_string(),
            definition_type: DefinitionType::FunctionDefinition,
            position: Position::from_node(&name),
            scope_id: Some(scope),
            accessibility: None, // Will be set by ASTScopeTraverser to ScopeLocal
            is_hoisted: Some(false),
        })
    }

    fn extract_associated_type_definition(
        &self,
        node: Node,
        scope: ScopeId,
        source: &str,
    ) -> Option<Definition> {
        let name = self.find_child_by_field_name(node, "name")?;
        let name_text = name.utf8_text(source.as_bytes()).ok()?;

        Some(Definition {
            name: name_text.to_string(),
            definition_type: DefinitionType::TypeDefinition,
            position: Position::from_node(&name),
            scope_id: Some(scope),
            accessibility: None, // Will be set by ASTScopeTraverser to ScopeLocal
            is_hoisted: Some(false),
        })
    }

    fn extract_trait_definition(
        &self,
        node: Node,
        scope: ScopeId,
        source: &str,
    ) -> Option<Definition> {
        let name = self.find_child_by_field_name(node, "name")?;
        let name_text = name.utf8_text(source.as_bytes()).ok()?;

        Some(Definition {
            name: name_text.to_string(),
            definition_type: DefinitionType::TypeDefinition, // Match old implementation
            position: Position::from_node(&name),
            scope_id: Some(scope),
            accessibility: None, // Will be set by ASTScopeTraverser to ScopeLocal
            is_hoisted: Some(false),
        })
    }

    fn extract_field_definition(
        &self,
        node: Node,
        scope: ScopeId,
        source: &str,
    ) -> Option<Definition> {
        let name = self.find_child_by_field_name(node, "name")?;
        let name_text = name.utf8_text(source.as_bytes()).ok()?;

        Some(Definition {
            name: name_text.to_string(),
            definition_type: DefinitionType::StructFieldDefinition,
            position: Position::from_node(&name),
            scope_id: Some(scope),
            accessibility: None, // Will be set by ASTScopeTraverser to ScopeLocal
            is_hoisted: Some(false),
        })
    }

    fn extract_import_definition(
        &self,
        node: Node,
        scope: ScopeId,
        source: &str,
    ) -> Vec<Definition> {
        let mut definitions = vec![];

        if let Some(argument) = node.child_by_field_name("argument") {
            match argument.kind() {
                "scoped_identifier" => {
                    // Simple use: use my_module::MyStruct
                    if let Some(name_node) = argument.child_by_field_name("name") {
                        if let Ok(name_text) = name_node.utf8_text(source.as_bytes()) {
                            definitions.push(Definition {
                                name: name_text.to_string(),
                                definition_type: DefinitionType::ImportDefinition,
                                position: Position::from_node(&name_node),
                                scope_id: Some(scope),
                                accessibility: None,
                                is_hoisted: Some(false),
                            });
                        }
                    }
                }
                "scoped_use_list" => {
                    // use my_module::{my_function, MY_CONST}
                    if let Some(list_node) = argument.child_by_field_name("list") {
                        let mut cursor = list_node.walk();
                        for child in list_node.children(&mut cursor) {
                            match child.kind() {
                                "identifier" => {
                                    if let Ok(name_text) = child.utf8_text(source.as_bytes()) {
                                        definitions.push(Definition {
                                            name: name_text.to_string(),
                                            definition_type: DefinitionType::ImportDefinition,
                                            position: Position::from_node(&child),
                                            scope_id: Some(scope),
                                            accessibility: None,
                                            is_hoisted: Some(false),
                                        });
                                    }
                                }
                                "scoped_identifier" => {
                                    // For items like module::function in the use list
                                    if let Some(name_node) = child.child_by_field_name("name") {
                                        if let Ok(name_text) =
                                            name_node.utf8_text(source.as_bytes())
                                        {
                                            definitions.push(Definition {
                                                name: name_text.to_string(),
                                                definition_type: DefinitionType::ImportDefinition,
                                                position: Position::from_node(&name_node),
                                                scope_id: Some(scope),
                                                accessibility: None,
                                                is_hoisted: Some(false),
                                            });
                                        }
                                    }
                                }
                                _ => {}
                            }
                        }
                    }
                }
                "use_as_clause" => {
                    // use my_module as mm
                    if let Some(alias_node) = argument.child_by_field_name("alias") {
                        if let Ok(name_text) = alias_node.utf8_text(source.as_bytes()) {
                            definitions.push(Definition {
                                name: name_text.to_string(),
                                definition_type: DefinitionType::ImportDefinition,
                                position: Position::from_node(&alias_node),
                                scope_id: Some(scope),
                                accessibility: None,
                                is_hoisted: Some(false),
                            });
                        }
                    }
                }
                _ => {}
            }
        }

        definitions
    }

    fn extract_macro_definition(
        &self,
        node: Node,
        scope: ScopeId,
        source: &str,
    ) -> Option<Definition> {
        let name = self.find_child_by_field_name(node, "name")?;
        let name_text = name.utf8_text(source.as_bytes()).ok()?;

        Some(Definition {
            name: name_text.to_string(),
            definition_type: DefinitionType::MacroDefinition,
            position: Position::from_node(&name),
            scope_id: Some(scope),
            accessibility: None, // Will be set by ASTScopeTraverser to ScopeLocal
            is_hoisted: Some(false),
        })
    }

    fn extract_metavariable_definition(
        &self,
        node: Node,
        scope: ScopeId,
        source: &str,
    ) -> Option<Definition> {
        // Only treat metavariables as definitions if they're in macro patterns, not in macro body
        if self.is_metavariable_in_pattern(node) {
            let name_text = node.utf8_text(source.as_bytes()).ok()?;

            Some(Definition {
                name: name_text.to_string(),
                definition_type: DefinitionType::MacroVariableDefinition,
                position: Position::from_node(&node),
                scope_id: Some(scope),
                accessibility: None, // Will be set by ASTScopeTraverser to ScopeLocal
                is_hoisted: Some(false),
            })
        } else {
            None
        }
    }

    fn extract_type_parameter_definition(
        &self,
        node: Node,
        scope: ScopeId,
        source: &str,
    ) -> Option<Definition> {
        let name = self.find_child_by_field_name(node, "name")?;
        let name_text = name.utf8_text(source.as_bytes()).ok()?;

        Some(Definition {
            name: name_text.to_string(),
            definition_type: DefinitionType::TypeDefinition,
            position: Position::from_node(&name),
            scope_id: Some(scope),
            accessibility: None, // Will be set by ASTScopeTraverser to ScopeLocal
            is_hoisted: Some(false),
        })
    }

    fn extract_constrained_type_parameter_definition(
        &self,
        node: Node,
        scope: ScopeId,
        source: &str,
    ) -> Option<Definition> {
        if let Some(first_child) = node.child(0) {
            if first_child.kind() == "type_identifier" {
                let name_text = first_child.utf8_text(source.as_bytes()).ok()?;

                return Some(Definition {
                    name: name_text.to_string(),
                    definition_type: DefinitionType::TypeDefinition,
                    position: Position::from_node(&first_child),
                    scope_id: Some(scope),
                    accessibility: None, // Will be set by ASTScopeTraverser to ScopeLocal
                    is_hoisted: Some(false),
                });
            }
        }
        None
    }

    fn extract_struct_definition(
        &self,
        node: Node,
        scope: ScopeId,
        source: &str,
    ) -> Option<Definition> {
        let name = self.find_child_by_field_name(node, "name")?;
        let name_text = name.utf8_text(source.as_bytes()).ok()?;

        Some(Definition {
            name: name_text.to_string(),
            definition_type: DefinitionType::StructDefinition,
            position: Position::from_node(&name),
            scope_id: Some(scope),
            accessibility: None, // Will be set by ASTScopeTraverser to ScopeLocal
            is_hoisted: Some(false),
        })
    }

    fn extract_union_definition(
        &self,
        node: Node,
        scope: ScopeId,
        source: &str,
    ) -> Option<Definition> {
        let name = self.find_child_by_field_name(node, "name")?;
        let name_text = name.utf8_text(source.as_bytes()).ok()?;

        Some(Definition {
            name: name_text.to_string(),
            definition_type: DefinitionType::StructDefinition, // Unions are similar to structs in Rust
            position: Position::from_node(&name),
            scope_id: Some(scope),
            accessibility: None, // Will be set by ASTScopeTraverser to ScopeLocal
            is_hoisted: Some(false),
        })
    }

    fn extract_enum_definition(
        &self,
        node: Node,
        scope: ScopeId,
        source: &str,
    ) -> Option<Definition> {
        let name = self.find_child_by_field_name(node, "name")?;
        let name_text = name.utf8_text(source.as_bytes()).ok()?;

        Some(Definition {
            name: name_text.to_string(),
            definition_type: DefinitionType::EnumDefinition,
            position: Position::from_node(&name),
            scope_id: Some(scope),
            accessibility: None, // Will be set by ASTScopeTraverser to ScopeLocal
            is_hoisted: Some(false),
        })
    }

    #[allow(dead_code)]
    fn extract_impl_definition(
        &self,
        node: Node,
        scope: ScopeId,
        source: &str,
    ) -> Option<Definition> {
        // For impl blocks, we might want to extract the type being implemented
        let type_node = self.find_child_by_field_name(node, "type")?;
        let type_text = type_node.utf8_text(source.as_bytes()).ok()?;

        Some(Definition {
            name: format!("impl {}", type_text),
            definition_type: DefinitionType::ClassDefinition,
            position: Position::from_node(&type_node),
            scope_id: Some(scope),
            accessibility: None, // Will be set by ASTScopeTraverser to ScopeLocal
            is_hoisted: Some(false),
        })
    }

    fn extract_module_definition(
        &self,
        node: Node,
        scope: ScopeId,
        source: &str,
    ) -> Option<Definition> {
        let name = self.find_child_by_field_name(node, "name")?;
        let name_text = name.utf8_text(source.as_bytes()).ok()?;

        Some(Definition {
            name: name_text.to_string(),
            definition_type: DefinitionType::ModuleDefinition,
            position: Position::from_node(&name),
            scope_id: Some(scope),
            accessibility: None, // Will be set by ASTScopeTraverser to ScopeLocal
            is_hoisted: Some(false),
        })
    }

    #[allow(dead_code)]
    fn extract_variable_definition(
        &self,
        node: Node,
        scope: ScopeId,
        source: &str,
    ) -> Option<Definition> {
        let pattern = self.find_child_by_field_name(node, "pattern")?;
        let name = if pattern.kind() == "identifier" {
            pattern
        } else {
            // Handle more complex patterns if needed
            return None;
        };

        let name_text = name.utf8_text(source.as_bytes()).ok()?;

        Some(Definition {
            name: name_text.to_string(),
            definition_type: DefinitionType::VariableDefinition,
            position: Position::from_node(&name),
            scope_id: Some(scope),
            accessibility: None, // Will be set by ASTScopeTraverser to ScopeLocal
            is_hoisted: Some(false),
        })
    }

    fn extract_const_definition(
        &self,
        node: Node,
        scope: ScopeId,
        source: &str,
    ) -> Option<Definition> {
        let name = self.find_child_by_field_name(node, "name")?;
        let name_text = name.utf8_text(source.as_bytes()).ok()?;

        Some(Definition {
            name: name_text.to_string(),
            definition_type: DefinitionType::ConstDefinition,
            position: Position::from_node(&name),
            scope_id: Some(scope),
            accessibility: None, // Will be set by ASTScopeTraverser to ScopeLocal
            is_hoisted: Some(false),
        })
    }

    fn extract_static_definition(
        &self,
        node: Node,
        scope: ScopeId,
        source: &str,
    ) -> Option<Definition> {
        let name = self.find_child_by_field_name(node, "name")?;
        let name_text = name.utf8_text(source.as_bytes()).ok()?;

        Some(Definition {
            name: name_text.to_string(),
            definition_type: DefinitionType::VariableDefinition,
            position: Position::from_node(&name),
            scope_id: Some(scope),
            accessibility: None, // Will be set by ASTScopeTraverser to ScopeLocal
            is_hoisted: Some(false),
        })
    }

    fn extract_type_alias_definition(
        &self,
        node: Node,
        scope: ScopeId,
        source: &str,
    ) -> Option<Definition> {
        let name = self.find_child_by_field_name(node, "name")?;
        let name_text = name.utf8_text(source.as_bytes()).ok()?;

        Some(Definition {
            name: name_text.to_string(),
            definition_type: DefinitionType::TypeDefinition,
            position: Position::from_node(&name),
            scope_id: Some(scope),
            accessibility: None, // Will be set by ASTScopeTraverser to ScopeLocal
            is_hoisted: Some(false),
        })
    }

    fn extract_parameter_definition(
        &self,
        node: Node,
        scope: ScopeId,
        source: &str,
    ) -> Option<Definition> {
        let pattern = self.find_child_by_field_name(node, "pattern")?;
        let name = if pattern.kind() == "identifier" {
            pattern
        } else {
            return None;
        };

        let name_text = name.utf8_text(source.as_bytes()).ok()?;

        Some(Definition {
            name: name_text.to_string(),
            definition_type: DefinitionType::VariableDefinition,
            position: Position::from_node(&name),
            scope_id: Some(scope),
            accessibility: None, // Will be set by ASTScopeTraverser to ScopeLocal
            is_hoisted: Some(false),
        })
    }

    fn extract_type_parameters_definitions(
        &self,
        node: Node,
        scope: ScopeId,
        source: &str,
    ) -> Vec<Definition> {
        let mut definitions = vec![];
        let mut cursor = node.walk();

        for child in node.children(&mut cursor) {
            if child.kind() == "type_identifier" {
                if let Ok(name_text) = child.utf8_text(source.as_bytes()) {
                    definitions.push(Definition {
                        name: name_text.to_string(),
                        definition_type: DefinitionType::TypeDefinition,
                        position: Position::from_node(&child),
                        scope_id: Some(scope),
                        accessibility: None, // Will be set by ASTScopeTraverser to ScopeLocal
                        is_hoisted: Some(false),
                    });
                }
            }
        }

        definitions
    }

    fn is_metavariable_in_pattern(&self, node: Node) -> bool {
        // Check if this metavariable is in a macro pattern (left side) or macro body (right side)
        let mut current = node.parent();
        while let Some(parent) = current {
            match parent.kind() {
                "token_tree_pattern" => {
                    // This is in the pattern part of a macro rule (definition)
                    return true;
                }
                "token_tree" => {
                    // This is in the body part of a macro rule (usage)
                    return false;
                }
                _ => {}
            }
            current = parent.parent();
        }
        false
    }

    fn find_child_by_field_name<'a>(&self, node: Node<'a>, field_name: &str) -> Option<Node<'a>> {
        node.child_by_field_name(field_name)
    }

    /// Determine accessibility for Rust definitions
    /// In Rust, items are private by default unless marked with `pub`
    #[allow(dead_code)]
    fn determine_rust_accessibility(
        &self,
        node: Node,
        source: &str,
    ) -> Option<crate::models::Accessibility> {
        // Look for `pub` visibility modifier
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "visibility_modifier" {
                // Has pub modifier - determine if it's public, pub(crate), pub(super), etc.
                if let Ok(vis_text) = child.utf8_text(source.as_bytes()) {
                    if vis_text.starts_with("pub(") {
                        return Some(crate::models::Accessibility::ScopeLocal);
                    } else if vis_text == "pub" {
                        return Some(crate::models::Accessibility::Public);
                    }
                }
            }
        }
        // No pub modifier found - private by default in Rust
        Some(crate::models::Accessibility::Private)
    }
}

/// Rust-specific usage extractor
pub struct RustUsageExtractor;

impl NodeUsageExtractor for RustUsageExtractor {
    fn extract_usage(&self, node: Node, scope: ScopeId, source: &str) -> Option<Usage> {
        let kind = match node.kind() {
            "identifier" => {
                // Only treat identifier as usage if it's not in a definition context
                // and not the function name part of a call_expression (to avoid duplication)
                if self.is_identifier_in_definition_context(node)
                    || self.is_function_name_in_call_expression(node)
                    || self.is_identifier_part_of_field_access(node, source)
                {
                    None
                } else if self.is_identifier_in_type_context(node) {
                    // Check if this identifier is in a type context (like use statements)
                    Some(UsageKind::TypeIdentifier)
                } else {
                    Some(UsageKind::Identifier)
                }
            }
            "type_identifier" => {
                // Only treat type_identifier as usage if it's not in a definition context
                // This is for dependency resolution to work correctly
                if self.is_identifier_in_definition_context(node) {
                    None
                } else {
                    Some(UsageKind::TypeIdentifier)
                }
            }
            "call_expression" => {
                // Use special handling for call expressions to extract function name
                return self.extract_call_usage(node, scope, source);
            }
            "field_expression" => {
                // Use special handling for field expressions to extract field name
                return self.extract_field_usage(node, scope, source);
            }
            "struct_expression" => Some(UsageKind::StructExpression),
            "metavariable" => Some(UsageKind::Metavariable),
            _ => None,
        };

        kind.and_then(|k| {
            let name_text = node.utf8_text(source.as_bytes()).ok()?;
            Some(Usage {
                name: name_text.to_string(),
                kind: k,
                position: Position::from_node(&node),
                context: self.get_node_context(&node),
                scope_id: Some(scope),
            })
        })
    }
}

impl RustUsageExtractor {
    fn is_identifier_in_definition_context(&self, node: Node) -> bool {
        // Use the same definition patterns as the original implementation
        if let Some(parent) = node.parent() {
            match parent.kind() {
                "let_declaration" => {
                    if let Some(pattern_field) = parent.child_by_field_name("pattern") {
                        return node.id() == pattern_field.id();
                    }
                }
                // Pattern types are definition contexts
                "tuple_pattern" | "slice_pattern" | "reference_pattern" | "ref_pattern" => {
                    // Identifiers inside patterns are definitions
                    return true;
                }
                "struct_pattern" => {
                    // Check if this is the type field (usage) or a field identifier (definition)
                    if let Some(type_field) = parent.child_by_field_name("type") {
                        if node.id() == type_field.id() {
                            // This is the struct type being matched against (usage)
                            return false;
                        }
                    }
                    // Other identifiers in struct_pattern are field bindings (definitions)
                    return true;
                }
                "parameter" => {
                    if let Some(pattern_field) = parent.child_by_field_name("pattern") {
                        return node.id() == pattern_field.id();
                    }
                }
                "for_expression" => {
                    if let Some(pattern_field) = parent.child_by_field_name("pattern") {
                        return node.id() == pattern_field.id();
                    }
                }
                "closure_parameters" => return true,
                "type_parameters" => return true,
                "lifetime" => return true,
                "trait_bounds" => return false,
                "where_clause" => return true,
                "bounded_type" => return true,
                "constrained_type_parameter" => return true,
                "function_item" | "struct_item" | "union_item" | "enum_item" | "trait_item"
                | "mod_item" | "const_item" | "static_item" | "type_item" | "associated_type" => {
                    if let Some(name_field) = parent.child_by_field_name("name") {
                        return node.id() == name_field.id();
                    }
                }
                _ => {}
            }
        }
        false
    }

    fn is_function_name_in_call_expression(&self, node: Node) -> bool {
        let mut current = node.parent();
        while let Some(parent) = current {
            match parent.kind() {
                "call_expression" => {
                    // For simple function calls, check if this is directly the function name
                    if let Some(function_node) = parent.child(0) {
                        if function_node.id() == node.id() {
                            return true;
                        }
                    }
                    return false;
                }
                "scoped_identifier" => {
                    // For qualified calls (e.g., HashMap::new), continue checking if this scoped_identifier
                    // is the function part of a call_expression, but don't exclude path components
                    current = parent.parent();
                    continue;
                }
                _ => current = parent.parent(),
            }
        }
        false
    }

    fn is_identifier_part_of_field_access(&self, node: Node, _source_code: &str) -> bool {
        // Check if this identifier is the field part of a field_expression
        if let Some(parent) = node.parent() {
            if parent.kind() == "field_expression" {
                if let Some(field_node) = parent.child_by_field_name("field") {
                    return node.id() == field_node.id();
                }
            }
        }
        false
    }

    fn is_identifier_in_type_context(&self, node: Node) -> bool {
        let mut current = node.parent();
        while let Some(parent) = current {
            match parent.kind() {
                "use_declaration" | "use_as_clause" | "scoped_use_list" | "use_list" => {
                    return true;
                }
                "scoped_identifier" => {
                    // Check if this scoped_identifier is in a type context
                    current = parent.parent();
                    continue;
                }
                _ => current = parent.parent(),
            }
        }
        false
    }

    fn get_node_context(&self, node: &Node) -> Option<String> {
        // Use the same logic as Usage::get_node_context from the original
        let mut current = node.parent();
        while let Some(parent) = current {
            match parent.kind() {
                "scoped_identifier" => return Some("scoped_identifier".to_string()),
                "field_expression" => return Some("field_expression".to_string()),
                "call_expression" => return Some("call_expression".to_string()),
                _ => current = parent.parent(),
            }
        }
        None
    }

    #[allow(dead_code)]
    fn extract_identifier_usage(&self, node: Node, scope: ScopeId, source: &str) -> Option<Usage> {
        let name_text = node.utf8_text(source.as_bytes()).ok()?;

        Some(Usage {
            name: name_text.to_string(),
            kind: UsageKind::Identifier,
            position: Position::from_node(&node),
            context: None,
            scope_id: Some(scope),
        })
    }

    fn extract_call_usage(&self, node: Node, scope: ScopeId, source: &str) -> Option<Usage> {
        // Use the same logic as Usage::new_call_expression from the original
        let function_name = if let Some(function_node) = node.child(0) {
            function_node
                .utf8_text(source.as_bytes())
                .unwrap_or("")
                .trim()
                .replace("\r\n", "\n")
        } else {
            // Fallback to full text if we can't get the function child
            node.utf8_text(source.as_bytes())
                .unwrap_or("")
                .trim()
                .replace("\r\n", "\n")
        };

        Some(Usage {
            name: function_name,
            kind: UsageKind::CallExpression,
            position: Position::from_node(&node),
            context: Some("call_expression".to_string()),
            scope_id: Some(scope),
        })
    }

    fn extract_field_usage(&self, node: Node, scope: ScopeId, source: &str) -> Option<Usage> {
        // Use the same logic as Usage::new_field_expression from the original
        let field_name = if let Some(field_node) = node.child_by_field_name("field") {
            field_node
                .utf8_text(source.as_bytes())
                .unwrap_or("")
                .trim()
                .replace("\r\n", "\n")
        } else if let Some(last_child) = node.child(node.child_count().saturating_sub(1)) {
            // Fallback: try the last child
            last_child
                .utf8_text(source.as_bytes())
                .unwrap_or("")
                .trim()
                .replace("\r\n", "\n")
        } else {
            // Final fallback to full text
            node.utf8_text(source.as_bytes())
                .unwrap_or("")
                .trim()
                .replace("\r\n", "\n")
        };

        Some(Usage {
            name: field_name,
            kind: UsageKind::FieldExpression,
            position: Position::from_node(&node),
            context: Some("field_expression".to_string()),
            scope_id: Some(scope),
        })
    }
}

// Add closure parameter extraction back to RustDefinitionExtractor
impl RustDefinitionExtractor {
    fn extract_closure_parameter_definition(
        &self,
        node: Node,
        scope: ScopeId,
        source: &str,
    ) -> Option<Definition> {
        let name_text = node.utf8_text(source.as_bytes()).ok()?;

        Some(Definition {
            name: name_text.to_string(),
            definition_type: DefinitionType::VariableDefinition,
            position: Position::from_node(&node),
            scope_id: Some(scope),
            accessibility: None, // Will be set by ASTScopeTraverser to ScopeLocal
            is_hoisted: Some(false),
        })
    }

    #[allow(dead_code)]
    fn extract_closure_definitions(
        &self,
        node: Node,
        _scope: ScopeId,
        source: &str,
    ) -> Vec<Definition> {
        let mut definitions = vec![];
        if let Some(params_node) = node.child_by_field_name("parameters") {
            definitions.extend(self.extract_closure_parameters(params_node, source));
        }
        definitions
    }

    #[allow(dead_code)]
    fn extract_closure_parameters(&self, node: Node, source: &str) -> Vec<Definition> {
        let mut definitions = vec![];
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "identifier" {
                definitions.push(Definition::new(
                    &child,
                    source,
                    DefinitionType::VariableDefinition,
                ));
            }
        }
        definitions
    }

    fn is_pattern_binding(&self, node: Node) -> bool {
        // Check if this identifier is inside a scoped_identifier (like Vec::<T>::new)
        // If so, it's a usage, not a pattern binding
        let mut current = node.parent();
        while let Some(parent) = current {
            if parent.kind() == "scoped_identifier" {
                return false;
            }
            current = parent.parent();
        }

        let mut current = node;

        // Traverse up to find pattern contexts
        while let Some(parent) = current.parent() {
            match parent.kind() {
                "for_expression" => {
                    if let Some(pattern_field) = parent.child_by_field_name("pattern") {
                        if self.is_child_of(node, pattern_field) {
                            return true;
                        }
                    }
                }
                "if_expression" | "while_expression" => {
                    if let Some(condition) = parent.child_by_field_name("condition") {
                        if condition.kind() == "let_condition" {
                            if let Some(pattern_field) = condition.child_by_field_name("pattern") {
                                if self.is_child_of(node, pattern_field) {
                                    return true;
                                }
                            }
                        }
                    }
                }
                "match_arm" => {
                    if let Some(pattern_field) = parent.child_by_field_name("pattern") {
                        if self.is_child_of(node, pattern_field) {
                            return true;
                        }
                    }
                }
                _ => {}
            }
            current = parent;
        }
        false
    }

    #[allow(clippy::only_used_in_recursion)]
    fn is_child_of(&self, child: Node, parent: Node) -> bool {
        let mut cursor = parent.walk();
        for descendant in parent.children(&mut cursor) {
            if descendant.id() == child.id() {
                return true;
            }
            if self.is_child_of(child, descendant) {
                return true;
            }
        }
        false
    }

    fn is_for_loop_pattern(&self, node: Node) -> bool {
        if let Some(parent) = node.parent() {
            if parent.kind() == "for_expression" {
                if let Some(pattern_field) = parent.child_by_field_name("pattern") {
                    return node.id() == pattern_field.id();
                }
            }
        }
        false
    }

    fn extract_pattern_binding_definition(
        &self,
        node: Node,
        scope: ScopeId,
        source: &str,
    ) -> Option<Definition> {
        let name_text = node.utf8_text(source.as_bytes()).ok()?;

        // Skip constructors and enum variants (start with uppercase)
        if name_text.chars().next().is_some_and(|c| c.is_uppercase()) {
            return None;
        }

        Some(Definition {
            name: name_text.to_string(),
            definition_type: DefinitionType::VariableDefinition,
            position: Position::from_node(&node),
            scope_id: Some(scope),
            accessibility: None, // Will be set by ASTScopeTraverser to ScopeLocal
            is_hoisted: Some(false),
        })
    }
}
