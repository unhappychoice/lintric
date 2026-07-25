use crate::dependency_resolver::trait_implementation::{self, Queries};
use crate::models::Dependency;
use tree_sitter::Node;

const QUERIES: Queries = Queries {
    // A class method can satisfy an interface it implements or override one from the class it
    // extends, so both heritage clauses name a type whose declarations it may be satisfying.
    implementations: r#"
        (class_declaration
          (class_heritage [
            (implements_clause (type_identifier) @type)
            (extends_clause value: (identifier) @type)
          ])
          body: (class_body
            (method_definition name: (property_identifier) @method)))
    "#,
    // A base class declares by providing a body, so its methods count as declarations alongside
    // the signatures an interface declares.
    declarations: r#"
        [
          (interface_declaration
            name: (type_identifier) @type
            body: (interface_body
              (method_signature name: (property_identifier) @method)))
          (class_declaration
            name: (type_identifier) @type
            body: (class_body
              (method_definition name: (property_identifier) @method)))
        ]
    "#,
    // `interface Extended extends Base` and `class Derived extends Base` both inherit
    // declarations, so a lookup that misses the named type continues into these.
    supertypes: r#"
        [
          (interface_declaration
            name: (type_identifier) @type
            (extends_type_clause type: (type_identifier) @super))
          (class_declaration
            name: (type_identifier) @type
            (class_heritage [
              (extends_clause value: (identifier) @super)
              (implements_clause (type_identifier) @super)
            ]))
        ]
    "#,
};

/// Resolve dependencies from a TypeScript class method to the interface method it implements.
pub fn resolve(source_code: &str, root_node: Node) -> Result<Vec<Dependency>, String> {
    trait_implementation::resolve(&QUERIES, source_code, root_node)
}
