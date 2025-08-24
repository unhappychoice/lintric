use lintric_core::languages::rust::rust_usage_node_collector::RustUsageNodeCollector;
use lintric_core::usage_collector::UsageCollector;
use tree_sitter::Parser;

fn setup_rust_parser() -> Parser {
    let mut parser = Parser::new();
    parser.set_language(&tree_sitter_rust::language()).expect("Error loading Rust grammar");
    parser
}

#[test]
fn test_rust_usage_collector_creation() {
    let source_code = "fn main() {}";
    let collector = RustUsageNodeCollector::new(source_code);
    
    // Test that collector can be created with source code
    let mut parser = setup_rust_parser();
    let tree = parser.parse(source_code, None).unwrap();
    
    let result = collector.collect_usage_nodes(tree.root_node(), source_code);
    assert!(result.is_ok(), "Should successfully collect usage nodes");
}

#[test]
fn test_function_call_usage_collection() {
    let source_code = r#"
fn main() {
    println!("Hello");
    std::process::exit(0);
    helper_function();
}

fn helper_function() {
    eprintln!("Debug message");
}
    "#;
    
    let collector = RustUsageNodeCollector::new(source_code);
    let mut parser = setup_rust_parser();
    let tree = parser.parse(source_code, None).unwrap();
    
    let usages = collector.collect_usage_nodes(tree.root_node(), source_code).unwrap();
    
    // Should find identifier usages
    assert!(!usages.is_empty(), "Should find usage nodes");
    
    // Look for specific function calls
    let usage_names: Vec<_> = usages.iter().map(|u| &u.name).collect();
    assert!(usage_names.contains(&&"helper_function".to_string()), "Should find helper_function usage");
}

#[test]
fn test_variable_usage_collection() {
    let source_code = r#"
fn main() {
    let x = 5;
    let y = 10;
    let sum = x + y;
    println!("sum = {}", sum);
    
    let mut counter = 0;
    counter += 1;
    counter *= 2;
}
    "#;
    
    let collector = RustUsageNodeCollector::new(source_code);
    let mut parser = setup_rust_parser();
    let tree = parser.parse(source_code, None).unwrap();
    
    let usages = collector.collect_usage_nodes(tree.root_node(), source_code).unwrap();
    
    // Should find variable usages
    assert!(!usages.is_empty(), "Should find variable usages");
    
    // Look for specific variable names
    let usage_names: Vec<_> = usages.iter().map(|u| &u.name).collect();
    assert!(usage_names.contains(&&"x".to_string()), "Should find variable x usage");
    assert!(usage_names.contains(&&"y".to_string()), "Should find variable y usage");
    assert!(usage_names.contains(&&"sum".to_string()), "Should find variable sum usage");
    assert!(usage_names.contains(&&"counter".to_string()), "Should find variable counter usage");
}

#[test]
fn test_method_call_usage_collection() {
    let source_code = r#"
fn main() {
    let s = String::new();
    let len = s.len();
    s.push_str("hello");
    
    let vec = vec![1, 2, 3];
    vec.iter().map(|x| x * 2).collect::<Vec<_>>();
}
    "#;
    
    let collector = RustUsageNodeCollector::new(source_code);
    let mut parser = setup_rust_parser();
    let tree = parser.parse(source_code, None).unwrap();
    
    let usages = collector.collect_usage_nodes(tree.root_node(), source_code).unwrap();
    
    // Should find method call usages
    assert!(!usages.is_empty(), "Should find method usages");
    
    let usage_names: Vec<_> = usages.iter().map(|u| &u.name).collect();
    assert!(usage_names.contains(&&"len".to_string()), "Should find len method usage");
    assert!(usage_names.contains(&&"push_str".to_string()), "Should find push_str method usage");
}

#[test]
fn test_struct_field_access_usage() {
    let source_code = r#"
struct Point {
    x: i32,
    y: i32,
}

fn main() {
    let p = Point { x: 10, y: 20 };
    let x_val = p.x;
    let y_val = p.y;
    
    let Point { x, y } = p;
    println!("x: {}, y: {}", x, y);
}
    "#;
    
    let collector = RustUsageNodeCollector::new(source_code);
    let mut parser = setup_rust_parser();
    let tree = parser.parse(source_code, None).unwrap();
    
    let usages = collector.collect_usage_nodes(tree.root_node(), source_code).unwrap();
    
    // Should find field access usages
    assert!(!usages.is_empty(), "Should find field access usages");
    
    let usage_names: Vec<_> = usages.iter().map(|u| &u.name).collect();
    assert!(usage_names.contains(&&"x".to_string()), "Should find x field usage");
    assert!(usage_names.contains(&&"y".to_string()), "Should find y field usage");
    assert!(usage_names.contains(&&"p".to_string()), "Should find p struct usage");
}

