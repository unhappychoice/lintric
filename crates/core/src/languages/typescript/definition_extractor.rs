use tree_sitter::Node;

use crate::models::{
    ast_traverser::NodeDefinitionExtractor, Definition, DefinitionType, Position, ScopeId,
    ScopeType, Usage,
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

    fn find_child_by_field_name<'a>(&self, node: Node<'a>, field_name: &str) -> Option<Node<'a>> {
        node.child_by_field_name(field_name)
    }
}
