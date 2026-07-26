use tree_sitter::Node;

use crate::models::{
    ast_traverser::{NodeDefinitionExtractor, NodeUsageExtractor},
    Definition, DefinitionType, Position, ScopeId, ScopeType, Usage, UsageKind,
};
use crate::query::{DeclaredAs, Roles};

/// TypeScript-specific definition extractor
///
/// Declarations whose shape alone identifies them are located by
/// `queries/typescript/definitions.scm`; the arms below handle the rest, where classifying needs
/// more than the node — a `variable_declarator` hoists or not depending on its keyword, and a
/// constructor parameter may declare a property.
pub struct TypeScriptDefinitionExtractor {
    declared_types: Roles<DeclaredAs>,
    scope_kinds: Roles<ScopeType>,
}

impl TypeScriptDefinitionExtractor {
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
            name: Usage::normalize_line_endings(name),
            definition_type: declared.definition_type.clone(),
            position: Position::from_node(&node),
            scope_id: Some(scope),
            accessibility: None,
            is_hoisted: Some(declared.is_hoisted),
        }]
    }
}

impl NodeDefinitionExtractor for TypeScriptDefinitionExtractor {
    fn extract_definition(&self, node: Node, scope: ScopeId, source: &str) -> Vec<Definition> {
        // The query is the primary source; the arms below are the exceptions it cannot express.
        // Asking it first also keeps a name node from being swallowed by a kind arm.
        let queried = self.queried_definition(node, scope, source);
        if !queried.is_empty() {
            return queried;
        }

        match node.kind() {
            "arrow_function" => self.extract_arrow_function_definition(node, scope, source),
            "variable_declarator" => self.extract_variable_definition(node, scope, source),
            "formal_parameters" => self.extract_function_parameters(node, scope, source),
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
        self.scope_kinds
            .get(&node.id())
            .map(|scope_type| (scope_type.clone(), Position::from_node(&node)))
    }
}

impl TypeScriptDefinitionExtractor {
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
        let in_constructor = self.is_constructor_parameter_list(node, source);
        let mut definitions = vec![];
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            match child.kind() {
                "required_parameter" | "optional_parameter" => {
                    let definition_type = match in_constructor && self.declares_property(child) {
                        true => DefinitionType::PropertyDefinition,
                        false => DefinitionType::VariableDefinition,
                    };

                    if let Some(pattern_node) = child.child_by_field_name("pattern") {
                        // Find identifiers in the pattern node
                        let identifiers = self.find_identifier_nodes_in_node(pattern_node);
                        for identifier_node in identifiers {
                            let mut def =
                                Definition::new(&identifier_node, source, definition_type.clone());
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

    /// Only a constructor can declare properties through its parameters.
    fn is_constructor_parameter_list(&self, node: Node, source: &str) -> bool {
        node.parent()
            .filter(|parent| parent.kind() == "method_definition")
            .and_then(|method| method.child_by_field_name("name"))
            .and_then(|name| name.utf8_text(source.as_bytes()).ok())
            .is_some_and(|name| name == "constructor")
    }

    /// `constructor(public value: number)` declares a class property as well as a parameter.
    ///
    /// An accessibility modifier is a named node, whereas `readonly` is an anonymous token, so
    /// both named and unnamed children have to be considered.
    fn declares_property(&self, parameter: Node) -> bool {
        let mut cursor = parameter.walk();
        let has_modifier = parameter
            .children(&mut cursor)
            .any(|child| matches!(child.kind(), "accessibility_modifier" | "readonly"));

        has_modifier
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
pub struct TypeScriptUsageExtractor {
    roles: super::binding_queries::Roles,
}

impl TypeScriptUsageExtractor {
    /// Fails if the binding query does not compile, which is a bug in the `.scm` file rather than
    /// anything about the source being analyzed.
    pub fn new(source_code: &str, root_node: Node) -> Result<Self, String> {
        Ok(Self {
            roles: super::binding_queries::roles(source_code, root_node)?,
        })
    }
}

impl NodeUsageExtractor for TypeScriptUsageExtractor {
    fn extract_usage(&self, node: Node, scope: ScopeId, source: &str) -> Vec<Usage> {
        let usage = match node.kind() {
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
            // A `#name` member is the same thing under a different node kind, and the definition
            // positions it can occupy are already covered
            "property_identifier" | "private_property_identifier" => {
                self.extract_property_identifier_usage(node, scope, source)
            }
            // `{ x }` is shorthand for `{ x: x }`, so it reads the binding. The pattern form is a
            // different node kind and stays a binding.
            "shorthand_property_identifier" => self.extract_identifier_usage(node, scope, source),
            _ => None,
        };

        usage.into_iter().collect()
    }
}

impl TypeScriptUsageExtractor {
    /// Whether this identifier reads a name, as opposed to declaring one or naming the callee of a
    /// call expression that is already recorded as the usage.
    ///
    /// The patterns live in `queries/typescript/bindings.scm`.
    fn is_usage_context(&self, node: Node) -> bool {
        self.roles.reads(node)
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
        let function_node = node.child_by_field_name("function")?;
        if function_node.kind() != "identifier" {
            return None;
        }
        let function_name =
            Usage::normalize_line_endings(function_node.utf8_text(source.as_bytes()).ok()?);

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

    /// A member read through a receiver, unless this occurrence declares the member instead.
    ///
    /// Which occurrences declare rather than read is stated in `queries/typescript/bindings.scm`.
    fn extract_property_identifier_usage(
        &self,
        node: Node,
        scope: ScopeId,
        source: &str,
    ) -> Option<Usage> {
        if !self.roles.reads(node) {
            return None;
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

    /// Whether this type identifier is the name a declaration introduces.
    fn is_type_identifier_in_definition_context(&self, node: Node) -> bool {
        self.roles.declares(node)
    }
}
