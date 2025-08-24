use lintric_core::languages::typescript::typescript_definition_collector::TypescriptDefinitionCollector;
use lintric_core::definition_collectors::DefinitionCollector;
use lintric_core::models::DefinitionType;
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
fn test_typescript_definition_collector_creation() {
    let source_code = "function test() {}";
    let _collector = TypescriptDefinitionCollector::new(source_code);
    // Test that collector can be created with source code
}

#[test]
fn test_function_definition_collection() {
    let source_code = r#"
function testFunction() {
    console.log("Hello, world!");
}

function anotherFunction(param: number): number {
    return param + 1;
}

const arrowFunction = (x: number) => x * 2;
    "#;
    
    let collector = TypescriptDefinitionCollector::new(source_code);
    let mut parser = setup_typescript_parser();
    let tree = parser.parse(source_code, None).unwrap();
    
    let definitions = collector.collect_definitions_from_root(tree.root_node()).unwrap();
    
    // Should find function definitions
    assert!(!definitions.is_empty(), "Should find function definitions");
    
    let function_defs: Vec<_> = definitions.iter()
        .filter(|def| matches!(def.definition_type, DefinitionType::FunctionDefinition))
        .collect();
    
    assert!(!function_defs.is_empty(), "Should find function definitions");
}

#[test]
fn test_variable_definition_collection() {
    let source_code = r#"
const x = 5;
let y: number = 10;
var z = "hello";

const { a, b } = { a: 1, b: 2 };
const [first, second] = [1, 2];
    "#;
    
    let collector = TypescriptDefinitionCollector::new(source_code);
    let mut parser = setup_typescript_parser();
    let tree = parser.parse(source_code, None).unwrap();
    
    let definitions = collector.collect_definitions_from_root(tree.root_node()).unwrap();
    
    // Should find variable definitions
    assert!(!definitions.is_empty(), "Should find variable definitions");
    
    let var_defs: Vec<_> = definitions.iter()
        .filter(|def| matches!(def.definition_type, DefinitionType::VariableDefinition))
        .collect();
    
    assert!(!var_defs.is_empty(), "Should find variable definitions");
    
    // Check for specific variable names
    let def_names: Vec<_> = definitions.iter().map(|d| &d.name).collect();
    assert!(def_names.contains(&&"x".to_string()), "Should find variable x definition");
    assert!(def_names.contains(&&"y".to_string()), "Should find variable y definition");
}

#[test]
fn test_class_definition_collection() {
    let source_code = r#"
class TestClass {
    private field: number;
    public name: string;
    
    constructor(name: string) {
        this.name = name;
        this.field = 0;
    }
    
    public getValue(): number {
        return this.field;
    }
    
    private setValue(value: number): void {
        this.field = value;
    }
}

abstract class AbstractBase {
    abstract doSomething(): void;
}
    "#;
    
    let collector = TypescriptDefinitionCollector::new(source_code);
    let mut parser = setup_typescript_parser();
    let tree = parser.parse(source_code, None).unwrap();
    
    let definitions = collector.collect_definitions_from_root(tree.root_node()).unwrap();
    
    // Should find class definitions
    let class_defs: Vec<_> = definitions.iter()
        .filter(|def| matches!(def.definition_type, DefinitionType::ClassDefinition))
        .collect();
    
    assert!(!class_defs.is_empty(), "Should find class definitions");
    
    // Should find property definitions
    let prop_defs: Vec<_> = definitions.iter()
        .filter(|def| matches!(def.definition_type, DefinitionType::PropertyDefinition))
        .collect();
    
    assert!(!prop_defs.is_empty(), "Should find property definitions");
    
    let def_names: Vec<_> = definitions.iter().map(|d| &d.name).collect();
    assert!(def_names.contains(&&"TestClass".to_string()), "Should find TestClass definition");
    assert!(def_names.contains(&&"AbstractBase".to_string()), "Should find AbstractBase definition");
}

