//! Recognising a fixture whose annotations were written against a different numbering.
//!
//! Inserting a line — a sentence in the header comment, usually — moves every line below it, and
//! every annotation written before that edit then points one or more lines short. The harness sees
//! it as a fixture full of missing and spurious edges, which reads like a broken analyzer rather
//! than a stale annotation.
//!
//! The signature is that each missing edge has a spurious twin naming the same symbol, at the same
//! offset. Saying so turns a wall of edges into one sentence.

use crate::edge::Edge;

/// The offset by which every missing edge is a shifted spurious one, when that is what happened.
///
/// Requires the same count on both sides and one consistent non-zero offset, so a fixture that
/// genuinely lost and gained edges is not explained away as a shift.
pub fn detect(missing: &[Edge], spurious: &[Edge]) -> Option<isize> {
    if missing.is_empty() || missing.len() != spurious.len() {
        return None;
    }

    let offset = offset_between(&missing[0], spurious)?;

    missing
        .iter()
        .all(|edge| offset_between(edge, spurious) == Some(offset))
        .then_some(offset)
}

/// The single offset at which this edge appears among the spurious ones.
///
/// A symbol appearing at two different offsets is ambiguous, so no offset is reported rather than
/// an arbitrary one.
fn offset_between(missing: &Edge, spurious: &[Edge]) -> Option<isize> {
    let mut offsets = spurious
        .iter()
        .filter(|candidate| candidate.symbol == missing.symbol)
        .filter(|candidate| candidate.source_line == missing.source_line)
        .map(|candidate| candidate.target_line as isize - missing.target_line as isize)
        .filter(|offset| *offset != 0)
        .collect::<Vec<_>>();

    offsets.dedup();
    match offsets.as_slice() {
        [offset] => Some(*offset),
        _ => None,
    }
}

/// What to tell the reader when a shift is what happened.
pub fn explain(offset: isize) -> String {
    let direction = if offset > 0 { "later" } else { "earlier" };
    let lines = offset.abs();
    let plural = if lines == 1 { "line" } else { "lines" };

    format!(
        "  every missing edge is a spurious one {lines} {plural} {direction}: the annotations were \
         written against a different numbering, so check whether a line was added or removed above \
         them\n"
    )
}
