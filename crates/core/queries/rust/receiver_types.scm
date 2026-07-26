; Bindings whose type is stated where they are declared, which is the only way a single file can know
; what `receiver.method()` reaches. A binding with no annotation is left unknown rather than guessed
; at.
;
; The whole type is captured rather than a name, because a reference or a generic wraps it.

(parameter
  pattern: (identifier) @binding
  type: (_) @annotated)

(let_declaration
  pattern: (identifier) @binding
  type: (_) @annotated)

; An initializer states the type as plainly as an annotation does. A unit struct arrives as an
; `identifier`, which the grammar cannot tell from reading a variable — and neither can this, so a
; name that turns out to be a variable simply matches no type.
(let_declaration
  pattern: (identifier) @binding
  value: (struct_expression name: (type_identifier) @annotated))

(let_declaration
  pattern: (identifier) @binding
  value: (identifier) @annotated)
