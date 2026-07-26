use lintric_accuracy::edge::Edge;
use lintric_accuracy::shift::detect;

#[test]
fn recognises_every_target_moved_by_one_line() {
    // What inserting a sentence into the header comment does to a fixture.
    let missing = edges([(6, 4, "a"), (7, 5, "b")]);
    let spurious = edges([(6, 5, "a"), (7, 6, "b")]);

    assert_eq!(detect(&missing, &spurious), Some(1));
}

#[test]
fn recognises_a_shift_in_the_other_direction() {
    let missing = edges([(6, 5, "a")]);
    let spurious = edges([(6, 3, "a")]);

    assert_eq!(detect(&missing, &spurious), Some(-2));
}

#[test]
fn stays_quiet_when_the_offsets_disagree() {
    // Two unrelated defects are not a shift, and claiming otherwise would hide them.
    let missing = edges([(6, 4, "a"), (7, 5, "b")]);
    let spurious = edges([(6, 5, "a"), (7, 9, "b")]);

    assert_eq!(detect(&missing, &spurious), None);
}

#[test]
fn stays_quiet_when_only_some_edges_have_a_twin() {
    let missing = edges([(6, 4, "a"), (7, 5, "b")]);
    let spurious = edges([(6, 5, "a")]);

    assert_eq!(detect(&missing, &spurious), None);
}

#[test]
fn stays_quiet_when_a_symbol_appears_at_two_offsets() {
    // Ambiguous, so no offset is better than an arbitrary one.
    let missing = edges([(6, 4, "a"), (6, 8, "a")]);
    let spurious = edges([(6, 5, "a"), (6, 10, "a")]);

    assert_eq!(detect(&missing, &spurious), None);
}

#[test]
fn stays_quiet_when_nothing_is_wrong() {
    assert_eq!(detect(&[], &[]), None);
}

fn edges<const N: usize>(rows: [(usize, usize, &str); N]) -> Vec<Edge> {
    rows.into_iter()
        .map(|(source_line, target_line, symbol)| Edge {
            source_line,
            target_line,
            symbol: symbol.to_string(),
        })
        .collect()
}
