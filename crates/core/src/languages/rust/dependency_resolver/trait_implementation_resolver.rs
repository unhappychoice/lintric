use crate::dependency_resolver::trait_implementation::{self, Queries};
use crate::models::Dependency;
use tree_sitter::Node;

const QUERIES: Queries = Queries {
    // A trait declares consts and types alongside methods, and an implementation is coupled to each
    // of them the same way, so all three are captured under one name.
    implementations: r#"
        (impl_item
          trait: [(type_identifier) (generic_type)] @type
          body: (declaration_list [
            (function_item name: (identifier) @method)
            (const_item name: (identifier) @method)
            (type_item name: (type_identifier) @method)
          ]))
    "#,
    // An implementation can satisfy a required signature or override a method the trait already
    // provides a body for, so both count as declarations.
    declarations: r#"
        (trait_item
          name: (type_identifier) @type
          body: (declaration_list [
            (function_signature_item name: (identifier) @method)
            (function_item name: (identifier) @method)
            (const_item name: (identifier) @method)
            (associated_type name: (type_identifier) @method)
          ]))
    "#,
    // Rust has no accessors: a getter and a setter would be two ordinary methods with different
    // names.
    accessors: None,
    // `trait Extended: Base` inherits Base's declarations, so an implementation of Extended can be
    // satisfying something Base declared.
    supertypes: r#"
        (trait_item
          name: (type_identifier) @type
          bounds: (trait_bounds [(type_identifier) (generic_type)] @super))
    "#,
};

/// Resolve dependencies from a Rust trait method implementation to the declaration it satisfies.
pub fn resolve(source_code: &str, root_node: Node) -> Result<Vec<Dependency>, String> {
    trait_implementation::resolve(&QUERIES, source_code, root_node)
}
