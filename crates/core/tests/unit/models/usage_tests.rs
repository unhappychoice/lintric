use lintric_core::models::{Usage, UsageKind, Position};
use tree_sitter::Parser;

fn setup_rust_parser() -> Parser {
    let mut parser = Parser::new();
    parser.set_language(&tree_sitter_rust::language()).expect("Error loading Rust grammar");
    parser
}

#[test]
fn test_usage_creation() {
    let position = Position {
        start_line: 1,
        start_column: 1,
        end_line: 1,
        end_column: 10,
    };
    
    let usage = Usage {
        name: "variable_name".to_string(),
        kind: UsageKind::Identifier,
        position: position.clone(),
    };
    
    assert_eq!(usage.name, "variable_name");
    assert_eq!(usage.kind, UsageKind::Identifier);
    assert_eq!(usage.position.start_line, position.start_line);
}

#[test]
fn test_usage_kinds() {
    let position = Position {
        start_line: 1,
        start_column: 1,
        end_line: 1,
        end_column: 10,
    };
    
    let identifier_usage = Usage {
        name: "var".to_string(),
        kind: UsageKind::Identifier,
        position: position.clone(),
    };
    
    let call_usage = Usage {
        name: "function_call".to_string(),
        kind: UsageKind::CallExpression,
        position: position.clone(),
    };
    
    let field_usage = Usage {
        name: "field".to_string(),
        kind: UsageKind::FieldExpression,
        position: position.clone(),
    };
    
    assert!(matches!(identifier_usage.kind, UsageKind::Identifier));
    assert!(matches!(call_usage.kind, UsageKind::CallExpression));
    assert!(matches!(field_usage.kind, UsageKind::FieldExpression));
}

#[test]
fn test_usage_new() {
    let _source_code = "variable_name";
    let mut parser = setup_rust_parser();
    let tree = parser.parse("fn main() { variable_name; }", None).unwrap();
    
    // Find an identifier node
    let root = tree.root_node();
    let function = root.child(0).unwrap();
    let body = function.child_by_field_name("body").unwrap();
    let statement = body.child(1).unwrap(); // Skip opening brace
    let expression = statement.child(0).unwrap();
    
    if expression.kind() == "identifier" {
        let usage = Usage::new(&expression, "fn main() { variable_name; }", UsageKind::Identifier);
        assert_eq!(usage.name, "variable_name");
        assert_eq!(usage.kind, UsageKind::Identifier);
    }
}

#[test]
fn test_usage_new_simple() {
    let position = Position {
        start_line: 5,
        start_column: 10,
        end_line: 5,
        end_column: 20,
    };
    
    let usage = Usage::new_simple(
        "test_usage".to_string(),
        position.clone(),
        UsageKind::Read,
    );
    
    assert_eq!(usage.name, "test_usage");
    assert_eq!(usage.position.start_line, position.start_line);
    assert_eq!(usage.kind, UsageKind::Read);
}

#[test]
fn test_usage_new_call_expression() {
    let _source_code = "function_call()";
    let mut parser = setup_rust_parser();
    let tree = parser.parse("fn main() { function_call(); }", None).unwrap();
    
    let root = tree.root_node();
    let function = root.child(0).unwrap();
    let body = function.child_by_field_name("body").unwrap();
    let statement = body.child(1).unwrap(); // Skip opening brace
    let call_expr = statement.child(0).unwrap();
    
    if call_expr.kind() == "call_expression" {
        let usage = Usage::new_call_expression(&call_expr, "fn main() { function_call(); }");
        assert_eq!(usage.name, "function_call");
        assert_eq!(usage.kind, UsageKind::CallExpression);
    }
}

#[test]
fn test_usage_fields_access() {
    let position = Position {
        start_line: 1,
        start_column: 1,
        end_line: 1,
        end_column: 10,
    };
    
    let usage = Usage {
        name: "test".to_string(),
        kind: UsageKind::Identifier,
        position: position.clone(),
    };
    
    assert_eq!(usage.name, "test");
    assert!(matches!(usage.kind, UsageKind::Identifier));
    assert_eq!(usage.position.start_line, position.start_line);
}