## Locating declarations with queries

Declarations whose shape alone identifies them are found by a tree-sitter query rather than by
hand-written traversal. The pattern lives in a `.scm` file, and Rust supplies only a table saying
what each capture means.

### Where things live

```
crates/core/queries/<language>/definitions.scm   the patterns
crates/core/src/languages/<language>/definition_queries.rs   capture name -> DefinitionType
crates/core/src/query/mod.rs                     runs a query, returns roles keyed by node
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

Then map the capture name in `definition_queries.rs`:

```rust
("definition.enum", DefinitionType::EnumDefinition),
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

### Adding a language

1. Write `crates/core/queries/<language>/definitions.scm`
2. Write the mapping table beside its extractor
3. Call `declared_types` from the extractor's constructor and consult it first

The accuracy fixtures (`crates/accuracy`) are the check: migrating a kind should leave the recorded
numbers untouched, since the point is to move where a pattern is written rather than what it finds.
