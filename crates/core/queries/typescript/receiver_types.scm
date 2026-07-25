; Bindings whose type is stated where they are declared, which is the only way a single file can
; know what `receiver.member` reaches. A binding with no annotation is left unknown rather than
; guessed at.
;
; The whole annotation is captured rather than a type name, because a union names several types and
; nests as it grows; the names are read off it in Rust.

(required_parameter
  pattern: (identifier) @binding
  type: (type_annotation) @annotated)

(optional_parameter
  pattern: (identifier) @binding
  type: (type_annotation) @annotated)

(variable_declarator
  name: (identifier) @binding
  type: (type_annotation) @annotated)
