use tree_sitter::Node;

use super::format_string;
use crate::models::{
    ast_traverser::{NodeDefinitionExtractor, NodeUsageExtractor},
    Definition, DefinitionType, Position, ScopeId, ScopeType, Usage, UsageKind,
};
use crate::query::{DeclaredAs, Roles};

/// Rust-specific definition extractor
///
/// Declarations whose shape alone identifies them are located by
/// `queries/rust/definitions.scm`; the arms below handle the rest, where classifying needs more
/// than the node — a `function_item` is a method inside an impl and a function outside it.
pub struct RustDefinitionExtractor {
    declared_types: Roles<DeclaredAs>,
}

impl RustDefinitionExtractor {
    /// Fails if the declaration query does not compile, which is a bug in the `.scm` file rather
    /// than anything about the source being analyzed.
    pub fn new(source_code: &str, root_node: Node) -> Result<Self, String> {
        Ok(Self {
            declared_types: super::definition_queries::declared_types(source_code, root_node)?,
        })
    }

    /// The declaration this node introduces, if the query located one here.
    fn queried_definition(&self, node: Node, scope: ScopeId, source: &str) -> Vec<Definition> {
        let Some(declared) = self.declared_types.get(&node.id()) else {
            return vec![];
        };
        let Ok(name) = node.utf8_text(source.as_bytes()) else {
            return vec![];
        };

        vec![Definition {
            name: name.to_string(),
            definition_type: declared.definition_type.clone(),
            position: Position::from_node(&node),
            scope_id: Some(scope),
            accessibility: None,
            is_hoisted: Some(declared.is_hoisted),
        }]
    }
}

