use tree_sitter::Node;

use crate::models::{
    ast_traverser::{NodeDefinitionExtractor, NodeUsageExtractor},
    Definition, DefinitionType, Position, ScopeId, ScopeType, Usage, UsageKind,
};

/// TypeScript-specific definition extractor
pub struct TypeScriptDefinitionExtractor;

impl NodeDefinitionExtractor for TypeScriptDefinitionExtractor {
    fn extract_definition(&self, node: Node, scope: ScopeId, source: &str) -> Vec<Definition> {
        match node.kind() {
            "function_declaration" => self
                .extract_function_definition(node, scope, source)
                .into_iter()
                .collect(),
            "method_definition" => self
                .extract_method_definition(node, scope, source)
                .into_iter()
                .collect(),
            "arrow_function" => self.extract_arrow_function_definition(node, scope, source),
            "class_declaration" => self
                .extract_class_definition(node, scope, source)
                .into_iter()
                .collect(),
            "interface_declaration" => self
                .extract_interface_definition(node, scope, source)
                .into_iter()
                .collect(),
            "type_alias_declaration" => self
                .extract_type_alias_definition(node, scope, source)
                .into_iter()
                .collect(),
            "enum_declaration" => self
                .extract_enum_definition(node, scope, source)
                .into_iter()
                .collect(),
            "namespace_declaration" | "internal_module" => self
                .extract_namespace_definition(node, scope, source)
                .into_iter()
                .collect(),
            "variable_declarator" => self.extract_variable_definition(node, scope, source),
            "formal_parameters" => self.extract_function_parameters(node, scope, source),
            "type_parameter" => self
                .extract_type_parameter_definition(node, scope, source)
                .into_iter()
                .collect(),
            "enum_body" => self.extract_enum_members(node, scope, source),
            "public_field_definition" | "private_field_definition" | "field_definition" => self
                .extract_field_definition(node, scope, source)
                .into_iter()
                .collect(),
            "property_signature" => self
                .extract_property_signature(node, scope, source)
                .into_iter()
                .collect(),
            "method_signature" => self
                .extract_method_signature(node, scope, source)
                .into_iter()
                .collect(),
            "import_specifier" => self
                .extract_import_specifier_definition(node, scope, source)
                .into_iter()
                .collect(),
            "import_statement" => self
                .extract_import_statement_definition(node, scope, source)
                .into_iter()
                .collect(),
            "export_statement" => self
                .extract_export_statement_definition(node, scope, source)
                .into_iter()
                .collect(),
            "namespace_import" => self
                .extract_namespace_import_definition(node, scope, source)
                .into_iter()
                .collect(),
            "import_clause" => self
                .extract_import_clause_definition(node, scope, source)
                .into_iter()
                .collect(),
            _ => vec![],
        }
    }

    fn creates_scope(&self, node: Node) -> Option<(ScopeType, Position)> {
        let scope_type = match node.kind() {
            "function_declaration" | "method_definition" | "arrow_function" => ScopeType::Function,
            "class_declaration" => ScopeType::Class,
            "interface_declaration" => ScopeType::Interface,
            "namespace_declaration" | "internal_module" => ScopeType::Module,
            "block" => ScopeType::Block,
            "for_statement" | "while_statement" | "if_statement" => ScopeType::Block,
            _ => return None,
        };

        Some((scope_type, Position::from_node(&node)))
    }
}

impl TypeScriptDefinitionExtractor {
    fn extract_function_definition(
        &self,
        node: Node,
        scope: ScopeId,
        source: &str,
    ) -> Option<Definition> {
        let name = self.find_child_by_field_name(node, "name")?;
        let name_text = name.utf8_text(source.as_bytes()).ok()?;

        Some(Definition {
            name: Usage::normalize_line_endings(name_text),
            definition_type: DefinitionType::FunctionDefinition,
            position: Position::from_node(&name),
            scope_id: Some(scope),
            accessibility: None, // Will be set by ASTScopeTraverser to ScopeLocal
            is_hoisted: Some(true), // Functions are hoisted in TypeScript/JavaScript
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
            name: Usage::normalize_line_endings(name_text),
            definition_type: DefinitionType::MethodDefinition,
            position: Position::from_node(&name),
            scope_id: Some(scope),
            accessibility: None, // Will be set by ASTScopeTraverser to ScopeLocal
            is_hoisted: Some(false),
        })
    }

