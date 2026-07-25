; What each `receiver.member` reads from, keyed by the member's position, since a usage carries its
; position and only the member's own name.
;
; The receiver is captured whole rather than as a name: it may be wearing a wrapper (`(a)`, `a!`) or
; state its own type (`a as First`), and what it is gets decided in Rust.

(member_expression
  object: (_) @receiver
  property: (property_identifier) @accessed)
