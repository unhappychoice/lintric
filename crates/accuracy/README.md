# Lintric Accuracy

Measures how accurately Lintric detects line-level dependencies, and records the result as a
baseline so that changes in detection behaviour are visible in diffs.

## Why this exists

This crate does **not** replace the snapshot suite, and covers far less ground than it does. The
352 snapshots across `crates/core`, `crates/cli` and `crates/test-generator` exercise a much wider
range of constructs than the fixtures here, and they remain the right tool for detecting that
analysis output changed at all — a behaviour-preserving refactor should leave every one of them
untouched.

What they cannot answer is "how accurate are we?". A snapshot records whatever the analyzer
currently outputs, so wrong output is locked in as faithfully as right output. Every defect fixed
in #171, #172, #175, #176 and #177 sat inside green snapshots as an accepted expected value — a
trait method declaration recorded as a usage, `dependent_lines: [1, 1, 1]` for a line depending on
one line. Nothing was failing.

This crate takes the opposite approach: expectations are written by hand from the language's
semantics, **independent of what the analyzer currently produces**. That is what makes precision
and recall meaningful, and what lets a diff say whether a change was an improvement rather than
just a change.

Use both. Snapshots tell you something moved; these fixtures tell you which direction.

## Running it

```bash
cargo run -p lintric-accuracy              # print the report
cargo run -p lintric-accuracy -- --check   # exit non-zero if it differs from the baseline
cargo run -p lintric-accuracy -- --update  # record the current numbers as the baseline
```

`cargo test -p lintric-accuracy` gates the same comparison, so a change in detection behaviour
fails the suite until the baseline is deliberately updated.

## Expectation format

Fixtures live under `fixtures/<language>/` and carry their expectations inline:

```rust
fn main() {
    let a = 1;
    let b = a + 2; //~ depends: a@2
    let c = a + b; //~ depends: a@2, b@3
}
```

An annotation is `//~ depends: <symbol>@<line>[, <symbol>@<line>]*` and applies to **the line it
appears on**. Because the annotation attaches to its own line, an expectation can be placed on
any line of a multi-line statement.

Rules:

- **Fixtures are annotated exhaustively.** Any detected edge that is not annotated counts as
  spurious. There is no way to leave an edge unstated but tolerated — that is the point.
- A malformed annotation is an error, not a skipped line. A typo cannot silently weaken the
  expected set.
- An edge is identified by `(source line, target line, symbol)`, so a wrong symbol on the right
  lines is counted as one missing edge plus one spurious edge.
- Dependencies on symbols defined outside the fixture (`println`, `Send`, `JSX`) have no target
  line and therefore produce no edge.

## Reported numbers

| Column | Meaning |
| --- | --- |
| Expected | Hand-written edges |
| Detected | Distinct edges the analyzer reported |
| Correct | Detected edges that were expected |
| Missing | Expected edges not detected |
| Spurious | Detected edges not expected |
| Duplicates | Edges the IR reported more than once, before the graph collapses them |
| Precision | Correct / Detected |
| Recall | Correct / Expected |

Duplicates count edges the IR reports more than once, and are kept out of precision rather than
folded into it. They are **not** a defect: the IR deliberately keeps every occurrence because they
are useful when inspecting it, and the metric calculator collapses them when building the
line-to-line graph, so `self.x * self.x` depends on one line rather than two. The column measures
how often the fixtures reference the same target twice, which is a property of the fixtures.

Precision and recall are derived when reporting rather than stored, so `baseline.json` holds
only integer counts and does not churn on float formatting.

## Interpreting the baseline

The recorded numbers describe **these fixtures only**, and are not an accuracy claim for the
analyzer as a whole. Constructs the fixtures do not reach are simply unmeasured.

A high score is therefore a signal to add fixtures, not a sign that detection is finished — and
the history here is unambiguous on that point. Every time the fixture set has grown, it has found
defects that a green harness had been reporting as perfect:

