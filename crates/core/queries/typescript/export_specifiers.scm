; Names an export clause exposes.
;
; `export { X }` names a local declaration whether that declaration is a type or a value, and
; `export type { X }` reads as an ordinary identifier — so an export is the one position where the
; two namespaces are not kept apart.

(export_specifier name: (identifier) @exported)
