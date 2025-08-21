use super::position::Position;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
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
    ImportDefinition,
    Other(String),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Definition {
    pub name: String,
    pub position: Position,
    pub definition_type: DefinitionType,
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

    pub fn line_number(&self) -> usize {
        self.position.line_number()
    }
}
