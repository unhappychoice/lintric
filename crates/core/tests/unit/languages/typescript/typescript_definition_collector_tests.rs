#[cfg(test)]
mod tests {
    use lintric_core::definition_collectors::DefinitionCollector;
    use lintric_core::languages::typescript::typescript_definition_collector::TypescriptDefinitionCollector;
    use lintric_core::models::{DefinitionType, ScopeType};
    use tree_sitter::Parser;

    fn get_typescript_parser() -> Parser {
        let mut parser = Parser::new();
        parser
            .set_language(&tree_sitter_typescript::language_typescript())
            .expect("Error loading TypeScript grammar");
        parser
    }

    #[test]
    fn test_variable_definitions() {
        let collector = TypescriptDefinitionCollector::new("");
        let mut parser = get_typescript_parser();
        let code = r#"
            const x = 5;
            let y: string = "hello";
            var z;
        "#;

        let tree = parser.parse(code, None).unwrap();
        let symbol_table = collector.collect(code, tree.root_node()).unwrap();

        let symbols = symbol_table.get_all_symbols();
        assert_eq!(symbols.len(), 3);
        assert!(symbols.contains_key("x"));
        assert!(symbols.contains_key("y"));
        assert!(symbols.contains_key("z"));

        let x_entries = symbols.get("x").unwrap();
        assert_eq!(
            x_entries[0].definition.definition_type,
            DefinitionType::VariableDefinition
        );
    }

    #[test]
    fn test_function_definitions() {
        let collector = TypescriptDefinitionCollector::new("");
        let mut parser = get_typescript_parser();
        let code = r#"
            function add(a: number, b: number): number {
                return a + b;
            }
            const multiply = (x: number, y: number) => x * y;
        "#;

        let tree = parser.parse(code, None).unwrap();
        let symbol_table = collector.collect(code, tree.root_node()).unwrap();

        let symbols = symbol_table.get_all_symbols();
        assert!(symbols.contains_key("add"));
        assert!(symbols.contains_key("multiply"));
        assert!(symbols.contains_key("a"));
        assert!(symbols.contains_key("b"));
        assert!(symbols.contains_key("x"));
        assert!(symbols.contains_key("y"));

        let add_entries = symbols.get("add").unwrap();
        assert_eq!(
            add_entries[0].definition.definition_type,
            DefinitionType::FunctionDefinition
        );
    }

    #[test]
    fn test_class_definitions() {
        let collector = TypescriptDefinitionCollector::new("");
        let mut parser = get_typescript_parser();
        let code = r#"
            class Person {
                private name: string;
                public age: number;
                
                constructor(name: string, age: number) {
                    this.name = name;
                    this.age = age;
                }
                
                getName(): string {
                    return this.name;
                }
            }
        "#;

        let tree = parser.parse(code, None).unwrap();
        let symbol_table = collector.collect(code, tree.root_node()).unwrap();

        let symbols = symbol_table.get_all_symbols();
        assert!(symbols.contains_key("Person"));
        assert!(symbols.contains_key("name"));
        assert!(symbols.contains_key("age"));
        assert!(symbols.contains_key("getName"));

        let person_entries = symbols.get("Person").unwrap();
        assert_eq!(
            person_entries[0].definition.definition_type,
            DefinitionType::ClassDefinition
        );

        let name_entries = symbols.get("name").unwrap();
        assert_eq!(
            name_entries[0].definition.definition_type,
            DefinitionType::PropertyDefinition
        );
    }

    #[test]
    fn test_interface_definitions() {
        let collector = TypescriptDefinitionCollector::new("");
        let mut parser = get_typescript_parser();
        let code = r#"
            interface User {
                id: number;
                name: string;
                getProfile(): Profile;
            }
        "#;

        let tree = parser.parse(code, None).unwrap();
        let symbol_table = collector.collect(code, tree.root_node()).unwrap();

        let symbols = symbol_table.get_all_symbols();
        assert!(symbols.contains_key("User"));
        assert!(symbols.contains_key("id"));
        assert!(symbols.contains_key("name"));
        assert!(symbols.contains_key("getProfile"));

        let user_entries = symbols.get("User").unwrap();
        assert_eq!(
            user_entries[0].definition.definition_type,
            DefinitionType::InterfaceDefinition
        );

        let id_entries = symbols.get("id").unwrap();
        assert_eq!(
            id_entries[0].definition.definition_type,
            DefinitionType::PropertyDefinition
        );

        let get_profile_entries = symbols.get("getProfile").unwrap();
        assert_eq!(
            get_profile_entries[0].definition.definition_type,
            DefinitionType::MethodDefinition
        );
    }

