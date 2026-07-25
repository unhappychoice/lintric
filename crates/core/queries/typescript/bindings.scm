; Identifiers that bind a name rather than reference one.
;
; The same node shape means different things by position: `x` in `const x = y` declares, `x` in
; `f(x)` reads. Everything captured as @binding is a declaration occurrence, and the usage extractor
; takes anything else as a reference.

; A declaration's own name is not a use of itself.
(function_declaration name: (identifier) @binding)
(enum_declaration name: (identifier) @binding)
(internal_module name: (identifier) @binding)
(variable_declarator name: (identifier) @binding)
(import_specifier name: (identifier) @binding)

(interface_declaration name: (type_identifier) @binding)
(type_alias_declaration name: (type_identifier) @binding)
(class_declaration name: (type_identifier) @binding)
(abstract_class_declaration name: (type_identifier) @binding)
(type_parameter name: (type_identifier) @binding)

; Names a pattern or parameter list introduces.
(array_pattern (identifier) @binding)
(rest_pattern (identifier) @binding)
(assignment_pattern left: (identifier) @binding)
(required_parameter pattern: (identifier) @binding)
(optional_parameter pattern: (identifier) @binding)
(arrow_function parameter: (identifier) @binding)

; `{ key: renamed }` binds `renamed` and reads `key`.
(pair_pattern value: (identifier) @binding)

; Not a binding, but not counted here either: the call expression itself is extracted as the usage,
; so counting its callee again would record the same reference twice.
(call_expression function: (identifier) @call_target)
