use lintric_core::definition_collectors::DefinitionCollector;
use lintric_core::languages::rust::rust_definition_collector::RustDefinitionCollector;
use lintric_core::models::DefinitionType;
use tree_sitter::Parser;

fn setup_rust_parser() -> Parser {
    let mut parser = Parser::new();
    parser.set_language(&tree_sitter_rust::language()).expect("Error loading Rust grammar");
    parser
}

#[test]
fn test_rust_definition_collector_creation() {
    let source_code = "fn main() {}";
    let collector = RustDefinitionCollector::new(source_code);
    
    // Test that collector can be created with source code
    let mut parser = setup_rust_parser();
    let tree = parser.parse(source_code, None).unwrap();
    
    let result = collector.collect_definitions_from_root(tree.root_node());
    assert!(result.is_ok(), "Should successfully collect definitions");
}

#[test]
fn test_function_definition_collection() {
    let source_code = r#"
fn test_function() {
    println!("Hello, world!");
}

fn another_function(param: i32) -> i32 {
    param + 1
}
    "#;
    
    let collector = RustDefinitionCollector::new(source_code);
    let mut parser = setup_rust_parser();
    let tree = parser.parse(source_code, None).unwrap();
    
    let definitions = collector.collect_definitions_from_root(tree.root_node()).unwrap();
    
    // Should find function definitions
    let func_defs: Vec<_> = definitions.iter()
        .filter(|d| d.definition_type == DefinitionType::FunctionDefinition)
        .collect();
    
    assert!(!func_defs.is_empty(), "Should find function definitions");
    
    // Check for specific function names
    let function_names: Vec<_> = func_defs.iter().map(|d| &d.name).collect();
    assert!(function_names.contains(&&"test_function".to_string()), "Should find test_function");
    assert!(function_names.contains(&&"another_function".to_string()), "Should find another_function");
}

#[test]
fn test_struct_definition_collection() {
    let source_code = r#"
struct TestStruct {
    field1: i32,
    field2: String,
}

struct EmptyStruct;

struct TupleStruct(i32, String);
    "#;
    
    let collector = RustDefinitionCollector::new(source_code);
    let mut parser = setup_rust_parser();
    let tree = parser.parse(source_code, None).unwrap();
    
    let definitions = collector.collect_definitions_from_root(tree.root_node()).unwrap();
    
    // Should find struct definitions
    let struct_defs: Vec<_> = definitions.iter()
        .filter(|d| d.definition_type == DefinitionType::StructDefinition)
        .collect();
    
    assert!(!struct_defs.is_empty(), "Should find struct definitions");
    
    // Check for specific struct names
    let struct_names: Vec<_> = struct_defs.iter().map(|d| &d.name).collect();
    assert!(struct_names.contains(&&"TestStruct".to_string()), "Should find TestStruct");
    assert!(struct_names.contains(&&"EmptyStruct".to_string()), "Should find EmptyStruct");
    assert!(struct_names.contains(&&"TupleStruct".to_string()), "Should find TupleStruct");
}

#[test]
fn test_variable_definition_collection() {
    let source_code = r#"
fn main() {
    let x = 5;
    let mut y = 10;
    let (a, b) = (1, 2);
    
    for i in 0..10 {
        let loop_var = i * 2;
    }
}
    "#;
    
    let collector = RustDefinitionCollector::new(source_code);
    let mut parser = setup_rust_parser();
    let tree = parser.parse(source_code, None).unwrap();
    
    let definitions = collector.collect_definitions_from_root(tree.root_node()).unwrap();
    
    // Should find variable definitions
    let var_defs: Vec<_> = definitions.iter()
        .filter(|d| d.definition_type == DefinitionType::VariableDefinition)
        .collect();
    
    assert!(!var_defs.is_empty(), "Should find variable definitions");
    
    // Check for specific variable names
    let var_names: Vec<_> = var_defs.iter().map(|d| &d.name).collect();
    assert!(var_names.contains(&&"x".to_string()), "Should find variable x");
    assert!(var_names.contains(&&"y".to_string()), "Should find variable y");
}