    fn extract_arrow_function_definition(
        &self,
        node: Node,
        scope: ScopeId,
        source: &str,
    ) -> Vec<Definition> {
        let mut definitions = vec![];

        // Extract single parameter (without parentheses) if present
        if let Some(parameter_field) = node.child_by_field_name("parameter") {
            if parameter_field.kind() == "identifier" {
                let name_text = parameter_field.utf8_text(source.as_bytes()).ok();
                if let Some(name) = name_text {
                    definitions.push(Definition {
                        name: Usage::normalize_line_endings(name),
                        definition_type: DefinitionType::VariableDefinition,
                        position: Position::from_node(&parameter_field),
                        scope_id: Some(scope),
                        accessibility: None,
                        is_hoisted: Some(false),
                    });
                }
            }
        }

        definitions
    }

    fn extract_class_definition(
        &self,
        node: Node,
        scope: ScopeId,
        source: &str,
    ) -> Option<Definition> {
        let name = self.find_child_by_field_name(node, "name")?;
        let name_text = name.utf8_text(source.as_bytes()).ok()?;

        Some(Definition {
            name: Usage::normalize_line_endings(name_text),
            definition_type: DefinitionType::ClassDefinition,
            position: Position::from_node(&name),
            scope_id: Some(scope),
            accessibility: None, // Will be set by ASTScopeTraverser to ScopeLocal
            is_hoisted: Some(true), // Classes are hoisted
        })
    }

    fn extract_interface_definition(
        &self,
        node: Node,
        scope: ScopeId,
        source: &str,
    ) -> Option<Definition> {
        let name = self.find_child_by_field_name(node, "name")?;
        let name_text = name.utf8_text(source.as_bytes()).ok()?;

        Some(Definition {
            name: Usage::normalize_line_endings(name_text),
            definition_type: DefinitionType::InterfaceDefinition,
            position: Position::from_node(&name),
            scope_id: Some(scope),
            accessibility: None, // Will be set by ASTScopeTraverser to ScopeLocal
            is_hoisted: Some(true),
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
            name: Usage::normalize_line_endings(name_text),
            definition_type: DefinitionType::TypeDefinition,
            position: Position::from_node(&name),
            scope_id: Some(scope),
            accessibility: None, // Will be set by ASTScopeTraverser to ScopeLocal
            is_hoisted: Some(true),
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
            name: Usage::normalize_line_endings(name_text),
            definition_type: DefinitionType::EnumDefinition,
            position: Position::from_node(&name),
            scope_id: Some(scope),
            accessibility: None, // Will be set by ASTScopeTraverser to ScopeLocal
            is_hoisted: Some(true),
        })
    }

    fn extract_namespace_definition(
        &self,
        node: Node,
        scope: ScopeId,
        source: &str,
    ) -> Option<Definition> {
        let name = self.find_child_by_field_name(node, "name")?;
        let name_text = name.utf8_text(source.as_bytes()).ok()?;

        Some(Definition {
            name: Usage::normalize_line_endings(name_text),
            definition_type: DefinitionType::ModuleDefinition,
            position: Position::from_node(&name),
            scope_id: Some(scope),
            accessibility: None, // Will be set by ASTScopeTraverser to ScopeLocal
            is_hoisted: Some(true),
        })
    }

    fn extract_variable_definition(
        &self,
        node: Node,
        scope: ScopeId,
        source: &str,
    ) -> Vec<Definition> {
        let name_node = match self.find_child_by_field_name(node, "name") {
            Some(n) => n,
            None => return vec![],
        };

        // Check if it's a const declaration for hoisting
        let is_hoisted = if let Some(parent) = node.parent() {
            if let Some(grandparent) = parent.parent() {
                grandparent.kind() == "variable_declaration"
                    && grandparent.child_by_field_name("kind").map(|k| k.kind()) == Some("var")
            } else {
                false
            }
        } else {
            false
        };

        // Use find_identifier_nodes_in_node to handle destructuring patterns
        self.find_identifier_nodes_in_node(name_node)
            .into_iter()
            .filter_map(|identifier_node| {
                let name_text = identifier_node.utf8_text(source.as_bytes()).ok()?;
                Some(Definition {
                    name: Usage::normalize_line_endings(name_text),
                    definition_type: DefinitionType::VariableDefinition,
                    position: Position::from_node(&identifier_node),
                    scope_id: Some(scope),
                    accessibility: None, // Will be set by ASTScopeTraverser to ScopeLocal
                    is_hoisted: Some(is_hoisted),
                })
            })
            .collect()
    }