| Fixtures added | Score before | Score after | Found |
| --- | --- | --- | --- |
| `nested_scopes` | 1.000 / 1.000 | 0.994 / 0.988 | #187, the first spurious edge ever recorded |
| `supertraits`, `inheritance` | 1.000 / 1.000 | — | #191, which the harness could not see until then |
| 11 more across both languages | 1.000 / 1.000 | 0.979 / 0.956 | #194, #195, #196, #197, #198, #199 |

So the numbers being at 1.000 has twice meant "the fixtures do not reach the bug" rather than
"there is no bug". Treat any perfect score as an invitation to write the fixture that breaks it.

The same growth also corrected assumptions recorded here earlier: that the analyzer only
under-reports (#187 disproved it), and that it makes no false positives (#197 invents edges from
nothing more than a shared field name).

### Editing a fixture's header moves every annotation below it

Adding a sentence to the comment at the top of a fixture shifts every line under it, and every
`@line` written before that edit now points one short. The harness reports a wall of missing and
spurious edges, which reads like a broken analyzer.

It says so instead:

```
  every missing edge is a spurious one 1 line later: the annotations were written against a
  different numbering, so check whether a line was added or removed above them
```

The signature is that each missing edge has a spurious twin naming the same symbol from the same
source line, all at one offset. Where the offsets disagree, or only some edges have a twin, nothing
is claimed — two unrelated defects are not a shift, and explaining them away would hide them.

When it fires, strip every annotation and re-derive them against the new numbering rather than
adjusting them one at a time.

Writing expectations is not free of error either. Several apparent defects turned out to be wrong
annotations — a JSX attribute does reference the prop it names, a type parameter reference is real,
`super::X` does not depend on the module's declaration. When a fixture disagrees with the analyzer,
check which one is wrong before filing.

## Auditing real code for self-consistency

The fixtures measure correctness on cases written by hand, which is why they are small. A
complementary check runs the analyzer over the repository's own source and asserts something weaker
but at scale: **for every edge, the symbol appears as a word on both the source and the target line.**

Over 7,990 edges from `crates/**/*.rs`, target lines matched in every case, and every source-line
mismatch fell into one of two documented behaviours:

| Count | Why the source line does not contain the symbol |
| --- | --- |
| 206 | `Self` is rewritten to the type it stands for, so the line says `Self` while the edge names the type |
| 48 | a multi-line expression is recorded at its start, so `let x = thing` carries an edge for a method called two lines further down |

Nothing else. The check finds a different class of defect from the fixtures — a resolution pointing
at a line that does not mention the name at all cannot survive it, however exotic the construct — and
it costs a couple of minutes to run over a few thousand edges rather than a few hundred expectations.

It is a sweep to re-run after changing resolution, not a test: it is too slow for CI, and it proves
consistency rather than correctness. An edge can name the right symbol on the wrong line, which is
what the fixtures are for.

## Import resolution

Given

```rust
mod geometry {
    pub struct Rect { pub width: i32 }
}
use geometry::Rect;          // L4

fn main() {
    let r = Rect { width: 1 };
}
```

`Rect` in `main` depends on the **`use` line**, and the `use` line depends on the struct
definition. The chain, rather than a direct edge to the definition, is deliberate:

- a `use` statement creates a local binding, and that binding is what the reference names
- both edges are real: changing the `use` line breaks the reference, and renaming the struct
  breaks the `use` line
- transitive metrics recover the full chain anyway, so nothing is lost by going through it
- it agrees with #87's proposal that import-like dependencies belong at the top scope level

A `use` line names the declaration and creates a binding of it at one position, so the name refers
to the declaration while the binding is what it produces. Without that distinction the line resolved
to its own import and produced no edge at all.

This was an open question when the harness was written, and `rust/imports.rs` now pins the
answer. It is a decision rather than a fact about the language, so it is revisitable — but it
should be revisited by changing that fixture, not by discovering that behaviour drifted.

## Object shapes

