; Member accesses that write, and those that read before writing.
;
; `o.p = v` reaches a setter alone. `o.p += v` and `o.p++` read the old value first, so they reach
; both accessors of a pair. Everything not captured here reads.

(assignment_expression
  left: (member_expression property: (property_identifier) @written))

(augmented_assignment_expression
  left: (member_expression property: (property_identifier) @modified))

(update_expression
  argument: (member_expression property: (property_identifier) @modified))
