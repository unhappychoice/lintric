use lintric_core::models::{ScopeTree, ScopeType, Position};

#[test]
fn test_scope_tree_creation() {
    let scope_tree = ScopeTree::new();
    
    // Should have a root global scope
    assert!(scope_tree.get_scope(0).is_some());
    assert_eq!(scope_tree.get_scope(0).unwrap().scope_type, ScopeType::Global);
}

#[test]
fn test_scope_tree_create_child_scope() {
    let mut scope_tree = ScopeTree::new();
    
    let child_id = scope_tree.create_scope(
        Some(0),
        ScopeType::Function,
        Position { start_line: 2, start_column: 1, end_line: 2, end_column: 1 },
        Position { start_line: 5, start_column: 1, end_line: 5, end_column: 1 },
    );
    
    assert!(scope_tree.get_scope(child_id).is_some());
    assert_eq!(scope_tree.get_scope(child_id).unwrap().parent, Some(0));
}