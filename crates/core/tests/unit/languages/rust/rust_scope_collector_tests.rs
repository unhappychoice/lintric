use lintric_core::languages::rust::rust_scope_collector::RustScopeCollector;
use lintric_core::models::ScopeType;
use lintric_core::scope_collector::ScopeCollector;
use tree_sitter::Parser;

fn setup_rust_parser() -> Parser {
    let mut parser = Parser::new();
    parser.set_language(&tree_sitter_rust::language()).expect("Error loading Rust grammar");
    parser
}

#[test]
fn test_rust_scope_collector_creation() {
    let collector = RustScopeCollector::new();
    
    // Test that collector can be created
    assert_eq!(collector.scope_tree.get_scope(0).unwrap().scope_type, ScopeType::Global);
}

#[test]
fn test_function_scope_collection() {
    let source_code = r#"
fn test_function() {
    let x = 1;
    println!("x = {}", x);
}

fn another_function(param: i32) -> i32 {
    let result = param + 1;
    result
}
    "#;
    
    let mut collector = RustScopeCollector::new();
    let mut parser = setup_rust_parser();
    let tree = parser.parse(source_code, None).unwrap();
    
    let scope_tree = collector.scopes(tree.root_node(), source_code).unwrap();
    
    // Should have global scope and function scopes
    let scopes = scope_tree.get_all_scopes();
    assert!(scopes.len() >= 3, "Should have at least global and two function scopes");
    
    // Should have function scopes
    let function_scopes: Vec<_> = scopes.iter()
        .filter(|scope| scope.scope_type == ScopeType::Function)
        .collect();
    assert!(!function_scopes.is_empty(), "Should have function scopes");
}

#[test]
fn test_block_scope_collection() {
    let source_code = r#"
fn main() {
    let x = 1;
    
    {
        let y = 2;
        println!("y = {}", y);
    }
    
    if x > 0 {
        let z = 3;
        println!("z = {}", z);
    }
}
    "#;
    
    let mut collector = RustScopeCollector::new();
    let mut parser = setup_rust_parser();
    let tree = parser.parse(source_code, None).unwrap();
    
    let scope_tree = collector.scopes(tree.root_node(), source_code).unwrap();
    
    // Should have global, function, and block scopes
    let scopes = scope_tree.get_all_scopes();
    assert!(scopes.len() >= 4, "Should have at least 4 scopes");
    
    // Should have block scopes
    let block_scopes: Vec<_> = scopes.iter()
        .filter(|scope| scope.scope_type == ScopeType::Block)
        .collect();
    assert!(!block_scopes.is_empty(), "Should have block scopes");
}

#[test]
fn test_nested_function_scopes() {
    let source_code = r#"
fn outer() {
    let outer_var = 1;
    
    fn inner() {
        let inner_var = 2;
        
        fn deeply_nested() {
            let deep_var = 3;
        }
        
        deeply_nested();
    }
    
    inner();
}
    "#;
    
    let mut collector = RustScopeCollector::new();
    let mut parser = setup_rust_parser();
    let tree = parser.parse(source_code, None).unwrap();
    
    let scope_tree = collector.scopes(tree.root_node(), source_code).unwrap();
    
    // Should handle nested function scopes
    let scopes = scope_tree.get_all_scopes();
    assert!(scopes.len() >= 4, "Should handle nested function scopes");
    
    // Should have multiple function scopes
    let function_scopes: Vec<_> = scopes.iter()
        .filter(|scope| scope.scope_type == ScopeType::Function)
        .collect();
    assert!(function_scopes.len() >= 3, "Should have at least 3 function scopes");
}

#[test]
fn test_struct_impl_scope_collection() {
    let source_code = r#"
struct TestStruct {
    field: i32,
}

impl TestStruct {
    fn new(value: i32) -> Self {
        TestStruct { field: value }
    }
    
    fn get_field(&self) -> i32 {
        self.field
    }
    
    fn set_field(&mut self, value: i32) {
        self.field = value;
    }
}
    "#;
    
    let mut collector = RustScopeCollector::new();
    let mut parser = setup_rust_parser();
    let tree = parser.parse(source_code, None).unwrap();
    
    let scope_tree = collector.scopes(tree.root_node(), source_code).unwrap();
    
    let scopes = scope_tree.get_all_scopes();
    assert!(scopes.len() >= 4, "Should handle impl block and method scopes");
    
    // Should have function scopes for methods
    let function_scopes: Vec<_> = scopes.iter()
        .filter(|scope| scope.scope_type == ScopeType::Function)
        .collect();
    assert!(function_scopes.len() >= 3, "Should have method scopes");
}

