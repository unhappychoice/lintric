; The owner node is resolved in its own lexical scope, including local aliases.
(impl_item
  type: (_) @owner
  body: (declaration_list [
    (function_item name: (identifier) @member)
    (function_signature_item name: (identifier) @member)
    (const_item name: (identifier) @member)
    (type_item name: (type_identifier) @member)
  ]))

(trait_item
  name: (type_identifier) @owner
  body: (declaration_list [
    (function_item name: (identifier) @member)
    (function_signature_item name: (identifier) @member)
    (const_item name: (identifier) @member)
    (associated_type name: (type_identifier) @member)
  ]))

(enum_item
  name: (type_identifier) @owner
  body: (enum_variant_list (enum_variant name: (identifier) @member)))
