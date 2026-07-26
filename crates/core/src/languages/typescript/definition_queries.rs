use crate::models::DefinitionType;
use crate::query::{self, DeclaredAs, Roles};
use tree_sitter::Node;

/// Declarations located by query rather than by hand-written traversal.
const QUERY: &str = include_str!("../../../queries/typescript/definitions.scm");

/// What each capture in that file means, and whether it hoists.
const ROLES: [(&str, DeclaredAs); 11] = [
    (
        "definition.class",
        DeclaredAs::hoisted(DefinitionType::ClassDefinition),
    ),
    (
        "definition.interface",
        DeclaredAs::hoisted(DefinitionType::InterfaceDefinition),
    ),
    (
        "definition.type_alias",
        DeclaredAs::hoisted(DefinitionType::TypeDefinition),
    ),
    (
        "definition.enum",
        DeclaredAs::hoisted(DefinitionType::EnumDefinition),
    ),
    (
        "definition.namespace",
        DeclaredAs::hoisted(DefinitionType::ModuleDefinition),
    ),
    (
        "definition.function",
        DeclaredAs::hoisted(DefinitionType::FunctionDefinition),
    ),
    (
        "definition.method",
        DeclaredAs::plain(DefinitionType::MethodDefinition),
    ),
    (
        "definition.property",
        DeclaredAs::plain(DefinitionType::PropertyDefinition),
    ),
    (
        "definition.variable",
        DeclaredAs::plain(DefinitionType::VariableDefinition),
    ),
    (
        "definition.import",
        DeclaredAs::plain(DefinitionType::ImportDefinition),
    ),
    (
        "definition.type_parameter",
        DeclaredAs::plain(DefinitionType::TypeDefinition),
    ),
];

/// Run the declaration query over a file, giving what each captured name node declares.
pub fn declared_types(source_code: &str, root_node: Node) -> Result<Roles<DeclaredAs>, String> {
    query::capture_roles(QUERY, source_code, root_node, &ROLES)
}
