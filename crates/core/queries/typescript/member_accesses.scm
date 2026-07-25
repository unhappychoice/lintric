; What each `receiver.member` reads from, keyed by the member's position, since a usage carries its
; position and only the member's own name.
;
; A chained access such as `a.b.c` is deliberately unmatched: its receiver is an expression rather
; than a name, so the file does not state its type.

(member_expression
  object: (identifier) @receiver
  property: (property_identifier) @accessed)

(member_expression
  object: (this) @receiver
  property: (property_identifier) @accessed)
