; Identifiers that bind a name rather than reference one.
;
; The same node shape means different things by position: `x` in `const x = y` declares, `x` in
; `f(x)` reads. Everything captured as @binding is a declaration occurrence, and the usage extractor
; takes anything else as a reference.

; A declaration's own name is not a use of itself.
(function_declaration name: (identifier) @binding)
(generator_function_declaration name: (identifier) @binding)
(enum_declaration name: (identifier) @binding)
(internal_module name: (identifier) @binding)
(module name: (identifier) @binding)
(variable_declarator name: (identifier) @binding)
(import_specifier name: (identifier) @binding)

; An overload signature and its implementation are one function declared twice, not a reference
; from one to the other.
(function_signature name: (identifier) @binding)

; A function or class expression's name is bound inside its own body; it names nothing outside.
(function_expression name: (identifier) @binding)
(generator_function name: (identifier) @binding)
(class name: (type_identifier) @binding)

; `[key: string]: T` and `[K in keyof T]` each introduce the name they then use.
(index_signature name: (identifier) @binding)
(mapped_type_clause name: (type_identifier) @binding)

(interface_declaration name: (type_identifier) @binding)
(type_alias_declaration name: (type_identifier) @binding)
(class_declaration name: (type_identifier) @binding)
(abstract_class_declaration name: (type_identifier) @binding)
(type_parameter name: (type_identifier) @binding)

; `catch (e)` declares `e`, and `for (const x of xs)` declares `x`. A `for` without `kind:` assigns
; to a binding that already exists, so it reads rather than declares — which is why the field is
; required rather than the shape alone.
(catch_clause parameter: (identifier) @binding)
(for_in_statement kind: _ left: (identifier) @binding)

; Names a pattern or parameter list introduces.
(array_pattern (identifier) @binding)
(rest_pattern (identifier) @binding)
(assignment_pattern left: (identifier) @binding)
(required_parameter pattern: (identifier) @binding)
(optional_parameter pattern: (identifier) @binding)
(arrow_function parameter: (identifier) @binding)

; `{ key: renamed }` binds `renamed` and reads `key`.
(pair_pattern value: (identifier) @binding)

; A member declaration's own name. Left as a usage it resolved to any same-named member of another
; interface, and since that member's declaration did the same, the two invented a cycle.
(public_field_definition name: [(property_identifier) (private_property_identifier)] @binding)
(property_signature name: [(property_identifier) (private_property_identifier)] @binding)
(method_signature name: [(property_identifier) (private_property_identifier)] @binding)
(abstract_method_signature name: [(property_identifier) (private_property_identifier)] @binding)
(method_definition name: [(property_identifier) (private_property_identifier)] @binding)
(enum_body name: (property_identifier) @binding)
(enum_assignment name: [(property_identifier) (private_property_identifier)] @binding)

; An object literal's key, or a pattern's, references no declared member: TypeScript is structurally
; typed, so `{ x: 1 }` is a self-contained value, and the type it satisfies is usually declared in
; another file. See "Object shapes" in crates/accuracy/README.md.
(pair key: [(property_identifier) (private_property_identifier)] @shape_key)
(pair_pattern key: [(property_identifier) (private_property_identifier)] @shape_key)

; Not a binding, but not counted here either: the call expression itself is extracted as the usage,
; so counting its callee again would record the same reference twice.
(call_expression function: (identifier) @call_target)
