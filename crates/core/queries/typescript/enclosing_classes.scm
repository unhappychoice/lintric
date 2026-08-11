; Where each class body begins and ends, so that a `this.member` access can be attributed to the
; class it sits inside.

(class_declaration
  name: (type_identifier) @owner
  body: (class_body) @body)

(abstract_class_declaration
  name: (type_identifier) @owner
  body: (class_body) @body)
