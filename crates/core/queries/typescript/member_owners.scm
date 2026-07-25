; Which type declares a member, so that `receiver.member` can be pointed at the one member the
; receiver's type owns rather than at every member sharing that name.
;
; The member's name node is captured because its position is what a Definition carries, and the
; owner's name is captured as text.

(interface_declaration
  name: (type_identifier) @owner
  body: (interface_body [
    (property_signature name: (property_identifier) @member)
    (method_signature name: (property_identifier) @member)
  ]))

(class_declaration
  name: (type_identifier) @owner
  body: (class_body [
    (public_field_definition name: (property_identifier) @member)
    (method_definition name: (property_identifier) @member)
  ]))

(abstract_class_declaration
  name: (type_identifier) @owner
  body: (class_body [
    (public_field_definition name: (property_identifier) @member)
    (method_definition name: (property_identifier) @member)
    (abstract_method_signature name: (property_identifier) @member)
  ]))

(type_alias_declaration
  name: (type_identifier) @owner
  value: (object_type [
    (property_signature name: (property_identifier) @member)
    (method_signature name: (property_identifier) @member)
  ]))