#[test]
fn test_interface_definition_collection() {
    let source_code = r#"
interface User {
    id: number;
    name: string;
    email?: string;
    
    getName(): string;
    setEmail(email: string): void;
}

interface Admin extends User {
    role: string;
    permissions: string[];
}
    "#;
    
    let collector = TypescriptDefinitionCollector::new(source_code);
    let mut parser = setup_typescript_parser();
    let tree = parser.parse(source_code, None).unwrap();
    
    let definitions = collector.collect_definitions_from_root(tree.root_node()).unwrap();
    
    // Should find interface definitions
    let interface_defs: Vec<_> = definitions.iter()
        .filter(|def| matches!(def.definition_type, DefinitionType::InterfaceDefinition))
        .collect();
    
    assert!(!interface_defs.is_empty(), "Should find interface definitions");
    
    let def_names: Vec<_> = definitions.iter().map(|d| &d.name).collect();
    assert!(def_names.contains(&&"User".to_string()), "Should find User interface definition");
    assert!(def_names.contains(&&"Admin".to_string()), "Should find Admin interface definition");
}

#[test]
fn test_type_definition_collection() {
    let source_code = r#"
type StringOrNumber = string | number;
type UserID = number;
type EventHandler<T> = (event: T) => void;

type ComplexType = {
    id: number;
    data: string[];
    callback: () => void;
};
    "#;
    
    let collector = TypescriptDefinitionCollector::new(source_code);
    let mut parser = setup_typescript_parser();
    let tree = parser.parse(source_code, None).unwrap();
    
    let definitions = collector.collect_definitions_from_root(tree.root_node()).unwrap();
    
    // Should find type definitions
    let type_defs: Vec<_> = definitions.iter()
        .filter(|def| matches!(def.definition_type, DefinitionType::TypeDefinition))
        .collect();
    
    assert!(!type_defs.is_empty(), "Should find type definitions");
    
    let def_names: Vec<_> = definitions.iter().map(|d| &d.name).collect();
    assert!(def_names.contains(&&"StringOrNumber".to_string()), "Should find StringOrNumber type definition");
    assert!(def_names.contains(&&"UserID".to_string()), "Should find UserID type definition");
}

#[test]
fn test_import_definition_collection() {
    let source_code = r#"
import { Component, useState } from 'react';
import * as fs from 'fs';
import path from 'path';
import { default as express, Router } from 'express';
    "#;
    
    let collector = TypescriptDefinitionCollector::new(source_code);
    let mut parser = setup_typescript_parser();
    let tree = parser.parse(source_code, None).unwrap();
    
    let definitions = collector.collect_definitions_from_root(tree.root_node()).unwrap();
    
    // Should find import definitions
    let import_defs: Vec<_> = definitions.iter()
        .filter(|def| matches!(def.definition_type, DefinitionType::ImportDefinition))
        .collect();
    
    assert!(!import_defs.is_empty(), "Should find import definitions");
    
    let def_names: Vec<_> = definitions.iter().map(|d| &d.name).collect();
    assert!(def_names.contains(&&"Component".to_string()), "Should find Component import definition");
    assert!(def_names.contains(&&"fs".to_string()), "Should find fs import definition");
    assert!(def_names.contains(&&"path".to_string()), "Should find path import definition");
}

#[test]
fn test_generic_type_parameters() {
    let source_code = r#"
function identity<T>(arg: T): T {
    return arg;
}

class Container<T, K extends string> {
    private value: T;
    private key: K;
    
    constructor(value: T, key: K) {
        this.value = value;
        this.key = key;
    }
}

interface Repository<T> {
    find(id: string): T | null;
    save(entity: T): void;
}
    "#;
    
    let collector = TypescriptDefinitionCollector::new(source_code);
    let mut parser = setup_typescript_parser();
    let tree = parser.parse(source_code, None).unwrap();
    
    let definitions = collector.collect_definitions_from_root(tree.root_node()).unwrap();
    
    // Should find generic type parameter definitions
    assert!(!definitions.is_empty(), "Should find type parameter definitions");
    
    let type_defs: Vec<_> = definitions.iter()
        .filter(|def| matches!(def.definition_type, DefinitionType::TypeDefinition))
        .collect();
    
    assert!(!type_defs.is_empty(), "Should find type definitions including type parameters");
}