#[test]
fn test_enum_definition_collection() {
    let source_code = r#"
enum Color {
    Red,
    Green,
    Blue,
}

enum Message {
    Quit,
    Move { x: i32, y: i32 },
    Write(String),
}
    "#;
    
    let collector = RustDefinitionCollector::new(source_code);
    let mut parser = setup_rust_parser();
    let tree = parser.parse(source_code, None).unwrap();
    
    let definitions = collector.collect_definitions_from_root(tree.root_node()).unwrap();
    
    // Should find enum definitions
    let enum_defs: Vec<_> = definitions.iter()
        .filter(|d| d.definition_type == DefinitionType::EnumDefinition)
        .collect();
    
    assert!(!enum_defs.is_empty(), "Should find enum definitions");
    
    let enum_names: Vec<_> = enum_defs.iter().map(|d| &d.name).collect();
    assert!(enum_names.contains(&&"Color".to_string()), "Should find Color enum");
    assert!(enum_names.contains(&&"Message".to_string()), "Should find Message enum");
}

#[test]
fn test_trait_definition_collection() {
    let source_code = r#"
trait Display {
    fn fmt(&self) -> String;
}

trait Clone {
    fn clone(&self) -> Self;
    
    fn clone_from(&mut self, source: &Self) {
        *self = source.clone();
    }
}
    "#;
    
    let collector = RustDefinitionCollector::new(source_code);
    let mut parser = setup_rust_parser();
    let tree = parser.parse(source_code, None).unwrap();
    
    let definitions = collector.collect_definitions_from_root(tree.root_node()).unwrap();
    
    // Should find trait definitions
    let trait_defs: Vec<_> = definitions.iter()
        .filter(|d| d.definition_type == DefinitionType::TypeDefinition)
        .collect();
    
    assert!(!trait_defs.is_empty(), "Should find trait definitions");
    
    let trait_names: Vec<_> = trait_defs.iter().map(|d| &d.name).collect();
    assert!(trait_names.contains(&&"Display".to_string()), "Should find Display trait");
    assert!(trait_names.contains(&&"Clone".to_string()), "Should find Clone trait");
}

#[test]
fn test_impl_block_method_collection() {
    let source_code = r#"
struct Point {
    x: i32,
    y: i32,
}

impl Point {
    fn new(x: i32, y: i32) -> Self {
        Point { x, y }
    }
    
    fn distance_from_origin(&self) -> f64 {
        ((self.x * self.x + self.y * self.y) as f64).sqrt()
    }
}
    "#;
    
    let collector = RustDefinitionCollector::new(source_code);
    let mut parser = setup_rust_parser();
    let tree = parser.parse(source_code, None).unwrap();
    
    let definitions = collector.collect_definitions_from_root(tree.root_node()).unwrap();
    
    // Should find both struct and method definitions
    let struct_defs: Vec<_> = definitions.iter()
        .filter(|d| d.definition_type == DefinitionType::StructDefinition)
        .collect();
    let method_defs: Vec<_> = definitions.iter()
        .filter(|d| d.definition_type == DefinitionType::MethodDefinition)
        .collect();
    
    assert!(!struct_defs.is_empty(), "Should find struct definitions");
    assert!(!method_defs.is_empty(), "Should find method definitions");
    
    let method_names: Vec<_> = method_defs.iter().map(|d| &d.name).collect();
    assert!(method_names.contains(&&"new".to_string()), "Should find new method");
    assert!(method_names.contains(&&"distance_from_origin".to_string()), "Should find distance_from_origin method");
}

#[test]
fn test_module_definition_collection() {
    let source_code = r#"
mod utils {
    pub fn helper() -> i32 {
        42
    }
}

pub mod network {
    pub mod tcp {
        pub fn connect() {
            // connection logic
        }
    }
}
    "#;
    
    let collector = RustDefinitionCollector::new(source_code);
    let mut parser = setup_rust_parser();
    let tree = parser.parse(source_code, None).unwrap();
    
    let definitions = collector.collect_definitions_from_root(tree.root_node()).unwrap();
    
    // Should find module definitions
    let mod_defs: Vec<_> = definitions.iter()
        .filter(|d| d.definition_type == DefinitionType::ModuleDefinition)
        .collect();
    
    assert!(!mod_defs.is_empty(), "Should find module definitions");
    
    let mod_names: Vec<_> = mod_defs.iter().map(|d| &d.name).collect();
    assert!(mod_names.contains(&&"utils".to_string()), "Should find utils module");
    assert!(mod_names.contains(&&"network".to_string()), "Should find network module");
    assert!(mod_names.contains(&&"tcp".to_string()), "Should find tcp module");
}