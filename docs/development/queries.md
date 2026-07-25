## Locating declarations with queries

Declarations whose shape alone identifies them are found by a tree-sitter query rather than by
hand-written traversal. The pattern lives in a `.scm` file, and Rust supplies only a table saying
what each capture means.

### Where things live

```
crates/core/queries/<language>/definitions.scm               the patterns
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

The extractor asks the query **first** and falls through to its arms only when nothing was captured.
That ordering matters: a `const_item`'s name is an `identifier`, and the `identifier` arm would
otherwise swallow it before the query was consulted.

### A broken query must fail loudly

`declared_types` returns a `Result`. An earlier version swallowed the error and returned no
captures, which the extractors read as "nothing declared here" — so one invalid node name emptied
the whole definition list instead of failing. That is how a `namespace_declaration` typo (the
grammar calls it `internal_module`) went unnoticed until the accuracy fixtures reported every
TypeScript declaration missing.

`crates/core/tests/unit/query/query_files_tests.rs` asserts each file compiles against its grammar,
including TSX, which is a separate grammar from TypeScript.

### Adding a language

1. Write `crates/core/queries/<language>/definitions.scm`
2. Write the mapping table beside its extractor
3. Call `declared_types` from the extractor's constructor and consult it first

The accuracy fixtures (`crates/accuracy`) are the check: migrating a kind should leave the recorded
numbers untouched, since the point is to move where a pattern is written rather than what it finds.
