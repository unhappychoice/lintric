use lintric_accuracy::analysis::Detected;
use lintric_accuracy::comparison::{compare, Counts};
use lintric_accuracy::edge::Edge;
use std::collections::BTreeSet;

#[test]
fn counts_an_exact_match_as_fully_correct() {
    let report = compare("f", &edges([(2, 1, "a")]), detected([(2, 1, "a")], 0));

    assert_eq!(report.counts.correct, 1);
    assert_eq!(report.counts.precision(), 1.0);
    assert_eq!(report.counts.recall(), 1.0);
}

#[test]
fn reports_an_expected_edge_the_analyzer_did_not_find_as_missing() {
    let report = compare(
        "f",
        &edges([(2, 1, "a"), (3, 1, "a")]),
        detected([(2, 1, "a")], 0),
    );

    assert_eq!(report.missing, vec![Edge::new(3, 1, "a")]);
    assert_eq!(report.counts.recall(), 0.5);
    assert_eq!(report.counts.precision(), 1.0);
}

#[test]
fn reports_an_unexpected_edge_as_spurious() {
    let report = compare(
        "f",
        &edges([(2, 1, "a")]),
        detected([(2, 1, "a"), (9, 1, "a")], 0),
    );

    assert_eq!(report.spurious, vec![Edge::new(9, 1, "a")]);
    assert_eq!(report.counts.precision(), 0.5);
    assert_eq!(report.counts.recall(), 1.0);
}

#[test]
fn treats_a_wrong_symbol_as_both_missing_and_spurious() {
    let report = compare("f", &edges([(2, 1, "a")]), detected([(2, 1, "b")], 0));

    assert_eq!(report.counts.missing, 1);
    assert_eq!(report.counts.spurious, 1);
    assert_eq!(report.counts.correct, 0);
}

#[test]
fn keeps_duplicates_out_of_the_precision_figure() {
    let report = compare("f", &edges([(2, 1, "a")]), detected([(2, 1, "a")], 3));

    assert_eq!(report.counts.duplicates, 3);
    assert_eq!(report.counts.precision(), 1.0);
}

#[test]
fn treats_an_empty_comparison_as_perfect_rather_than_undefined() {
    let report = compare("f", &edges([]), detected([], 0));

    assert_eq!(report.counts.precision(), 1.0);
    assert_eq!(report.counts.recall(), 1.0);
}

#[test]
fn sums_counts_across_fixtures() {
    let counts = [
        Counts {
            expected: 2,
            detected: 1,
            correct: 1,
            missing: 1,
            spurious: 0,
            duplicates: 1,
        },
        Counts {
            expected: 3,
            detected: 4,
            correct: 3,
            missing: 0,
            spurious: 1,
            duplicates: 2,
        },
    ];

    let total: Counts = counts.iter().sum();

    assert_eq!(
        total,
        Counts {
            expected: 5,
            detected: 5,
            correct: 4,
            missing: 1,
            spurious: 1,
            duplicates: 3
        }
    );
}

fn edges<const N: usize>(entries: [(usize, usize, &str); N]) -> BTreeSet<Edge> {
    entries
        .into_iter()
        .map(|(source, target, symbol)| Edge::new(source, target, symbol))
        .collect()
}

fn detected<const N: usize>(entries: [(usize, usize, &str); N], duplicates: usize) -> Detected {
    Detected {
        edges: edges(entries),
        duplicates,
    }
}
