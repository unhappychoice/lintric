use crate::dependency_resolver::trait_implementation::{self, Queries};
use crate::models::Dependency;
use tree_sitter::Node;

const QUERIES: Queries = Queries {
    implementations: r#"
        (class_declaration
          (class_heritage
            (implements_clause (type_identifier) @type))
          body: (class_body
            (method_definition name: (property_identifier) @method)))
    "#,
    declarations: r#"
        (interface_declaration
          name: (type_identifier) @type
          body: (interface_body
            (method_signature name: (property_identifier) @method)))
    "#,
};

/// Resolve dependencies from a TypeScript class method to the interface method it implements.
pub fn resolve(source_code: &str, root_node: Node) -> Result<Vec<Dependency>, String> {
    trait_implementation::resolve(&QUERIES, source_code, root_node)
}
