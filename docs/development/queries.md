## Locating declarations with queries

Declarations whose shape alone identifies them are found by a tree-sitter query rather than by
hand-written traversal. The pattern lives in a `.scm` file, and Rust supplies only a table saying
what each capture means.

### Where things live

```
crates/core/queries/<language>/definitions.scm               declaration patterns
crates/core/queries/<language>/scopes.scm                    scope patterns
crates/core/queries/<language>/bindings.scm                  which identifiers bind rather than read
crates/core/src/languages/<language>/definition_queries.rs   capture name -> what it declares
crates/core/src/query/mod.rs                                 runs a query, returns roles keyed by node
```

Query files are embedded with `include_str!`, so the binary stays self-contained and a malformed
query is caught by the tests rather than at a user's runtime.

### Adding a declaration kind

Capture the node that holds the **name**, not the item:

```scheme
(enum_item name: (type_identifier) @definition.enum)
```

The captured node's position becomes the declaration's position, which is what resolution matches
against, and the scope it lands in is the one the traverser is inside when it reaches that node.

Then map the capture name in `definition_queries.rs`, saying whether the declaration hoists —
visible before its own line, as a class is and a method is not:

```rust
("definition.enum", DeclaredAs::hoisted(DefinitionType::EnumDefinition)),
("definition.method", DeclaredAs::plain(DefinitionType::MethodDefinition)),
```

A capture name absent from the table is ignored, so a query may capture nodes for other purposes.

### What stays in the extractor

A query describes shape. When classifying needs more than shape, the extractor keeps the arm:

- `function_item` is a `MethodDefinition` inside an `impl` and a `FunctionDefinition` outside it,
  which the query cannot see
- `let` patterns need their bindings told apart from the type they match, and a path component from
  a binding
- format string captures do not exist as nodes at all and are parsed out of the literal
- "sits anywhere inside a `use` tree" is an ancestor at any depth, and enumerating the depths a path
  can nest to would be worse than the walk it replaces

The extractor asks the query **first** and falls through to its arms only when nothing was captured.
That ordering matters: a `const_item`'s name is an `identifier`, and the `identifier` arm would
otherwise swallow it before the query was consulted.

### Bindings

The same node shape means different things by position: `x` in `let x = y` declares a name, `x` in
`f(x)` reads one. `bindings.scm` captures the declaring occurrences, and the usage extractor takes
everything else as a reference — replacing a walk up the parent chain with a set lookup.

One case a query cannot state directly is "every child of this pattern except the type it matches
against", since there is no way to exclude a field. So the type is captured as `@reference` and a
reference wins over a binding for the same node:

```scheme
(tuple_struct_pattern (identifier) @binding)
(tuple_struct_pattern type: (identifier) @reference)
```

TypeScript's file also carries `@call_target`, which is not a binding: a call expression is itself
extracted as the usage, so counting its callee again would record the same reference twice.

Writing the patterns out is what showed which hand-written arms could never fire. In Rust,
`constrained_type_parameter` is not a node in the grammar at all, and `where_clause` and
`type_parameters` have no identifier among their children. In TypeScript, `namespace_declaration` is
the same non-existent node #222 found in `definitions.scm`, `shorthand_property_identifier_pattern`
is a leaf so nothing has it as a parent, `object_pattern` has no bare identifier child, and eight
arms named declarations whose `name:` is a `type_identifier` or `property_identifier` — never the
`identifier` the function was consulted for. All are gone rather than transcribed.

Checking a field against `node-types.json` before writing the pattern is worth the minute it takes:
a field name the node does not have fails to compile, and a node type that cannot appear there
compiles into a pattern that silently never matches.

### Scopes

`scopes.scm` captures the whole node rather than a name, since a scope spans the item:

```scheme
(function_item) @scope.function
(block) @scope.block
```

Writing these out is what exposed #223: the TypeScript extractor matched `block`, a node this
grammar does not have — its blocks are `statement_block` — so blocks had never scoped. The query
preserves that behaviour and says so in a comment rather than fixing it in passing, because adding
the node changes what is found rather than where the pattern lives.

### Queries that answer a relationship

Not every query labels a node. Some answer a question about a pair of nodes — which type declares a
member, which type a binding is annotated with — and those capture both halves:

```scheme
(interface_declaration
  name: (type_identifier) @owner
  body: (interface_body (property_signature name: (property_identifier) @member)))
```

`query::text_by_position` and `query::text_by_span` turn such a match into the shape the caller
needs: one capture's text keyed by the other's position, or paired with the other's line span. When
the caller needs the nodes themselves — reading the type names out of a union annotation, which nests as it
grows — `query::map_pairs` hands them over inside the match loop, since a captured node's lifetime
ends with the query cursor.

Position is how a captured node is matched back to a `Definition`, which carries a position rather
than a node; both count lines and columns from one.

`trait_implementation.rs` and `receiver_narrowing.rs` are both built this way.

### A broken query must fail loudly

`declared_types` returns a `Result`. An earlier version swallowed the error and returned no
captures, which the extractors read as "nothing declared here" — so one invalid node name emptied
the whole definition list instead of failing. That is how a `namespace_declaration` typo (the
grammar calls it `internal_module`) went unnoticed until the accuracy fixtures reported every
TypeScript declaration missing.

`crates/core/tests/unit/query/query_files_tests.rs` asserts each file compiles against its grammar,
including TSX, which is a separate grammar from TypeScript.

A hand-written `match node.kind()` has no such check: an arm naming a node the grammar does not have
compiles, runs, and never matches. Checking the arms against `node-types.json` is what found
`constrained_type_parameter`, `field_definition` and `private_field_definition` — three arms and the
function behind one of them, all unreachable. It is worth re-running after a grammar upgrade, since
that is how such an arm becomes dead in the first place.

### Sweeping the grammar for declarations

The same file answers the opposite question — which declarations the queries do **not** cover. List
every node with a field that can hold an identifier, and subtract what the query files capture:

| Field swept | Found |
| --- | --- |
| `name:` | #233 (nine TypeScript forms), #234 (`const_parameter`) |
| `parameter:`, `left:`, `pattern:` | #241 (`catch_clause`, `for_in_statement`) |

#241 is why the field list matters: the first sweep looked only at `name:`, and a catch parameter is
`parameter:`. Sweep the fields a declaration can hang off, not one of them.

What comes back is a list of candidates, not of bugs — most entries are references and correctly
uncovered. `where_predicate left:`, `range_pattern left:`, `type_binding name:`, `type_predicate
name:`, `export_specifier name:` and the `jsx_*_element` names were all checked and were already
right. Probe each candidate before changing anything.

Two traps worth knowing:

- **A field can be the deciding factor.** `for (const x of xs)` declares `x` while `for (x of xs)`
  assigns to an existing binding, and the two are identical in `left:`. Requiring the field that
  distinguishes them is what a query can say: `(for_in_statement kind: _ left: (identifier))`.
- **The same shape can be a declaration or a reference depending on what else is in the file.** A
  bare identifier in a Rust match arm binds a name unless it names a variant or a constant, and no
  query can tell. Capturing it as a binding loses those references — see #239, which records the
  attempt and why it was reverted.

### Adding a language

1. Write `crates/core/queries/<language>/definitions.scm`
2. Write the mapping table beside its extractor
3. Call `declared_types` from the extractor's constructor and consult it first

The accuracy fixtures (`crates/accuracy`) are the check: migrating a kind should leave the recorded
numbers untouched, since the point is to move where a pattern is written rather than what it finds.
