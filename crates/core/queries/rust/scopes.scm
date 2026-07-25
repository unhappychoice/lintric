; Nodes that introduce a scope, and what kind of scope each introduces.
;
; The whole node is captured rather than a name, since a scope spans the item.

(function_item) @scope.function
(closure_expression) @scope.closure

(impl_item) @scope.impl
(trait_item) @scope.trait
(mod_item) @scope.module

; A struct, union or enum scopes its own members.
(struct_item) @scope.block
(union_item) @scope.block
(enum_item) @scope.block

(block) @scope.block
(for_expression) @scope.block
(while_expression) @scope.block
(if_expression) @scope.block
(match_expression) @scope.block
