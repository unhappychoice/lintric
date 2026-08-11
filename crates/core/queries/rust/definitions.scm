; Rust declarations whose name is simply the `name:` field of the item.
;
; Each capture names what the declaration is; the mapping table beside this file turns that into a
; DefinitionType. Declarations needing more than the node's shape to classify — a `function_item`,
; which is a method inside an impl and a function outside — stay in the extractor.

(struct_item name: (type_identifier) @definition.struct)
(union_item name: (type_identifier) @definition.struct)

(enum_item name: (type_identifier) @definition.enum)
(enum_variant name: (identifier) @definition.enum_variant)

(trait_item name: (type_identifier) @definition.type)
(type_item name: (type_identifier) @definition.type)
(associated_type name: (type_identifier) @definition.type)
(type_parameter name: (type_identifier) @definition.type)
; A const generic declares a value, not a type, though it is written among the type parameters.
(const_parameter name: (identifier) @definition.const)

(mod_item name: (identifier) @definition.module)

; Every local name a `use` introduces: the last segment of a path, an item in a braced list, or an
; alias. A wildcard introduces no name of its own, so it is absent.
(use_declaration (scoped_identifier name: (identifier) @definition.import))
(use_list (identifier) @definition.import)
(use_list (scoped_identifier name: (identifier) @definition.import))
(use_as_clause alias: (identifier) @definition.import)

(const_item name: (identifier) @definition.const)
(static_item name: (identifier) @definition.variable)

(function_signature_item name: (identifier) @definition.function)

(field_declaration name: (field_identifier) @definition.field)

(macro_definition name: (identifier) @definition.macro)
