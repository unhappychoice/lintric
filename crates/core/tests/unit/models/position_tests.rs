use lintric_core::models::Position;
use tree_sitter::Parser;

fn setup_rust_parser() -> Parser {
    let mut parser = Parser::new();
    parser
        .set_language(&tree_sitter_rust::language())
        .expect("Error loading Rust grammar");
    parser
}

#[test]
fn test_position_creation() {
    let position = Position {
        start_line: 1,
        start_column: 5,
        end_line: 1,
        end_column: 15,
    };

    assert_eq!(position.start_line, 1);
    assert_eq!(position.start_column, 5);
    assert_eq!(position.end_line, 1);
    assert_eq!(position.end_column, 15);
}

#[test]
fn test_position_from_node() {
    let source_code = "fn test() {}";
    let mut parser = setup_rust_parser();
    let tree = parser.parse(source_code, None).unwrap();
    let root_node = tree.root_node();

    // Get the function node
    let function_node = root_node.child(0).unwrap();
    let position = Position::from_node(&function_node);

    assert_eq!(position.start_line, 1);
    assert_eq!(position.start_column, 1);
}

#[test]
fn test_position_equality() {
    let pos1 = Position {
        start_line: 1,
        start_column: 1,
        end_line: 1,
        end_column: 10,
    };

    let pos2 = Position {
        start_line: 1,
        start_column: 1,
        end_line: 1,
        end_column: 10,
    };

    let pos3 = Position {
        start_line: 2,
        start_column: 1,
        end_line: 2,
        end_column: 10,
    };

    assert_eq!(pos1.start_line, pos2.start_line);
    assert_eq!(pos1.start_column, pos2.start_column);
    assert_ne!(pos1.start_line, pos3.start_line);
}

#[test]
fn test_position_clone() {
    let original = Position {
        start_line: 5,
        start_column: 10,
        end_line: 5,
        end_column: 20,
    };

    let cloned = original.clone();

    assert_eq!(original.start_line, cloned.start_line);
    assert_eq!(original.start_column, cloned.start_column);
}

#[test]
fn test_position_debug() {
    let position = Position {
        start_line: 1,
        start_column: 1,
        end_line: 1,
        end_column: 10,
    };

    let debug_string = format!("{:?}", position);
    assert!(debug_string.contains("start_line"));
    assert!(debug_string.contains("start_column"));
    assert!(debug_string.contains("end_line"));
    assert!(debug_string.contains("end_column"));
}
