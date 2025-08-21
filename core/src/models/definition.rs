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
    Other(String),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Definition {
    pub name: String,
    pub line_number: usize,
    pub definition_type: DefinitionType,
    pub scope: Option<String>,
}

impl Definition {
    pub fn new(
        node: &tree_sitter::Node,
        source_code: &str,
        definition_type: DefinitionType,
        scope: Option<String>,
    ) -> Self {
        Definition {
            name: node
                .utf8_text(source_code.as_bytes())
                .unwrap()
                .trim()
                .replace("\r\n", "\n"),
            line_number: node.start_position().row + 1,
            definition_type,
            scope,
        }
    }

    pub fn from_naming_node(
        node: &tree_sitter::Node,
        source_code: &str,
        definition_type: DefinitionType,
        scope: Option<String>,
    ) -> Option<Self> {
        node.child_by_field_name("name")
            .map(|name_node| Definition::new(&name_node, source_code, definition_type, scope))
    }
}
