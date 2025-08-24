use lintric_core::languages::typescript::typescript_scope_collector::TypeScriptScopeCollector;
use lintric_core::models::ScopeType;
use lintric_core::scope_collector::ScopeCollector;
use tree_sitter::Parser;

extern "C" {
    fn tree_sitter_typescript() -> tree_sitter::Language;
}

fn setup_typescript_parser() -> Parser {
    let mut parser = Parser::new();
    parser.set_language(unsafe { &tree_sitter_typescript() }).expect("Error loading TypeScript grammar");
    parser
}

#[test]
fn test_typescript_scope_collector_creation() {
    let collector = TypeScriptScopeCollector::new();
    
    // Test that collector can be created
    assert_eq!(collector.scope_tree.get_scope(0).unwrap().scope_type, ScopeType::Global);
}

#[test]
fn test_function_scope_collection() {
    let source_code = r#"
function testFunction() {
    let x = 1;
    console.log(x);
}

function anotherFunction(param: number): number {
    let result = param + 1;
    return result;
}

const arrowFunction = (a: number) => {
    let doubled = a * 2;
    return doubled;
};
    "#;
    
    let mut collector = TypeScriptScopeCollector::new();
    let mut parser = setup_typescript_parser();
    let tree = parser.parse(source_code, None).unwrap();
    
    let scope_tree = collector.scopes(tree.root_node(), source_code).unwrap();
    
    // Should have global scope and function scopes
    let scopes = scope_tree.get_all_scopes();
    assert!(scopes.len() >= 3, "Should have at least global and function scopes");
    
    // Should have function scopes
    let function_scopes: Vec<_> = scopes.iter()
        .filter(|scope| scope.scope_type == ScopeType::Function)
        .collect();
    assert!(!function_scopes.is_empty(), "Should have function scopes");
}

#[test]
fn test_block_scope_collection() {
    let source_code = r#"
function main() {
    let x = 1;
    
    {
        let y = 2;
        console.log(y);
    }
    
    if (x > 0) {
        let z = 3;
        console.log(z);
    }
    
    for (let i = 0; i < 10; i++) {
        let loopVar = i * 2;
        console.log(loopVar);
    }
}
    "#;
    
    let mut collector = TypeScriptScopeCollector::new();
    let mut parser = setup_typescript_parser();
    let tree = parser.parse(source_code, None).unwrap();
    
    let scope_tree = collector.scopes(tree.root_node(), source_code).unwrap();
    
    // Should have global, function, and block scopes
    let scopes = scope_tree.get_all_scopes();
    assert!(scopes.len() >= 2, "Should have at least 2 scopes");
    
    // Should have function scopes at minimum
    let function_scopes: Vec<_> = scopes.iter()
        .filter(|scope| scope.scope_type == ScopeType::Function)
        .collect();
    assert!(!function_scopes.is_empty(), "Should have function scopes");
}

#[test]
fn test_class_scope_collection() {
    let source_code = r#"
class TestClass {
    private field: number = 0;
    
    constructor(value: number) {
        this.field = value;
    }
    
    public getValue(): number {
        return this.field;
    }
    
    private calculate() {
        let temp = this.field * 2;
        return temp;
    }
}

abstract class BaseClass {
    abstract process(): void;
    
    protected helper() {
        console.log("Helper method");
    }
}
    "#;
    
    let mut collector = TypeScriptScopeCollector::new();
    let mut parser = setup_typescript_parser();
    let tree = parser.parse(source_code, None).unwrap();
    
    let scope_tree = collector.scopes(tree.root_node(), source_code).unwrap();
    
    let scopes = scope_tree.get_all_scopes();
    assert!(scopes.len() >= 4, "Should handle class and method scopes");
    
    // Should have class scopes
    let class_scopes: Vec<_> = scopes.iter()
        .filter(|scope| scope.scope_type == ScopeType::Class)
        .collect();
    assert!(!class_scopes.is_empty(), "Should have class scopes");
    
    // Should have function scopes for methods
    let function_scopes: Vec<_> = scopes.iter()
        .filter(|scope| scope.scope_type == ScopeType::Function)
        .collect();
    assert!(function_scopes.len() >= 3, "Should have method scopes");
}

