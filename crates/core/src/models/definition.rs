use super::position::Position;
use serde::{Deserialize, Serialize};
use std::fmt;

pub type ScopeId = usize;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Accessibility {
    Public,
    Private,
    ScopeLocal,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum DefinitionType {
    FunctionDefinition,
    VariableDefinition,
    StructDefinition,
    EnumDefinition,
    TypeDefinition,
    ModuleDefinition,
    ClassDefinition,
    InterfaceDefinition,
    ConstDefinition,
    MacroDefinition,
    MacroVariableDefinition,
    PropertyDefinition,
    MethodDefinition,
    StructFieldDefinition,
    ImportDefinition,
    Module,
    Other(String),
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct Definition {
    pub name: String,
    pub position: Position,
    pub definition_type: DefinitionType,
    pub scope_id: Option<ScopeId>,
    pub accessibility: Option<Accessibility>,
    pub is_hoisted: Option<bool>,
}

impl Definition {
    pub fn new(
        node: &tree_sitter::Node,
        source_code: &str,
        definition_type: DefinitionType,
    ) -> Self {
        Definition {
            name: node
                .utf8_text(source_code.as_bytes())
                .unwrap()
                .trim()
                .replace("\r\n", "\n"),
            position: Position::from_node(node),
            definition_type,
            scope_id: None,
            accessibility: None,
            is_hoisted: None,
        }
    }

    pub fn with_context(
        node: &tree_sitter::Node,
        source_code: &str,
        definition_type: DefinitionType,
        scope_id: ScopeId,
        accessibility: Accessibility,
        is_hoisted: bool,
    ) -> Self {
        Definition {
            name: node
                .utf8_text(source_code.as_bytes())
                .unwrap()
                .trim()
                .replace("\r\n", "\n"),
            position: Position::from_node(node),
            definition_type,
            scope_id: Some(scope_id),
            accessibility: Some(accessibility),
            is_hoisted: Some(is_hoisted),
        }
    }

    pub fn from_naming_node(
        node: &tree_sitter::Node,
        source_code: &str,
        definition_type: DefinitionType,
    ) -> Option<Self> {
        node.child_by_field_name("name")
            .map(|name_node| Definition::new(&name_node, source_code, definition_type))
    }

    pub fn from_naming_node_with_context(
        node: &tree_sitter::Node,
        source_code: &str,
        definition_type: DefinitionType,
        scope_id: ScopeId,
        accessibility: Accessibility,
        is_hoisted: bool,
    ) -> Option<Self> {
        node.child_by_field_name("name").map(|name_node| {
            Definition::with_context(
                &name_node,
                source_code,
                definition_type,
                scope_id,
                accessibility,
                is_hoisted,
            )
        })
    }

    pub fn line_number(&self) -> usize {
        self.position.line_number()
    }

    pub fn set_context(
        &mut self,
        scope_id: ScopeId,
        accessibility: &Accessibility,
        is_hoisted: bool,
    ) {
        self.scope_id = Some(scope_id);
        self.accessibility = Some(accessibility.clone());
        self.is_hoisted = Some(is_hoisted);
    }

    pub fn get_scope_id(&self) -> Option<ScopeId> {
        self.scope_id
    }

    pub fn get_accessibility(&self) -> Option<&Accessibility> {
        self.accessibility.as_ref()
    }

    pub fn is_hoisted(&self) -> Option<bool> {
        self.is_hoisted
    }

    pub fn new_simple_with_context(
        name: String,
        definition_type: DefinitionType,
        position: Position,
        scope_id: ScopeId,
        accessibility: Accessibility,
        is_hoisted: bool,
    ) -> Self {
        Self {
            name,
            definition_type,
            position,
            scope_id: Some(scope_id),
            accessibility: Some(accessibility),
            is_hoisted: Some(is_hoisted),
        }
    }
}

impl PartialOrd for Definition {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Definition {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        // Sort by position first
        self.position.cmp(&other.position)
    }
}

impl fmt::Debug for Definition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Definition {{ position: {:?}, name: {:?}, definition_type: {:?}, scope_id: {:?}, accessibility: {:?}, is_hoisted: {:?} }}", 
            self.position, self.name, self.definition_type, self.scope_id, self.accessibility, self.is_hoisted)
    }
}
