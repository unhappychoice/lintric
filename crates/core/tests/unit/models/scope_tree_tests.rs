use lintric_core::models::{Position, ScopeTree, ScopeType};

/// A file whose scopes nest: a function spanning lines 2–9 holding a block spanning 4–6.
fn nested() -> (ScopeTree, usize, usize) {
    let mut tree = ScopeTree::new();
    let function = tree.create_scope(Some(0), ScopeType::Function, span(2, 9));
    let block = tree.create_scope(Some(function), ScopeType::Block, span(4, 6));

    (tree, function, block)
}

#[test]
fn finds_the_innermost_scope_containing_a_position() {
    let (tree, function, block) = nested();

    assert_eq!(tree.find_scope_at_position(&span(5, 5)), Some(block));
    assert_eq!(tree.find_scope_at_position(&span(8, 8)), Some(function));
}

#[test]
fn a_position_outside_every_scope_belongs_to_the_file() {
    let (tree, _, _) = nested();

    assert_eq!(tree.find_scope_at_position(&span(1, 1)), Some(0));
}

#[test]
fn a_scope_chain_runs_outward_from_the_scope_itself() {
    let (tree, function, block) = nested();

    assert_eq!(tree.get_parent_scopes(block), vec![function, 0]);
    assert_eq!(tree.get_parent_scopes(function), vec![0]);
}

#[test]
fn the_file_scope_has_nothing_outside_it() {
    let (tree, _, _) = nested();

    assert!(tree.get_parent_scopes(0).is_empty());
}

#[test]
fn a_scope_records_the_children_created_under_it() {
    let (tree, function, block) = nested();

    assert_eq!(tree.get_scope(function).unwrap().children, vec![block]);
    assert!(tree.get_scope(block).unwrap().children.is_empty());
}

#[test]
fn a_scope_that_was_never_created_is_absent() {
    let (tree, _, _) = nested();

    assert!(tree.get_scope(99).is_none());
}

fn span(start_line: usize, end_line: usize) -> Position {
    Position {
        start_line,
        start_column: 1,
        end_line,
        end_column: 1,
    }
}
