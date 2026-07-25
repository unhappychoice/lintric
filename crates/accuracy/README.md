# Lintric Accuracy

Measures how accurately Lintric detects line-level dependencies, and records the result as a
baseline so that changes in detection behaviour are visible in diffs.

## Why this exists

The rest of the test suite cannot answer "how accurate are we?". Snapshot tests record whatever
the analyzer currently outputs, so wrong output is locked in as faithfully as right output. The
generated tests in `crates/test-generator` assert that a node kind analyzes without panicking,
which is a useful crash net but silent about correctness.

This crate takes the opposite approach: expectations are written by hand from the language's
semantics, **independent of what the analyzer currently produces**. That makes it possible to
report precision and recall rather than merely detecting change.

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
| Duplicates | Edges the analyzer reported more than once |
| Precision | Correct / Detected |
| Recall | Correct / Expected |

Duplicates are reported separately rather than folded into precision. The line-to-line
dependency graph should contain each edge once, so repeated edges are an over-counting defect in
their own right — see #172 — and keeping them in their own column stops them from being confused
with false positives.

Precision and recall are derived when reporting rather than stored, so `baseline.json` holds
only integer counts and does not churn on float formatting.

## Interpreting the baseline

The current numbers describe **these fixtures only**. They are small, focused files, and the
recall figure should not be read as an overall accuracy claim for the analyzer — real code fares
considerably worse. `crates/cli/src/logger.rs`, for instance, yields 2 dependencies for 24 lines
because every method body is a single `println!("{message}")`.

Growing the fixture set will most likely push the recorded numbers *down*, and that is the
intended direction: it means the harness is measuring more of what the analyzer actually has to
handle.

## Deliberately unsettled

Import and cross-scope resolution is not covered yet, because the expected shape is a design
question rather than a fact about the language. Given

```rust
mod geometry {
    pub struct Rect { pub width: i32 }
}
use geometry::Rect;

fn main() {
    let r = Rect { width: 1 };
}
```

it is unclear whether `Rect` in `main` should depend on the `use` line (treating the import as
the local binding) or directly on the struct definition. #87 proposes that import-like
dependencies be placed at the top scope level, which implies the former, but this has not been
decided. Fixtures for modules and imports should be added once it is, so that the baseline does
not encode an accidental answer.
