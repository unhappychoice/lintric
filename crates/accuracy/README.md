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

Writing expectations is not free of error either. Several apparent defects turned out to be wrong
annotations — a JSX attribute does reference the prop it names, a type parameter reference is real,
`super::X` does not depend on the module's declaration. When a fixture disagrees with the analyzer,
check which one is wrong before filing.

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

This was an open question when the harness was written, and `rust/imports.rs` now pins the
answer. It is a decision rather than a fact about the language, so it is revisitable — but it
should be revisited by changing that fixture, not by discovering that behaviour drifted.

## Still unsettled

Whether a functional update, `Point { ..other }`, should imply a dependency on every field
declaration of the struct. It currently reads only the base expression, pinned by a test in
`crates/core/tests/unit/languages/rust/field_initializer_tests.rs`.