#[test]
fn test_closure_usage_collection() {
    let source_code = r#"
fn main() {
    let captured = 42;
    
    let closure = |x| {
        let local = x + captured;
        local * 2
    };
    
    let result = closure(10);
    println!("result: {}", result);
}
    "#;
    
    let collector = RustUsageNodeCollector::new(source_code);
    let mut parser = setup_rust_parser();
    let tree = parser.parse(source_code, None).unwrap();
    
    let usages = collector.collect_usage_nodes(tree.root_node(), source_code).unwrap();
    
    // Should find closure variable captures and usages
    assert!(!usages.is_empty(), "Should find closure usages");
    
    let usage_names: Vec<_> = usages.iter().map(|u| &u.name).collect();
    assert!(usage_names.contains(&&"captured".to_string()), "Should find captured variable usage");
    assert!(usage_names.contains(&&"closure".to_string()), "Should find closure usage");
}

#[test]
fn test_macro_usage_collection() {
    let source_code = r#"
macro_rules! debug_print {
    ($msg:expr) => {
        println!("DEBUG: {}", $msg);
    };
}

fn main() {
    let value = 42;
    debug_print!(value);
    println!("Value is {}", value);
    vec![1, 2, 3];
}
    "#;
    
    let collector = RustUsageNodeCollector::new(source_code);
    let mut parser = setup_rust_parser();
    let tree = parser.parse(source_code, None).unwrap();
    
    let usages = collector.collect_usage_nodes(tree.root_node(), source_code).unwrap();
    
    // Should find macro usages
    assert!(!usages.is_empty(), "Should find macro usages");
    
    let usage_names: Vec<_> = usages.iter().map(|u| &u.name).collect();
    assert!(usage_names.contains(&&"value".to_string()), "Should find value usage");
}

#[test]
fn test_module_path_usage_collection() {
    let source_code = r#"
use std::collections::HashMap;
use std::fs::File;

fn main() {
    let mut map = HashMap::new();
    map.insert("key", "value");
    
    let _file = File::open("test.txt");
    std::process::exit(0);
}
    "#;
    
    let collector = RustUsageNodeCollector::new(source_code);
    let mut parser = setup_rust_parser();
    let tree = parser.parse(source_code, None).unwrap();
    
    let usages = collector.collect_usage_nodes(tree.root_node(), source_code).unwrap();
    
    // Should find module path and type usages
    assert!(!usages.is_empty(), "Should find module path usages");
    
    let usage_names: Vec<_> = usages.iter().map(|u| &u.name).collect();
    assert!(usage_names.contains(&&"HashMap".to_string()), "Should find HashMap usage");
    assert!(usage_names.contains(&&"File".to_string()), "Should find File usage");
}

#[test]
fn test_pattern_matching_usage() {
    let source_code = r#"
enum Message {
    Quit,
    Move { x: i32, y: i32 },
    Write(String),
}

fn main() {
    let msg = Message::Write("hello".to_string());
    
    match msg {
        Message::Quit => println!("Quit"),
        Message::Move { x, y } => println!("Move to ({}, {})", x, y),
        Message::Write(text) => println!("Write: {}", text),
    }
}
    "#;
    
    let collector = RustUsageNodeCollector::new(source_code);
    let mut parser = setup_rust_parser();
    let tree = parser.parse(source_code, None).unwrap();
    
    let usages = collector.collect_usage_nodes(tree.root_node(), source_code).unwrap();
    
    // Should find pattern matching usages
    assert!(!usages.is_empty(), "Should find pattern matching usages");
    
    let usage_names: Vec<_> = usages.iter().map(|u| &u.name).collect();
    assert!(usage_names.contains(&&"Message".to_string()), "Should find Message enum usage");
    assert!(usage_names.contains(&&"msg".to_string()), "Should find msg variable usage");
}

#[test]
fn test_generic_usage_collection() {
    let source_code = r#"
fn generic_function<T>(value: T) -> T where T: Clone {
    value.clone()
}

fn main() {
    let vec: Vec<i32> = Vec::new();
    let result = generic_function(42);
    
    let option: Option<String> = Some("test".to_string());
    match option {
        Some(s) => println!("{}", s),
        None => println!("None"),
    }
}
    "#;
    
    let collector = RustUsageNodeCollector::new(source_code);
    let mut parser = setup_rust_parser();
    let tree = parser.parse(source_code, None).unwrap();
    
    let usages = collector.collect_usage_nodes(tree.root_node(), source_code).unwrap();
    
    // Should find generic type and function usages
    assert!(!usages.is_empty(), "Should find generic usages");
    
    let usage_names: Vec<_> = usages.iter().map(|u| &u.name).collect();
    assert!(usage_names.contains(&&"Vec".to_string()), "Should find Vec usage");
    assert!(usage_names.contains(&&"Option".to_string()), "Should find Option usage");
    assert!(usage_names.contains(&&"generic_function".to_string()), "Should find generic_function usage");
}