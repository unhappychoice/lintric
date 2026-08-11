; Identifiers that bind a name rather than reference one.
;
; The same node shape means different things by position: `x` in `let x = y` declares, `x` in
; `f(x)` reads. Everything captured here is a declaration occurrence, and the usage extractor takes
; anything else as a reference.
;
; @reference overrides @binding for the node it names, which is how the type a pattern matches
; against is kept out of the bindings that pattern introduces.

(let_declaration pattern: (identifier) @binding)
(parameter pattern: (identifier) @binding)
(for_expression pattern: (identifier) @binding)

; Every identifier a pattern names directly is bound by it.
(tuple_pattern (identifier) @binding)
(slice_pattern (identifier) @binding)
(reference_pattern (identifier) @binding)
(ref_pattern (identifier) @binding)
(closure_parameters (identifier) @binding)

; `Wrap(inner)` binds `inner` and references `Wrap`.
(tuple_struct_pattern (identifier) @binding)
(tuple_struct_pattern type: (identifier) @reference)

; Not a binding, but not counted here either: the call expression itself is extracted as the usage,
; so counting its callee again would record the same reference twice. A qualified callee such as
; `HashMap::new` is deliberately absent — its path components are references worth recording.
(call_expression function: (identifier) @call_target)

(type_parameter name: (type_identifier) @binding)
(const_parameter name: (identifier) @binding)
(extern_crate_declaration name: (identifier) @binding)
(lifetime (identifier) @binding)
(bounded_type (type_identifier) @binding)

; A declaration's own name is not a use of itself.
(function_item name: (identifier) @binding)
(function_signature_item name: (identifier) @binding)
(struct_item name: (type_identifier) @binding)
(union_item name: (type_identifier) @binding)
(enum_item name: (type_identifier) @binding)
(enum_variant name: (identifier) @binding)
(trait_item name: (type_identifier) @binding)
(mod_item name: (identifier) @binding)
(const_item name: (identifier) @binding)
(static_item name: (identifier) @binding)
(type_item name: (type_identifier) @binding)
(associated_type name: (type_identifier) @binding)
