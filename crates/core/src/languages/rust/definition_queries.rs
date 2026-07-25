use crate::models::DefinitionType;
use crate::query::{self, Roles};
use tree_sitter::Node;

/// Declarations located by query rather than by hand-written traversal.
const QUERY: &str = include_str!("../../../queries/rust/definitions.scm");

/// What each capture in that file means.
const ROLES: [(&str, DefinitionType); 10] = [
    ("definition.struct", DefinitionType::StructDefinition),
    ("definition.enum", DefinitionType::EnumDefinition),
    (
        "definition.enum_variant",
        DefinitionType::EnumVariantDefinition,
    ),
    ("definition.type", DefinitionType::TypeDefinition),
    ("definition.module", DefinitionType::ModuleDefinition),
    ("definition.const", DefinitionType::ConstDefinition),
    ("definition.variable", DefinitionType::VariableDefinition),
    ("definition.function", DefinitionType::FunctionDefinition),
    ("definition.field", DefinitionType::StructFieldDefinition),
    ("definition.macro", DefinitionType::MacroDefinition),
];

/// Run the declaration query over a file, giving the kind of declaration each captured name node
/// introduces.
///
/// A malformed query is a bug in the file beside this one rather than something a caller can act
/// on, so an empty map is returned and analysis continues with whatever the extractor still handles
/// itself.
pub fn declared_types(source_code: &str, root_node: Node) -> Roles<DefinitionType> {
    query::capture_roles(QUERY, source_code, root_node, &ROLES).unwrap_or_default()
}
