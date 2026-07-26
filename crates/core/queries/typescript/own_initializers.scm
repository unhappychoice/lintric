; Each declarator paired with its initializer, so that the binding can be kept out of the names its
; own initializer reads.
;
; `let x = x + 1` reads the previous `x`, never the one being declared.

(variable_declarator
  name: (identifier) @declared
  value: (_) @initializer)
