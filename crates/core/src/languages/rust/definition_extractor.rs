//! Locating what a Rust file declares.
//!
//! Declarations whose shape alone identifies them come from `queries/rust/definitions.scm`. The arms
//! here handle the rest, where classifying needs more than the node — a `function_item` is a method
//! inside an impl and a function outside it. What a pattern binds is its own question, answered in
//! `pattern_bindings`.

use tree_sitter::Node;

use crate::models::{
    ast_traverser::NodeDefinitionExtractor, Definition, DefinitionType, Position, ScopeId,
    ScopeType,
};
use crate::query::{DeclaredAs, Roles};

/// Rust-specific definition extractor
///
/// Declarations whose shape alone identifies them are located by
/// `queries/rust/definitions.scm`; the arms below handle the rest, where classifying needs more
/// than the node — a `function_item` is a method inside an impl and a function outside it.
pub struct RustDefinitionExtractor {
    declared_types: Roles<DeclaredAs>,
    scope_kinds: Roles<ScopeType>,
}

impl RustDefinitionExtractor {
    /// Fails if the declaration query does not compile, which is a bug in the `.scm` file rather
    /// than anything about the source being analyzed.
    pub fn new(source_code: &str, root_node: Node) -> Result<Self, String> {
        Ok(Self {
            declared_types: super::definition_queries::declared_types(source_code, root_node)?,
            scope_kinds: super::scope_queries::scope_kinds(source_code, root_node)?,
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
            "metavariable" => self
                .extract_metavariable_definition(node, scope, source)
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
        self.scope_kinds
            .get(&node.id())
            .map(|scope_type| (scope_type.clone(), Position::from_node(&node)))
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
