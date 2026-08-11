use crate::analysis::Detected;
use crate::edge::Edge;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::iter::Sum;

/// Result of comparing one fixture's expectations against analyzer output.
pub struct FixtureReport {
    pub name: String,
    pub counts: Counts,
    pub missing: Vec<Edge>,
    pub spurious: Vec<Edge>,
}

/// Countable outcome of a comparison. These are the numbers recorded in the baseline.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Counts {
    pub expected: usize,
    pub detected: usize,
    pub correct: usize,
    pub missing: usize,
    pub spurious: usize,
    pub duplicates: usize,
}

/// Compare hand-written expectations against analyzer output.
///
/// Fixtures are annotated exhaustively, so any detected edge that is not expected counts as
/// spurious rather than as unstated-but-acceptable.
pub fn compare(
    name: impl Into<String>,
    expected: &BTreeSet<Edge>,
    detected: Detected,
) -> FixtureReport {
    let missing: Vec<Edge> = expected.difference(&detected.edges).cloned().collect();
    let spurious: Vec<Edge> = detected.edges.difference(expected).cloned().collect();

    FixtureReport {
        name: name.into(),
        counts: Counts {
            expected: expected.len(),
            detected: detected.edges.len(),
            correct: expected.intersection(&detected.edges).count(),
            missing: missing.len(),
            spurious: spurious.len(),
            duplicates: detected.duplicates,
        },
        missing,
        spurious,
    }
}

impl Counts {
    /// Share of detected edges that are real.
    pub fn precision(&self) -> f64 {
        ratio(self.correct, self.detected)
    }

    /// Share of real edges that were detected.
    pub fn recall(&self) -> f64 {
        ratio(self.correct, self.expected)
    }
}

impl<'a> Sum<&'a Counts> for Counts {
    fn sum<I: Iterator<Item = &'a Counts>>(iter: I) -> Self {
        iter.fold(Counts::default(), |total, counts| Counts {
            expected: total.expected + counts.expected,
            detected: total.detected + counts.detected,
            correct: total.correct + counts.correct,
            missing: total.missing + counts.missing,
            spurious: total.spurious + counts.spurious,
            duplicates: total.duplicates + counts.duplicates,
        })
    }
}

fn ratio(numerator: usize, denominator: usize) -> f64 {
    match denominator {
        0 => 1.0,
        _ => numerator as f64 / denominator as f64,
    }
}