#[test]
fn test_module_scope_collection() {
    let source_code = r#"
mod utils {
    pub fn helper() -> i32 {
        42
    }
    
    mod inner {
        pub fn inner_helper() -> String {
            "hello".to_string()
        }
    }
}

mod network {
    pub fn connect() {
        // connection logic
    }
}
    "#;
    
    let mut collector = RustScopeCollector::new();
    let mut parser = setup_rust_parser();
    let tree = parser.parse(source_code, None).unwrap();
    
    let scope_tree = collector.scopes(tree.root_node(), source_code).unwrap();
    
    let scopes = scope_tree.get_all_scopes();
    assert!(scopes.len() >= 6, "Should handle module and function scopes");
    
    // Should have module scopes
    let module_scopes: Vec<_> = scopes.iter()
        .filter(|scope| scope.scope_type == ScopeType::Module)
        .collect();
    assert!(!module_scopes.is_empty(), "Should have module scopes");
}

#[test]
fn test_loop_scope_collection() {
    let source_code = r#"
fn main() {
    for i in 0..10 {
        let loop_var = i * 2;
        println!("loop_var = {}", loop_var);
    }
    
    while true {
        let condition_var = 42;
        if condition_var > 40 {
            break;
        }
    }
    
    loop {
        let infinite_var = 1;
        if infinite_var > 0 {
            break;
        }
    }
}
    "#;
    
    let mut collector = RustScopeCollector::new();
    let mut parser = setup_rust_parser();
    let tree = parser.parse(source_code, None).unwrap();
    
    let scope_tree = collector.scopes(tree.root_node(), source_code).unwrap();
    
    let scopes = scope_tree.get_all_scopes();
    assert!(scopes.len() >= 5, "Should handle loop and block scopes");
    
    // Should have block scopes for loops
    let block_scopes: Vec<_> = scopes.iter()
        .filter(|scope| scope.scope_type == ScopeType::Block)
        .collect();
    assert!(!block_scopes.is_empty(), "Should have block scopes for loops");
}

#[test]
fn test_closure_scope_collection() {
    let source_code = r#"
fn main() {
    let captured = 42;
    
    let closure = |x| {
        let closure_var = x + captured;
        closure_var * 2
    };
    
    let result = closure(10);
    println!("result = {}", result);
}
    "#;
    
    let mut collector = RustScopeCollector::new();
    let mut parser = setup_rust_parser();
    let tree = parser.parse(source_code, None).unwrap();
    
    let scope_tree = collector.scopes(tree.root_node(), source_code).unwrap();
    
    let scopes = scope_tree.get_all_scopes();
    assert!(scopes.len() >= 3, "Should handle closure scopes");
    
    // Should have function scope for the closure
    let function_scopes: Vec<_> = scopes.iter()
        .filter(|scope| scope.scope_type == ScopeType::Function)
        .collect();
    assert!(function_scopes.len() >= 2, "Should have main function and closure scopes");
}

#[test]
fn test_match_scope_collection() {
    let source_code = r#"
fn main() {
    let value = Some(42);
    
    match value {
        Some(x) => {
            let matched = x * 2;
            println!("matched = {}", matched);
        },
        None => {
            let default = 0;
            println!("default = {}", default);
        }
    }
}
    "#;
    
    let mut collector = RustScopeCollector::new();
    let mut parser = setup_rust_parser();
    let tree = parser.parse(source_code, None).unwrap();
    
    let scope_tree = collector.scopes(tree.root_node(), source_code).unwrap();
    
    let scopes = scope_tree.get_all_scopes();
    assert!(scopes.len() >= 4, "Should handle match arm scopes");
    
    // Should have block scopes for match arms
    let block_scopes: Vec<_> = scopes.iter()
        .filter(|scope| scope.scope_type == ScopeType::Block)
        .collect();
    assert!(!block_scopes.is_empty(), "Should have block scopes for match arms");
}

#[test]
fn test_scope_hierarchy() {
    let source_code = r#"
fn outer() {
    let outer_var = 1;
    
    {
        let block_var = 2;
        
        fn inner() {
            let inner_var = 3;
        }
    }
}
    "#;
    
    let mut collector = RustScopeCollector::new();
    let mut parser = setup_rust_parser();
    let tree = parser.parse(source_code, None).unwrap();
    
    let scope_tree = collector.scopes(tree.root_node(), source_code).unwrap();
    
    // Test that scopes have proper parent-child relationships
    let scopes = scope_tree.get_all_scopes();
    assert!(scopes.len() >= 4, "Should have proper scope hierarchy");
    
    // Global scope should be the root
    let global_scope = scope_tree.get_scope(0).unwrap();
    assert_eq!(global_scope.scope_type, ScopeType::Global);
    assert!(global_scope.parent.is_none(), "Global scope should have no parent");
    
    // Should have children
    assert!(!global_scope.children.is_empty(), "Global scope should have children");
}