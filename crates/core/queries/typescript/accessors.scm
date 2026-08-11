; Getters and setters, which declare one member in two places.
;
; Both are `method_definition`s of the same name, so only the keyword tells them apart — and which
; one an access reaches depends on whether it reads or writes.

(method_definition "get" name: (property_identifier) @getter)
(method_definition "set" name: (property_identifier) @setter)

; An interface declares them the same way, and a class getter satisfies the interface's getter rather
; than its setter.
(method_signature "get" name: (property_identifier) @getter)
(method_signature "set" name: (property_identifier) @setter)