    #[test]
    fn test_type_alias_definitions() {
        let collector = TypescriptDefinitionCollector::new("");
        let mut parser = get_typescript_parser();
        let code = r#"
            type Status = "pending" | "completed" | "failed";
            type User<T> = {
                id: number;
                data: T;
            };
        "#;

        let tree = parser.parse(code, None).unwrap();
        let symbol_table = collector.collect(code, tree.root_node()).unwrap();

        let symbols = symbol_table.get_all_symbols();
        assert!(symbols.contains_key("Status"));
        assert!(symbols.contains_key("User"));
        assert!(symbols.contains_key("T"));

        let status_entries = symbols.get("Status").unwrap();
        assert_eq!(
            status_entries[0].definition.definition_type,
            DefinitionType::TypeDefinition
        );
    }

    #[test]
    fn test_namespace_definitions() {
        let collector = TypescriptDefinitionCollector::new("");
        let mut parser = get_typescript_parser();
        let code = r#"
            namespace Utils {
                export function helper() {
                    return "helper";
                }
            }
        "#;

        let tree = parser.parse(code, None).unwrap();
        let symbol_table = collector.collect(code, tree.root_node()).unwrap();

        let symbols = symbol_table.get_all_symbols();
        assert!(symbols.contains_key("Utils"));
        assert!(symbols.contains_key("helper"));

        let utils_entries = symbols.get("Utils").unwrap();
        assert_eq!(
            utils_entries[0].definition.definition_type,
            DefinitionType::ModuleDefinition
        );
    }

    #[test]
    fn test_import_definitions() {
        let collector = TypescriptDefinitionCollector::new("");
        let mut parser = get_typescript_parser();
        let code = r#"
            import { readFile, writeFile } from 'fs';
            import * as path from 'path';
            import express from 'express';
        "#;

        let tree = parser.parse(code, None).unwrap();
        let symbol_table = collector.collect(code, tree.root_node()).unwrap();

        let symbols = symbol_table.get_all_symbols();
        assert!(symbols.contains_key("readFile"));
        assert!(symbols.contains_key("writeFile"));
        assert!(symbols.contains_key("path"));
        assert!(symbols.contains_key("express"));

        let read_file_entries = symbols.get("readFile").unwrap();
        assert_eq!(
            read_file_entries[0].definition.definition_type,
            DefinitionType::ImportDefinition
        );
    }

    #[test]
    fn test_scope_hierarchy() {
        let collector = TypescriptDefinitionCollector::new("");
        let mut parser = get_typescript_parser();
        let code = r#"
            class MyClass {
                method() {
                    if (true) {
                        let x = 1;
                    }
                }
            }
        "#;

        let tree = parser.parse(code, None).unwrap();
        let symbol_table = collector.collect(code, tree.root_node()).unwrap();

        let scopes = &symbol_table.scopes;
        assert!(scopes.scopes.len() > 1);

        let class_scopes: Vec<_> = scopes
            .scopes
            .values()
            .filter(|s| s.scope_type == ScopeType::Class)
            .collect();
        assert_eq!(class_scopes.len(), 1);

        let function_scopes: Vec<_> = scopes
            .scopes
            .values()
            .filter(|s| s.scope_type == ScopeType::Function)
            .collect();
        assert_eq!(function_scopes.len(), 1);

        let block_scopes: Vec<_> = scopes
            .scopes
            .values()
            .filter(|s| s.scope_type == ScopeType::Block)
            .collect();
        assert_eq!(block_scopes.len(), 1);
    }

    #[test]
    fn test_destructuring_patterns() {
        let collector = TypescriptDefinitionCollector::new("");
        let mut parser = get_typescript_parser();
        let code = r#"
            const { name, age } = user;
            const [first, second] = array;
            const { prop: aliasName } = obj;
        "#;

        let tree = parser.parse(code, None).unwrap();
        let symbol_table = collector.collect(code, tree.root_node()).unwrap();

        let symbols = symbol_table.get_all_symbols();
        assert!(symbols.contains_key("name"));
        assert!(symbols.contains_key("age"));
        assert!(symbols.contains_key("first"));
        assert!(symbols.contains_key("second"));
        assert!(symbols.contains_key("aliasName"));
    }

    #[test]
    fn test_generic_type_parameters() {
        let collector = TypescriptDefinitionCollector::new("");
        let mut parser = get_typescript_parser();
        let code = r#"
            function identity<T>(arg: T): T {
                return arg;
            }
            
            class Container<T, U> {
                value: T;
                helper: U;
            }
        "#;

        let tree = parser.parse(code, None).unwrap();
        let symbol_table = collector.collect(code, tree.root_node()).unwrap();

        let symbols = symbol_table.get_all_symbols();
        assert!(symbols.contains_key("T"));
        assert!(symbols.contains_key("U"));

        let t_entries = symbols.get("T").unwrap();
        assert_eq!(
            t_entries[0].definition.definition_type,
            DefinitionType::TypeDefinition
        );
    }
}
