use lintric_core::languages::typescript::typescript_usage_node_collector::TypescriptUsageNodeCollector;
use lintric_core::usage_collector::UsageCollector;
use tree_sitter::Parser;

extern "C" {
    fn tree_sitter_typescript() -> tree_sitter::Language;
}

fn setup_typescript_parser() -> Parser {
    let mut parser = Parser::new();
    parser
        .set_language(unsafe { &tree_sitter_typescript() })
        .expect("Error loading TypeScript grammar");
    parser
}

#[test]
fn test_typescript_usage_collector_creation() {
    let source_code = "function main() {}";
    let collector = TypescriptUsageNodeCollector::new(source_code);

    // Test that collector can be created with source code
    let mut parser = setup_typescript_parser();
    let tree = parser.parse(source_code, None).unwrap();

    let result = collector.collect_usage_nodes(tree.root_node(), source_code);
    assert!(result.is_ok(), "Should successfully collect usage nodes");
}

#[test]
fn test_function_call_usage_collection() {
    let source_code = r#"
function main() {
    console.log("Hello");
    Math.max(1, 2, 3);
    helperFunction();
}

function helperFunction() {
    console.error("Debug message");
}
    "#;

    let collector = TypescriptUsageNodeCollector::new(source_code);
    let mut parser = setup_typescript_parser();
    let tree = parser.parse(source_code, None).unwrap();

    let usages = collector
        .collect_usage_nodes(tree.root_node(), source_code)
        .unwrap();

    // Should find identifier usages
    assert!(!usages.is_empty(), "Should find usage nodes");

    // Look for specific function calls
    let usage_names: Vec<_> = usages.iter().map(|u| &u.name).collect();
    assert!(
        usage_names.contains(&&"helperFunction".to_string()),
        "Should find helperFunction usage"
    );
}

#[test]
fn test_variable_usage_collection() {
    let source_code = r#"
function main() {
    const x = 5;
    let y: number = 10;
    const sum = x + y;
    console.log(`sum = ${sum}`);
    
    let counter = 0;
    counter += 1;
    counter *= 2;
}
    "#;

    let collector = TypescriptUsageNodeCollector::new(source_code);
    let mut parser = setup_typescript_parser();
    let tree = parser.parse(source_code, None).unwrap();

    let usages = collector
        .collect_usage_nodes(tree.root_node(), source_code)
        .unwrap();

    // Should find variable usages
    assert!(!usages.is_empty(), "Should find variable usages");

    // Look for specific variable names
    let usage_names: Vec<_> = usages.iter().map(|u| &u.name).collect();
    assert!(
        usage_names.contains(&&"x".to_string()),
        "Should find variable x usage"
    );
    assert!(
        usage_names.contains(&&"y".to_string()),
        "Should find variable y usage"
    );
    assert!(
        usage_names.contains(&&"sum".to_string()),
        "Should find variable sum usage"
    );
    assert!(
        usage_names.contains(&&"counter".to_string()),
        "Should find variable counter usage"
    );
}

#[test]
fn test_method_call_usage_collection() {
    let source_code = r#"
function main() {
    const s = new String("hello");
    const len = s.length;
    s.charAt(0);
    
    const arr = [1, 2, 3];
    arr.map(x => x * 2).filter(x => x > 2);
}
    "#;

    let collector = TypescriptUsageNodeCollector::new(source_code);
    let mut parser = setup_typescript_parser();
    let tree = parser.parse(source_code, None).unwrap();

    let usages = collector
        .collect_usage_nodes(tree.root_node(), source_code)
        .unwrap();

    // Should find method call usages
    assert!(!usages.is_empty(), "Should find method usages");

    let usage_names: Vec<_> = usages.iter().map(|u| &u.name).collect();
    assert!(
        usage_names.contains(&&"length".to_string()),
        "Should find length property usage"
    );
    assert!(
        usage_names.contains(&&"charAt".to_string()),
        "Should find charAt method usage"
    );
}

#[test]
fn test_object_property_access_usage() {
    let source_code = r#"
interface Point {
    x: number;
    y: number;
}

function main() {
    const p: Point = { x: 10, y: 20 };
    const xVal = p.x;
    const yVal = p.y;
    
    const { x, y } = p;
    console.log(`x: ${x}, y: ${y}`);
}
    "#;

    let collector = TypescriptUsageNodeCollector::new(source_code);
    let mut parser = setup_typescript_parser();
    let tree = parser.parse(source_code, None).unwrap();

    let usages = collector
        .collect_usage_nodes(tree.root_node(), source_code)
        .unwrap();

    // Should find property access usages
    assert!(!usages.is_empty(), "Should find property access usages");

    let usage_names: Vec<_> = usages.iter().map(|u| &u.name).collect();
    assert!(
        usage_names.contains(&&"x".to_string()),
        "Should find x property usage"
    );
    assert!(
        usage_names.contains(&&"y".to_string()),
        "Should find y property usage"
    );
    assert!(
        usage_names.contains(&&"p".to_string()),
        "Should find p object usage"
    );
}

