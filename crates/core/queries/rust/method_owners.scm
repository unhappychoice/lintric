; Which type declares a method, so that `receiver.method()` can be pointed at the one the receiver's
; type declares rather than at every method sharing that name.
;
; The method's name node is captured because its position is what a Definition carries. A trait
; declares as well: a blanket impl's `self.name()` names the trait's method, not any implementor's.

(impl_item
  type: [(type_identifier) (generic_type)] @owner
  body: (declaration_list [
    (function_item name: (identifier) @member)
    (function_signature_item name: (identifier) @member)
  ]))

(trait_item
  name: (type_identifier) @owner
  body: (declaration_list [
    (function_item name: (identifier) @member)
    (function_signature_item name: (identifier) @member)
  ]))
