; Nodes that introduce a scope, and what kind of scope each introduces.
;
; The whole node is captured rather than a name, since a scope spans the declaration.

(function_declaration) @scope.function
(method_definition) @scope.function
(arrow_function) @scope.function

(class_declaration) @scope.class
(abstract_class_declaration) @scope.class

(interface_declaration) @scope.interface

; A namespace is an `internal_module` in this grammar.
(internal_module) @scope.module

; `const` and `let` are block-scoped, so a block is a scope. The extractor used to match `block`,
; which this grammar does not have — its blocks are `statement_block`.
(statement_block) @scope.block
(for_statement) @scope.block
(while_statement) @scope.block
(if_statement) @scope.block