#[test]
fn test_arrow_function_usage_collection() {
    let source_code = r#"
function main() {
    const captured = 42;
    
    const arrowFn = (x: number) => {
        const local = x + captured;
        return local * 2;
    };
    
    const result = arrowFn(10);
    console.log(`result: ${result}`);
}
    "#;

    let collector = TypescriptUsageNodeCollector::new(source_code);
    let mut parser = setup_typescript_parser();
    let tree = parser.parse(source_code, None).unwrap();

    let usages = collector
        .collect_usage_nodes(tree.root_node(), source_code)
        .unwrap();

    // Should find arrow function variable captures and usages
    assert!(!usages.is_empty(), "Should find arrow function usages");

    let usage_names: Vec<_> = usages.iter().map(|u| &u.name).collect();
    assert!(
        usage_names.contains(&&"captured".to_string()),
        "Should find captured variable usage"
    );
    assert!(
        usage_names.contains(&&"arrowFn".to_string()),
        "Should find arrow function usage"
    );
}

#[test]
fn test_class_usage_collection() {
    let source_code = r#"
class User {
    name: string;
    
    constructor(name: string) {
        this.name = name;
    }
    
    getName(): string {
        return this.name;
    }
}

function main() {
    const user = new User("John");
    const userName = user.getName();
    console.log(userName);
}
    "#;

    let collector = TypescriptUsageNodeCollector::new(source_code);
    let mut parser = setup_typescript_parser();
    let tree = parser.parse(source_code, None).unwrap();

    let usages = collector
        .collect_usage_nodes(tree.root_node(), source_code)
        .unwrap();

    // Should find class and method usages
    assert!(!usages.is_empty(), "Should find class usages");

    let usage_names: Vec<_> = usages.iter().map(|u| &u.name).collect();
    assert!(
        usage_names.contains(&&"User".to_string()),
        "Should find User class usage"
    );
    assert!(
        usage_names.contains(&&"getName".to_string()),
        "Should find getName method usage"
    );
}

#[test]
fn test_import_usage_collection() {
    let source_code = r#"
import { readFileSync } from 'fs';
import * as path from 'path';
import express from 'express';

function main() {
    const content = readFileSync('file.txt', 'utf8');
    const fullPath = path.join(__dirname, 'data');
    const app = express();
}
    "#;

    let collector = TypescriptUsageNodeCollector::new(source_code);
    let mut parser = setup_typescript_parser();
    let tree = parser.parse(source_code, None).unwrap();

    let usages = collector
        .collect_usage_nodes(tree.root_node(), source_code)
        .unwrap();

    // Should find import usages
    assert!(!usages.is_empty(), "Should find import usages");

    let usage_names: Vec<_> = usages.iter().map(|u| &u.name).collect();
    assert!(
        usage_names.contains(&&"readFileSync".to_string()),
        "Should find readFileSync usage"
    );
    assert!(
        usage_names.contains(&&"path".to_string()),
        "Should find path usage"
    );
    assert!(
        usage_names.contains(&&"express".to_string()),
        "Should find express usage"
    );
}

#[test]
fn test_type_usage_collection() {
    let source_code = r#"
type UserID = string;
interface User {
    id: UserID;
    name: string;
}

function createUser(id: UserID, name: string): User {
    return { id, name };
}

function main() {
    const userId: UserID = "123";
    const user: User = createUser(userId, "John");
    console.log(user);
}
    "#;

    let collector = TypescriptUsageNodeCollector::new(source_code);
    let mut parser = setup_typescript_parser();
    let tree = parser.parse(source_code, None).unwrap();

    let usages = collector
        .collect_usage_nodes(tree.root_node(), source_code)
        .unwrap();

    // Should find type usages
    assert!(!usages.is_empty(), "Should find type usages");

    let usage_names: Vec<_> = usages.iter().map(|u| &u.name).collect();
    assert!(
        usage_names.contains(&&"UserID".to_string()),
        "Should find UserID type usage"
    );
    assert!(
        usage_names.contains(&&"User".to_string()),
        "Should find User interface usage"
    );
}

#[test]
fn test_generic_usage_collection() {
    let source_code = r#"
function identity<T>(arg: T): T {
    return arg;
}

class Container<T> {
    private value: T;
    
    constructor(value: T) {
        this.value = value;
    }
    
    getValue(): T {
        return this.value;
    }
}

function main() {
    const result = identity<string>("hello");
    const container = new Container<number>(42);
    const value = container.getValue();
}
    "#;

    let collector = TypescriptUsageNodeCollector::new(source_code);
    let mut parser = setup_typescript_parser();
    let tree = parser.parse(source_code, None).unwrap();

    let usages = collector
        .collect_usage_nodes(tree.root_node(), source_code)
        .unwrap();

    // Should find generic function and class usages
    assert!(!usages.is_empty(), "Should find generic usages");

    let usage_names: Vec<_> = usages.iter().map(|u| &u.name).collect();
    assert!(
        usage_names.contains(&&"identity".to_string()),
        "Should find identity function usage"
    );
    assert!(
        usage_names.contains(&&"Container".to_string()),
        "Should find Container class usage"
    );
    assert!(
        usage_names.contains(&&"getValue".to_string()),
        "Should find getValue method usage"
    );
}
