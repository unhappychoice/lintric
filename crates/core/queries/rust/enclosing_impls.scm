; Where each impl or trait body begins and ends, so that a `self.method()` call can be attributed to
; the type it sits inside.

(impl_item
  type: [(type_identifier) (generic_type)] @owner
  body: (declaration_list) @body)

(trait_item
  name: (type_identifier) @owner
  body: (declaration_list) @body)