A TypeScript object literal's field name, and a destructuring pattern's, do **not** reference a
declared member:

```typescript
const origin: Point = { x: 0, y: 0 };   // depends on Point, not on Point.x
function shift({ x, y }: Point) { .. }  // same
```

The reasoning, in order of weight:

- TypeScript is structurally typed, so `{ x: 0 }` is a self-contained value rather than a reference
  to some `x` declared elsewhere. Unlike Rust's `Point { x: 0 }`, the literal does not name a type.
- **Single-file analysis cannot see the type anyway.** The interface a literal satisfies is usually
  declared in another file, so any same-file member of a matching name is more likely a coincidence
  than the real referent.
- The coupling to a declared type *is* recorded, through the annotation that names it.
- Matching by name alone was inventing edges: a literal in a function with no relation to an
  interface linked to it whenever a field name happened to coincide, and names like `id`, `name` and
  `value` recur across interfaces in any real codebase.

This has a real cost, which is worth stating rather than glossing: `const obj: MyInterface = { field: 10 }`
genuinely couples that line to `MyInterface.field`, and that edge is now gone. Recovering it would
mean resolving the annotation, the enclosing function's return type, or a callee's parameter type —
which is type resolution, and out of reach today. A missing edge understates coupling; an invented
one sends a reader to unrelated code.

Ordinary member access, `origin.x`, is untouched and still resolves — see below for which `x`.

## Member access and the receiver's type

`receiver.member` reaches what the receiver's **type** declares, so two types declaring the same
member name are told apart by what the receiver is:

```typescript
interface Reader { label: string; }   // L1
class Writer { label: number; }       // L2

function f(reader: Reader) {
    return reader.label;              // depends on L1's label, not L2's
}
```

The receiver's type is taken from where the file states it: a parameter or variable annotation, or
`this` inside a class. A union annotation names several types, and the member may be declared by any
of them, so all of them are reached.

Where the file does not state it — an unannotated parameter, or a chained `a.b.c` whose receiver is
an expression — **and** more than one declaration shares the name, no edge is produced. Choosing one
would point a reader at a type the line never mentions, which is the same reasoning as for object
shapes above. A single declaration of the name needs no receiver type, since there is nothing to
tell apart.

The cost is symmetrical to the object-shape decision and worth stating: a genuine access through an
unannotated receiver loses its edge. `typescript/member_access.ts` pins both halves, including the
non-edge.

Rust works the same way, with the type read from a parameter, a `let` annotation, a `let` whose
initializer names a type, or `self` inside an impl. `rust/method_receivers.rs` pins it.

Two consequences are worth stating outright, because they look like lost edges and are not:

- `fn measure(shape: impl Shape)` and `fn measure(shape: &dyn Shape)` state **the trait**, so
  `shape.area()` reaches the trait's declaration rather than any implementor's. Both fixtures
  recorded an implementor before the receiver's type was consulted, and both were wrong.
- a receiver whose type comes from a return value — `let cloned = item.clone(); cloned.display()` —
  is unknown, so a method name declared by more than one type resolves to nothing. Guessing pointed
  at an arbitrary implementor.

## Still unsettled

**Functional update.** Whether `Point { ..other }` should imply a dependency on every field
declaration of the struct. It currently reads only the base expression, pinned by a test in
`crates/core/tests/unit/languages/rust/field_initializer_tests.rs`.

**Lifetime parameters.** `struct H<'a> { v: &'a str }` produces no definition for `'a` and no edge
from the field to it, though renaming the parameter would break the field. The fixtures do not
annotate lifetime references, because the representation is undecided: it is unclear whether the
symbol should be `'a` or `a`, and annotating a guess would fix the wrong answer in the baseline.
Decide the representation first, then add the expectations.

**Settled since:** an impl's `Self::Item` now names the impl's own `type Item = i32` rather than
the trait's declaration, because the nearest enclosing scope decides. `rust/associated_types.rs`
pins it.
