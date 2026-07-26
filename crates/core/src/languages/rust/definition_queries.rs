use crate::models::DefinitionType;
use crate::query::{self, DeclaredAs, Roles};
use tree_sitter::Node;

/// Declarations located by query rather than by hand-written traversal.
const QUERY: &str = include_str!("../../../queries/rust/definitions.scm");

/// What each capture in that file means.
const ROLES: [(&str, DeclaredAs); 11] = [
    (
        "definition.struct",
        DeclaredAs::plain(DefinitionType::StructDefinition),
    ),
    (
        "definition.enum",
        DeclaredAs::plain(DefinitionType::EnumDefinition),
    ),
    (
        "definition.enum_variant",
        DeclaredAs::plain(DefinitionType::EnumVariantDefinition),
    ),
    (
        "definition.type",
        DeclaredAs::plain(DefinitionType::TypeDefinition),
    ),
    (
        "definition.module",
        DeclaredAs::plain(DefinitionType::ModuleDefinition),
    ),
    (
        "definition.const",
        DeclaredAs::plain(DefinitionType::ConstDefinition),
    ),
    (
        "definition.variable",
        DeclaredAs::plain(DefinitionType::VariableDefinition),
    ),
    (
        "definition.function",
        DeclaredAs::plain(DefinitionType::FunctionDefinition),
    ),
    (
        "definition.field",
        DeclaredAs::plain(DefinitionType::StructFieldDefinition),
    ),
    (
        "definition.import",
        DeclaredAs::plain(DefinitionType::ImportDefinition),
    ),
    (
        "definition.macro",
        DeclaredAs::plain(DefinitionType::MacroDefinition),
    ),
];

/// Run the declaration query over a file, giving the kind of declaration each captured name node
/// introduces.
///
/// A malformed query is a bug in the file beside this one, and swallowing it would empty the
/// definition list rather than fail, so the error is returned.
pub fn declared_types(source_code: &str, root_node: Node) -> Result<Roles<DeclaredAs>, String> {
    query::capture_roles(QUERY, source_code, root_node, &ROLES)
}