impl NodeDefinitionExtractor for RustDefinitionExtractor {
    fn extract_definition(&self, node: Node, scope: ScopeId, source: &str) -> Vec<Definition> {
        // The query is the primary source; the arms below are the exceptions it cannot express.
        // Asking it first also keeps a name node from being swallowed by the `identifier` arm.
        let queried = self.queried_definition(node, scope, source);
        if !queried.is_empty() {
            return queried;
        }

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
            "impl_item" => vec![], // impl items don't create definitions themselves

            // Non-scope-creating items: definitions go to CURRENT scope
            "let_declaration" => self.extract_let_definition(node, scope, source),
            "parameter" => self
                .extract_parameter_definition(node, scope, source)
                .into_iter()
                .collect(),
            "use_declaration" => self.extract_import_definition(node, scope, source),
            "metavariable" => self
                .extract_metavariable_definition(node, scope, source)
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
        let pattern = match self.find_child_by_field_name(node, "pattern") {
            Some(p) => p,
            None => return vec![],
        };

        self.find_pattern_bindings(pattern)
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
    fn extract_usage(&self, node: Node, scope: ScopeId, source: &str) -> Vec<Usage> {
        let kind = match node.kind() {
            "identifier" | "type_identifier" if self.is_self_type(node, source) => {
                // `Self` names a type without spelling it, so resolution needs the name it stands
                // for rather than the keyword
                return self
                    .extract_self_type_usage(node, scope, source)
                    .into_iter()
                    .collect();
            }
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
                return self
                    .extract_call_usage(node, scope, source)
                    .into_iter()
                    .collect();
            }
            "field_expression" => {
                // Use special handling for field expressions to extract field name
                return self
                    .extract_field_usage(node, scope, source)
                    .into_iter()
                    .collect();
            }
            "field_initializer" => {
                // `Point { x: value }` references the declaration of `x`, not just the value
                return self
                    .extract_field_initializer_usage(node, scope, source)
                    .into_iter()
                    .collect();
            }
            "field_identifier" | "shorthand_field_identifier" => {
                // A field named by a pattern references the field's declaration, the same way one
                // named by a struct literal does
                return self
                    .extract_pattern_field_usage(node, scope, source)
                    .into_iter()
                    .collect();
            }
            "shorthand_field_initializer" => {
                // `Point { x }` references the declaration of `x` as well as reading the binding,
                // which the inner identifier already covers
                return self
                    .extract_shorthand_field_initializer_usage(node, scope, source)
                    .into_iter()
                    .collect();
            }
            "struct_expression" => Some(UsageKind::StructExpression),
            "metavariable" => Some(UsageKind::Metavariable),
            "string_content" => {
                // Inline format string captures have no nodes of their own, so they are parsed
                // out of the literal rather than reached by traversal
                return format_string::capture_usages(node, scope, source);
            }
            _ => None,
        };

        kind.and_then(|k| {
            let name_text = node.utf8_text(source.as_bytes()).ok()?;
            Some(Usage {
                name: Usage::normalize_line_endings(name_text),
                kind: k,
                position: Position::from_node(&node),
                context: self.get_node_context(&node),
                scope_id: Some(scope),
            })
        })
        .into_iter()
        .collect()
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
                "tuple_struct_pattern" => {
                    // The type being matched is a reference; the elements are bindings.
                    if let Some(type_field) = parent.child_by_field_name("type") {
                        return node.id() != type_field.id();
                    }
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
                // `T` in `<T>` sits under a `type_parameter`, so the list above never sees it and
                // the declaration was being collected as a usage
                "type_parameter" => {
                    if let Some(name_field) = parent.child_by_field_name("name") {
                        return node.id() == name_field.id();
                    }
                    return true;
                }
                "lifetime" => return true,
                "trait_bounds" => return false,
                "where_clause" => return true,
                "bounded_type" => return true,
                "constrained_type_parameter" => return true,
                "function_item"
                | "struct_item"
                | "union_item"
                | "enum_item"
                | "trait_item"
                | "mod_item"
                | "const_item"
                | "static_item"
                | "type_item"
                | "associated_type"
                | "function_signature_item"
                | "enum_variant" => {
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
                // A path in type position is a different node, but it is still a path
                "scoped_type_identifier" => return Some("scoped_type_identifier".to_string()),
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

    /// The field named by `Point { x: value }`.
    fn extract_field_initializer_usage(
        &self,
        node: Node,
        scope: ScopeId,
        source: &str,
    ) -> Option<Usage> {
        self.field_reference(node.child_by_field_name("field")?, scope, source)
    }

    fn is_self_type(&self, node: Node, source: &str) -> bool {
        node.utf8_text(source.as_bytes()) == Ok("Self")
    }

    /// `Self`, resolved to the type it stands for.
    ///
    /// Rewriting the usage to that name is what lets ordinary resolution reach the type's own
    /// declaration. The position stays on the keyword and the context records the rewrite, so
    /// `debug ir` still shows where the name came from.
    fn extract_self_type_usage(&self, node: Node, scope: ScopeId, source: &str) -> Option<Usage> {
        let name = self.enclosing_self_type(node, source)?;

        Some(Usage {
            name,
            kind: UsageKind::TypeIdentifier,
            position: Position::from_node(&node),
            context: Some("self_type".to_string()),
            scope_id: Some(scope),
        })
    }

    /// The name `Self` stands for: the type of the nearest enclosing `impl`, or the trait itself
    /// inside a trait declaration.
    fn enclosing_self_type(&self, node: Node, source: &str) -> Option<String> {
        let mut current = node.parent();

        while let Some(parent) = current {
            let named = match parent.kind() {
                "impl_item" => parent.child_by_field_name("type"),
                "trait_item" => parent.child_by_field_name("name"),
                _ => None,
            };

            if let Some(named) = named {
                return self.type_name_text(named, source);
            }

            current = parent.parent();
        }

        None
    }

    /// The bare name of a type, looking through generic arguments so that `Container<T>` yields
    /// `Container`.
    fn type_name_text(&self, node: Node, source: &str) -> Option<String> {
        let named = match node.kind() {
            "generic_type" => node.child_by_field_name("type")?,
            _ => node,
        };

        named.utf8_text(source.as_bytes()).ok().map(str::to_string)
    }

    /// The field named by a pattern, as in `let Point { x, y: renamed } = p`.
    ///
    /// Restricted to `field_pattern`, since the same node kinds appear in field declarations and
    /// field expressions, which are handled elsewhere.
    fn extract_pattern_field_usage(
        &self,
        node: Node,
        scope: ScopeId,
        source: &str,
    ) -> Option<Usage> {
        node.parent()
            .filter(|parent| parent.kind() == "field_pattern")
            .and_then(|_| self.field_reference(node, scope, source))
    }

    /// The field named by the shorthand `Point { x }`, whose identifier also reads a binding.
    fn extract_shorthand_field_initializer_usage(
        &self,
        node: Node,
        scope: ScopeId,
        source: &str,
    ) -> Option<Usage> {
        self.field_reference(node.child(0)?, scope, source)
    }

    fn field_reference(&self, node: Node, scope: ScopeId, source: &str) -> Option<Usage> {
        let name = node.utf8_text(source.as_bytes()).ok()?;

        Some(Usage {
            name: Usage::normalize_line_endings(name),
            kind: UsageKind::FieldInitializer,
            position: Position::from_node(&node),
            context: Some("field_initializer".to_string()),
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

    /// The identifiers a pattern binds.
    ///
    /// A pattern's `type:` field names the struct or variant being matched, so it is a reference
    /// rather than a binding and is skipped — otherwise `let S::F(v) = s` would register the enum
    /// and its variant as locals. A shorthand field pattern binds under the field's own name, which
    /// is a `shorthand_field_identifier` rather than an `identifier`.
    #[allow(clippy::only_used_in_recursion)]
    fn find_pattern_bindings<'a>(&self, pattern: Node<'a>) -> Vec<Node<'a>> {
        if matches!(pattern.kind(), "identifier" | "shorthand_field_identifier") {
            return vec![pattern];
        }

        let matched_type = pattern.child_by_field_name("type").map(|node| node.id());
        let mut cursor = pattern.walk();
        let children: Vec<Node<'a>> = pattern.children(&mut cursor).collect();

        children
            .into_iter()
            .filter(|child| Some(child.id()) != matched_type)
            .flat_map(|child| self.find_pattern_bindings(child))
            .collect()
    }

    #[allow(clippy::only_used_in_recursion, dead_code)]
    fn find_identifier_nodes_in_node<'a>(&self, node: Node<'a>) -> Vec<Node<'a>> {
        let mut identifiers = vec![];
        if node.kind() == "identifier" {
            identifiers.push(node);
        } else {
            let mut cursor = node.walk();
            for child in node.children(&mut cursor) {
                identifiers.extend(self.find_identifier_nodes_in_node(child));
            }
        }
        identifiers
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
