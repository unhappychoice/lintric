use lintric_accuracy::edge::Edge;
use lintric_accuracy::expectation::parse_expectations;

#[test]
fn parses_a_single_expectation_against_the_annotated_line() {
    let expected = parse_expectations("let a = 1;\nlet b = a; //~ depends: a@1\n").unwrap();

    assert_eq!(expected, [Edge::new(2, 1, "a")].into_iter().collect());
}

#[test]
fn parses_several_expectations_on_one_line() {
    let expected = parse_expectations("//~ depends: a@10, b@11 , c@12").unwrap();

    assert_eq!(
        expected,
        [
            Edge::new(1, 10, "a"),
            Edge::new(1, 11, "b"),
            Edge::new(1, 12, "c")
        ]
        .into_iter()
        .collect()
    );
}

#[test]
fn keeps_same_symbol_resolving_to_different_lines_as_distinct_edges() {
    let expected = parse_expectations("//~ depends: x@2, x@5").unwrap();

    assert_eq!(expected.len(), 2);
}

#[test]
fn collapses_an_expectation_repeated_on_one_line() {
    let expected = parse_expectations("//~ depends: x@2, x@2").unwrap();

    assert_eq!(expected, [Edge::new(1, 2, "x")].into_iter().collect());
}

#[test]
fn finds_no_expectations_in_an_unannotated_source() {
    let expected = parse_expectations("let a = 1;\n// a plain comment\n").unwrap();

    assert!(expected.is_empty());
}

#[test]
fn rejects_an_unknown_directive_rather_than_ignoring_it() {
    let error = parse_expectations("let a = 1; //~ dependss: a@1").unwrap_err();

    assert_eq!(error.line, 1);
    assert!(error.message.contains("depends:"), "{}", error.message);
}

#[test]
fn rejects_an_entry_without_a_target_line() {
    let error = parse_expectations("//~ depends: a").unwrap_err();

    assert!(error.message.contains("symbol@line"), "{}", error.message);
}

#[test]
fn rejects_a_target_line_that_is_not_a_number() {
    let error = parse_expectations("//~ depends: a@one").unwrap_err();

    assert!(error.message.contains("line number"), "{}", error.message);
}