#[test]
fn test_interface_scope_collection() {
    let source_code = r#"
interface User {
    id: number;
    name: string;
    
    getName(): string;
    setName(name: string): void;
}

interface Admin extends User {
    role: string;
    permissions: string[];
    
    hasPermission(permission: string): boolean;
}
    "#;
    
    let mut collector = TypeScriptScopeCollector::new();
    let mut parser = setup_typescript_parser();
    let tree = parser.parse(source_code, None).unwrap();
    
    let scope_tree = collector.scopes(tree.root_node(), source_code).unwrap();
    
    let scopes = scope_tree.get_all_scopes();
    assert!(scopes.len() >= 3, "Should handle interface scopes");
    
    // Should have interface scopes
    let interface_scopes: Vec<_> = scopes.iter()
        .filter(|scope| scope.scope_type == ScopeType::Interface)
        .collect();
    assert!(!interface_scopes.is_empty(), "Should have interface scopes");
}

#[test]
fn test_module_scope_collection() {
    let source_code = r#"
declare module "my-module" {
    export function helper(): string;
    export const VERSION: string;
}

namespace Utils {
    export function formatString(str: string): string {
        return str.trim();
    }
    
    export namespace Math {
        export function add(a: number, b: number): number {
            return a + b;
        }
    }
}
    "#;
    
    let mut collector = TypeScriptScopeCollector::new();
    let mut parser = setup_typescript_parser();
    let tree = parser.parse(source_code, None).unwrap();
    
    let scope_tree = collector.scopes(tree.root_node(), source_code).unwrap();
    
    let scopes = scope_tree.get_all_scopes();
    assert!(scopes.len() >= 2, "Should handle module scopes");
    
    // Should have function scopes at minimum (namespace functions)
    let function_scopes: Vec<_> = scopes.iter()
        .filter(|scope| scope.scope_type == ScopeType::Function)
        .collect();
    assert!(!function_scopes.is_empty(), "Should have function scopes");
}

#[test]
fn test_nested_function_scopes() {
    let source_code = r#"
function outer() {
    let outerVar = 1;
    
    function inner() {
        let innerVar = 2;
        
        function deeplyNested() {
            let deepVar = 3;
            return deepVar;
        }
        
        return deeplyNested();
    }
    
    return inner();
}
    "#;
    
    let mut collector = TypeScriptScopeCollector::new();
    let mut parser = setup_typescript_parser();
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
fn test_arrow_function_scope_collection() {
    let source_code = r#"
const processData = (data: number[]) => {
    const filtered = data.filter(x => x > 0);
    const mapped = filtered.map((item, index) => {
        const processed = item * index;
        return processed;
    });
    return mapped;
};

const asyncOperation = async (id: string) => {
    const result = await fetch(`/api/${id}`);
    const data = await result.json();
    return data;
};
    "#;
    
    let mut collector = TypeScriptScopeCollector::new();
    let mut parser = setup_typescript_parser();
    let tree = parser.parse(source_code, None).unwrap();
    
    let scope_tree = collector.scopes(tree.root_node(), source_code).unwrap();
    
    let scopes = scope_tree.get_all_scopes();
    assert!(scopes.len() >= 4, "Should handle arrow function scopes");
    
    // Should have function scopes for arrow functions
    let function_scopes: Vec<_> = scopes.iter()
        .filter(|scope| scope.scope_type == ScopeType::Function)
        .collect();
    assert!(!function_scopes.is_empty(), "Should have function scopes for arrow functions");
}

#[test]
fn test_scope_hierarchy() {
    let source_code = r#"
class Container {
    private data: number[];
    
    constructor() {
        this.data = [];
    }
    
    public process() {
        const helper = (value: number) => {
            if (value > 0) {
                let temp = value * 2;
                return temp;
            }
            return 0;
        };
        
        return this.data.map(helper);
    }
}
    "#;
    
    let mut collector = TypeScriptScopeCollector::new();
    let mut parser = setup_typescript_parser();
    let tree = parser.parse(source_code, None).unwrap();
    
    let scope_tree = collector.scopes(tree.root_node(), source_code).unwrap();
    
    // Test that scopes have proper parent-child relationships
    let scopes = scope_tree.get_all_scopes();
    assert!(scopes.len() >= 5, "Should have proper scope hierarchy");
    
    // Global scope should be the root
    let global_scope = scope_tree.get_scope(0).unwrap();
    assert_eq!(global_scope.scope_type, ScopeType::Global);
    assert!(global_scope.parent.is_none(), "Global scope should have no parent");
    
    // Should have children
    assert!(!global_scope.children.is_empty(), "Global scope should have children");
}