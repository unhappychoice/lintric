; What each `receiver.method()` call reads from, keyed by the position of the whole field expression,
; since that is the position the usage carries — Rust records a method call at the start of the
; receiver rather than at the method's own name.
;
; The receiver is captured whole rather than as a name: it may be `self`, a borrow, or a
; parenthesised expression, and what it is gets decided in Rust.

(call_expression
  function: (field_expression
    value: (_) @receiver
    field: (field_identifier)) @accessed)