    fn extract_function_parameters(
        &self,
        node: Node,
        scope: ScopeId,
        source: &str,
    ) -> Vec<Definition> {
        let mut definitions = vec![];
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            match child.kind() {
                "required_parameter" | "optional_parameter" => {
                    if let Some(pattern_node) = child.child_by_field_name("pattern") {
                        // Find identifiers in the pattern node
                        let identifiers = self.find_identifier_nodes_in_node(pattern_node);
                        for identifier_node in identifiers {
                            let mut def = Definition::new(
                                &identifier_node,
                                source,
                                DefinitionType::VariableDefinition,
                            );
                            def.set_context(
                                scope,
                                &crate::models::Accessibility::ScopeLocal,
                                false,
                            );
                            definitions.push(def);
                        }
                    }
                }
                _ => {}
            }
        }
        definitions
    }

    #[allow(clippy::only_used_in_recursion)]
    fn find_identifier_nodes_in_node<'a>(&self, node: Node<'a>) -> Vec<Node<'a>> {
        let mut identifiers = vec![];
        if node.kind() == "identifier" {
            identifiers.push(node);
        } else if node.kind() == "shorthand_property_identifier_pattern" {
            // shorthand_property_identifier_pattern is itself an identifier in destructuring
            identifiers.push(node);
        } else {
            let mut cursor = node.walk();
            for child in node.children(&mut cursor) {
                identifiers.extend(self.find_identifier_nodes_in_node(child));
            }
        }
        identifiers
    }

    #[allow(dead_code)]
    fn extract_parameter_definition(
        &self,
        node: Node,
        scope: ScopeId,
        source: &str,
    ) -> Option<Definition> {
        let name = self
            .find_child_by_field_name(node, "pattern")
            .or_else(|| self.find_child_by_field_name(node, "name"))?;

        let name_text = if name.kind() == "identifier" {
            name.utf8_text(source.as_bytes()).ok()?
        } else {
            return None; // Complex patterns not supported yet
        };

        Some(Definition {
            name: Usage::normalize_line_endings(name_text),
            definition_type: DefinitionType::VariableDefinition, // Parameters are variables
            position: Position::from_node(&name),
            scope_id: Some(scope),
            accessibility: None, // Will be set by ASTScopeTraverser to ScopeLocal
            is_hoisted: Some(false),
        })
    }

    fn extract_property_signature(
        &self,
        node: Node,
        scope: ScopeId,
        source: &str,
    ) -> Option<Definition> {
        let name = self.find_child_by_field_name(node, "name")?;
        let name_text = name.utf8_text(source.as_bytes()).ok()?;

        Some(Definition {
            name: Usage::normalize_line_endings(name_text),
            definition_type: DefinitionType::PropertyDefinition,
            position: Position::from_node(&name),
            scope_id: Some(scope),
            accessibility: None, // Will be set by ASTScopeTraverser to ScopeLocal
            is_hoisted: Some(false),
        })
    }

    fn extract_method_signature(
        &self,
        node: Node,
        scope: ScopeId,
        source: &str,
    ) -> Option<Definition> {
        let name = self.find_child_by_field_name(node, "name")?;
        let name_text = name.utf8_text(source.as_bytes()).ok()?;

        Some(Definition {
            name: Usage::normalize_line_endings(name_text),
            definition_type: DefinitionType::MethodDefinition,
            position: Position::from_node(&name),
            scope_id: Some(scope),
            accessibility: None, // Will be set by ASTScopeTraverser to ScopeLocal
            is_hoisted: Some(false),
        })
    }

    fn extract_import_specifier_definition(
        &self,
        node: Node,
        scope: ScopeId,
        source: &str,
    ) -> Option<Definition> {
        let name = self.find_child_by_field_name(node, "name")?;
        let name_text = name.utf8_text(source.as_bytes()).ok()?;

        Some(Definition {
            name: Usage::normalize_line_endings(name_text),
            definition_type: DefinitionType::ImportDefinition,
            position: Position::from_node(&name),
            scope_id: Some(scope),
            accessibility: None, // Will be set by ASTScopeTraverser to ScopeLocal
            is_hoisted: Some(false), // Match old implementation
        })
    }

    fn extract_import_statement_definition(
        &self,
        _node: Node,
        _scope: ScopeId,
        _source: &str,
    ) -> Option<Definition> {
        None // import_statement itself doesn't create definitions
    }

    fn extract_export_statement_definition(
        &self,
        _node: Node,
        _scope: ScopeId,
        _source: &str,
    ) -> Option<Definition> {
        None // export_statement itself doesn't create definitions
    }

    fn extract_namespace_import_definition(
        &self,
        node: Node,
        scope: ScopeId,
        source: &str,
    ) -> Option<Definition> {
        // For namespace imports like: import * as Utils from './utils'
        if let Some(name_node) = node.child(2) {
            // The identifier after 'as'
            if name_node.kind() == "identifier" {
                let name_text = name_node.utf8_text(source.as_bytes()).ok()?;
                return Some(Definition {
                    name: Usage::normalize_line_endings(name_text),
                    definition_type: DefinitionType::ImportDefinition,
                    position: Position::from_node(&name_node),
                    scope_id: Some(scope),
                    accessibility: None,
                    is_hoisted: Some(false),
                });
            }
        }
        None
    }

    fn extract_import_clause_definition(
        &self,
        node: Node,
        scope: ScopeId,
        source: &str,
    ) -> Option<Definition> {
        // Handle default imports - direct identifier in import_clause
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "identifier" {
                let name_text = child.utf8_text(source.as_bytes()).ok()?;
                return Some(Definition {
                    name: Usage::normalize_line_endings(name_text),
                    definition_type: DefinitionType::ImportDefinition,
                    position: Position::from_node(&child),
                    scope_id: Some(scope),
                    accessibility: None,
                    is_hoisted: Some(false),
                });
            }
        }
        None
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
            name: Usage::normalize_line_endings(name_text),
            definition_type: DefinitionType::TypeDefinition,
            position: Position::from_node(&name),
            scope_id: Some(scope),
            accessibility: None, // Will be set by ASTScopeTraverser to ScopeLocal
            is_hoisted: Some(false),
        })
    }

    fn extract_enum_members(&self, node: Node, scope: ScopeId, source: &str) -> Vec<Definition> {
        let mut definitions = vec![];
        let mut cursor = node.walk();

        for child in node.children(&mut cursor) {
            if child.kind() == "property_identifier" {
                if let Ok(name_text) = child.utf8_text(source.as_bytes()) {
                    definitions.push(Definition {
                        name: Usage::normalize_line_endings(name_text),
                        definition_type: DefinitionType::PropertyDefinition,
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

    fn extract_field_definition(
        &self,
        node: Node,
        scope: ScopeId,
        source: &str,
    ) -> Option<Definition> {
        let name = self.find_child_by_field_name(node, "name")?;
        let name_text = name.utf8_text(source.as_bytes()).ok()?;

        Some(Definition {
            name: Usage::normalize_line_endings(name_text),
            definition_type: DefinitionType::PropertyDefinition,
            position: Position::from_node(&name),
            scope_id: Some(scope),
            accessibility: None, // Will be set by ASTScopeTraverser to ScopeLocal
            is_hoisted: Some(false),
        })
    }

    fn find_child_by_field_name<'a>(&self, node: Node<'a>, field_name: &str) -> Option<Node<'a>> {
        node.child_by_field_name(field_name)
    }

    /// Determine accessibility for TypeScript definitions
    /// TypeScript has explicit access modifiers: public, private, protected
    /// Default is public for top-level items, but can be private for class members
    #[allow(dead_code)]
    fn determine_typescript_accessibility(
        &self,
        node: Node,
        source: &str,
    ) -> Option<crate::models::Accessibility> {
        // Look for access modifiers
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "accessibility_modifier" {
                if let Ok(modifier_text) = child.utf8_text(source.as_bytes()) {
                    return match modifier_text {
                        "private" => Some(crate::models::Accessibility::Private),
                        "protected" => Some(crate::models::Accessibility::ScopeLocal),
                        "public" => Some(crate::models::Accessibility::Public),
                        _ => Some(crate::models::Accessibility::Public),
                    };
                }
            }
        }

        // Default accessibility depends on context
        // For class members without explicit modifier, default is public in TS
        // For variables and top-level functions, they're generally public
        if let Some(parent) = node.parent() {
            match parent.kind() {
                "class_body" => Some(crate::models::Accessibility::Public), // Default for class members
                _ => Some(crate::models::Accessibility::Public),            // Top-level items
            }
        } else {
            Some(crate::models::Accessibility::Public) // Top-level
        }
    }
}

/// TypeScript-specific usage extractor
pub struct TypeScriptUsageExtractor;

impl NodeUsageExtractor for TypeScriptUsageExtractor {
    fn extract_usage(&self, node: Node, scope: ScopeId, source: &str) -> Option<Usage> {
        match node.kind() {
            "identifier" => {
                if self.is_usage_context(node) {
                    self.extract_identifier_usage(node, scope, source)
                } else {
                    None
                }
            }
            "call_expression" => self.extract_call_usage(node, scope, source),
            // "new_expression" => self.extract_new_usage(node, scope, source), // Disable to match old implementation
            "type_identifier" => {
                if self.is_type_identifier_in_definition_context(node) {
                    None
                } else {
                    self.extract_type_usage(node, scope, source)
                }
            }
            "property_identifier" => self.extract_property_identifier_usage(node, scope, source),
            _ => None,
        }
    }
}

impl TypeScriptUsageExtractor {
    fn is_usage_context(&self, node: Node) -> bool {
        if let Some(parent) = node.parent() {
            match parent.kind() {
                // These are definition contexts, not usage
                "function_declaration"
                | "class_declaration"
                | "interface_declaration"
                | "type_alias_declaration"
                | "enum_declaration"
                | "namespace_declaration"
                | "internal_module"
                | "variable_declarator"
                | "method_definition"
                | "property_signature"
                | "method_signature"
                | "import_specifier" => {
                    // Check if this identifier is the name being defined
                    if let Some(name_field) = parent.child_by_field_name("name") {
                        return node.id() != name_field.id();
                    }
                }
                // Destructuring patterns are definition contexts, not usage
                "array_pattern" | "object_pattern" => {
                    // Identifiers inside destructuring patterns are definitions
                    return false;
                }
                "rest_pattern" => {
                    // Identifiers inside rest patterns (like ...rest) are definitions
                    return false;
                }
                "shorthand_property_identifier_pattern" => {
                    // Shorthand property identifiers in destructuring are definitions
                    return false;
                }
                "assignment_pattern" => {
                    // In patterns like [first = 1], 'first' is definition, '1' might be usage
                    if let Some(left_field) = parent.child_by_field_name("left") {
                        // If this node is the left side (the identifier being defined), it's not usage
                        return node.id() != left_field.id();
                    }
                }
                "pair_pattern" => {
                    // In patterns like { x: renamed }, only 'x' is usage, 'renamed' is definition
                    if let Some(value_field) = parent.child_by_field_name("value") {
                        // If this node is the value (the identifier being defined), it's not usage
                        return node.id() != value_field.id();
                    }
                }
                "required_parameter" | "optional_parameter" => {
                    // Check if this is the parameter name
                    if let Some(pattern_field) = parent.child_by_field_name("pattern") {
                        return node.id() != pattern_field.id();
                    }
                    if let Some(name_field) = parent.child_by_field_name("name") {
                        return node.id() != name_field.id();
                    }
                }
                "arrow_function" => {
                    // Check if this identifier is a parameter of the arrow function
                    // For single parameter arrow functions without parentheses: item => ...
                    if let Some(parameter_field) = parent.child_by_field_name("parameter") {
                        return node.id() != parameter_field.id();
                    }
                }
                "call_expression" => {
                    // Check if this identifier is the function name (function field) of call_expression
                    // If so, it should not be treated as a separate identifier usage
                    // because call_expression itself will handle the usage
                    if let Some(function_field) = parent.child_by_field_name("function") {
                        return node.id() != function_field.id();
                    }
                }
                _ => {}
            }
        }
        true
    }

    fn extract_identifier_usage(&self, node: Node, scope: ScopeId, source: &str) -> Option<Usage> {
        let name_text = node.utf8_text(source.as_bytes()).ok()?;

        // Determine context based on ancestor call_expression
        let context = self.find_call_expression_context(node);

        Some(Usage {
            name: Usage::normalize_line_endings(name_text),
            kind: UsageKind::Identifier,
            position: Position::from_node(&node),
            context,
            scope_id: Some(scope),
        })
    }

    fn find_call_expression_context(&self, node: Node) -> Option<String> {
        let mut current = node.parent();
        while let Some(parent) = current {
            if parent.kind() == "call_expression" {
                return Some("call_expression".to_string());
            }
            current = parent.parent();
        }
        None
    }

    fn extract_call_usage(&self, node: Node, scope: ScopeId, source: &str) -> Option<Usage> {
        // Extract function name from call_expression by getting the function field
        let function_name = if let Some(function_node) = node.child_by_field_name("function") {
            function_node
                .utf8_text(source.as_bytes())
                .unwrap_or("")
                .trim()
                .replace("\r\n", "\n")
        } else {
            // Fallback to full text if we can't get the function field
            node.utf8_text(source.as_bytes())
                .unwrap_or("")
                .trim()
                .replace("\r\n", "\n")
        };

        Some(Usage {
            name: function_name,
            kind: UsageKind::CallExpression,
            position: Position::from_node(&node), // Use the full call_expression range like old implementation
            context: Some("call_expression".to_string()), // Restore old implementation context
            scope_id: Some(scope),
        })
    }

    #[allow(dead_code)]
    fn extract_member_usage(&self, node: Node, scope: ScopeId, source: &str) -> Option<Usage> {
        let property_node = node.child_by_field_name("property")?;
        let name_text = property_node.utf8_text(source.as_bytes()).ok()?;

        Some(Usage {
            name: Usage::normalize_line_endings(name_text),
            kind: UsageKind::FieldExpression,
            position: Position::from_node(&property_node),
            context: Some("member_expression".to_string()),
            scope_id: Some(scope),
        })
    }

    #[allow(dead_code)]
    fn extract_new_usage(&self, node: Node, scope: ScopeId, source: &str) -> Option<Usage> {
        let constructor_node = node.child_by_field_name("constructor")?;
        let name_text = constructor_node.utf8_text(source.as_bytes()).ok()?;

        Some(Usage {
            name: Usage::normalize_line_endings(name_text),
            kind: UsageKind::CallExpression,
            position: Position::from_node(&constructor_node),
            context: None, // Match old implementation
            scope_id: Some(scope),
        })
    }

    fn extract_type_usage(&self, node: Node, scope: ScopeId, source: &str) -> Option<Usage> {
        let name_text = node.utf8_text(source.as_bytes()).ok()?;

        Some(Usage {
            name: Usage::normalize_line_endings(name_text),
            kind: UsageKind::TypeIdentifier,
            position: Position::from_node(&node),
            context: None, // Match old implementation
            scope_id: Some(scope),
        })
    }

    fn extract_property_identifier_usage(
        &self,
        node: Node,
        scope: ScopeId,
        source: &str,
    ) -> Option<Usage> {
        // Check if this property_identifier is in a definition context
        if let Some(parent) = node.parent() {
            match parent.kind() {
                "enum_body" => {
                    // Property identifiers in enum bodies are definitions, not usage
                    return None;
                }
                "public_field_definition" | "private_field_definition" | "field_definition" => {
                    // Property identifiers in field definitions are definitions, not usage
                    if let Some(name_field) = parent.child_by_field_name("name") {
                        if node.id() == name_field.id() {
                            return None;
                        }
                    }
                }
                "method_definition" => {
                    // Property identifiers in method definitions (method names) are definitions, not usage
                    if let Some(name_field) = parent.child_by_field_name("name") {
                        if node.id() == name_field.id() {
                            return None;
                        }
                    }
                }
                _ => {}
            }
        }

        // Property identifiers (like the "x" in "obj.x") should be treated as field expressions
        let name_text = node.utf8_text(source.as_bytes()).ok()?;

        Some(Usage {
            name: Usage::normalize_line_endings(name_text),
            kind: UsageKind::FieldExpression,
            position: Position::from_node(&node),
            context: None, // Match old implementation
            scope_id: Some(scope),
        })
    }

    fn is_type_identifier_in_definition_context(&self, node: Node) -> bool {
        // Check if this type_identifier is directly defining something (the name being defined)
        if let Some(parent) = node.parent() {
            match parent.kind() {
                "interface_declaration" | "type_alias_declaration" | "class_declaration" => {
                    // Check if this is the name field (being defined) or usage within the declaration
                    if let Some(name_field) = parent.child_by_field_name("name") {
                        return node.id() == name_field.id();
                    }
                }
                "type_parameter" => {
                    // Type parameters are definitions
                    if let Some(name_field) = parent.child_by_field_name("name") {
                        return node.id() == name_field.id();
                    }
                }
                _ => {}
            }
        }
        false
    }
}
