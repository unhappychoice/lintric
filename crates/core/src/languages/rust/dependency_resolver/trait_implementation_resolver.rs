use crate::dependency_resolver::trait_implementation::{self, Queries};
use crate::models::Dependency;
use tree_sitter::Node;

const QUERIES: Queries = Queries {
    implementations: r#"
        (impl_item
          trait: (type_identifier) @type
          body: (declaration_list
            (function_item name: (identifier) @method)))
    "#,
    // An implementation can satisfy a required signature or override a method the trait already
    // provides a body for, so both count as declarations.
    declarations: r#"
        (trait_item
          name: (type_identifier) @type
          body: (declaration_list [
            (function_signature_item name: (identifier) @method)
            (function_item name: (identifier) @method)
          ]))
    "#,
};

/// Resolve dependencies from a Rust trait method implementation to the declaration it satisfies.
pub fn resolve(source_code: &str, root_node: Node) -> Result<Vec<Dependency>, String> {
    trait_implementation::resolve(&QUERIES, source_code, root_node)
}
