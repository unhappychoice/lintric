; TypeScript declarations whose name is simply the `name:` field of the declaration.
;
; Each capture names what the declaration is; the mapping table beside this file turns that into a
; DefinitionType and says whether it hoists. Declarations needing more than the node's shape stay in
; the extractor: a `variable_declarator` hoists or not depending on its keyword, and a constructor
; parameter may declare a property.
;
; A namespace is an `internal_module` in this grammar; there is no `namespace_declaration` node.

(class_declaration name: (type_identifier) @definition.class)
(abstract_class_declaration name: (type_identifier) @definition.class)

(interface_declaration name: (type_identifier) @definition.interface)
(type_alias_declaration name: (type_identifier) @definition.type_alias)
(enum_declaration name: (identifier) @definition.enum)

(internal_module name: (identifier) @definition.namespace)

(function_declaration name: (identifier) @definition.function)
(generator_function_declaration name: (identifier) @definition.function)

(method_definition name: (property_identifier) @definition.method)
(method_definition name: (private_property_identifier) @definition.method)
(method_signature name: (property_identifier) @definition.method)
(abstract_method_signature name: (property_identifier) @definition.method)

(property_signature name: (property_identifier) @definition.property)
(public_field_definition name: (property_identifier) @definition.property)
(public_field_definition name: (private_property_identifier) @definition.property)

; `catch (e)` and `for (const x of xs)` each declare a local. A `for` without `kind:` assigns to a
; binding that already exists, so the field is required rather than the shape alone.
(catch_clause parameter: (identifier) @definition.variable)
(for_in_statement kind: _ left: (identifier) @definition.variable)

(import_specifier name: (identifier) @definition.import)

(type_parameter name: (type_identifier) @definition.type_parameter)
