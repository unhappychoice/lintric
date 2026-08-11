; Each `let` paired with its initializer, so that the binding can be kept out of the names its own
; initializer reads.
;
; `let w = w + 1` reads the previous `w`, never the one being declared — the binding starts after the
; statement. A pattern binding several names is matched once per name.

(let_declaration
  pattern: (identifier) @declared
  value: (_) @initializer)
