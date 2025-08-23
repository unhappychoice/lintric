use tree_sitter::{Language, Parser};

use lintric_core::{
    dependency_resolver::ScopeBuilder,
    models::{Position, ScopeType},
};

use super::fixtures;

extern "C" {
    fn tree_sitter_rust() -> Language;
    fn tree_sitter_typescript() -> Language;
}

#[test]
fn test_rust_nested_scope_building() {
    let mut parser = Parser::new();
    unsafe {
        parser.set_language(&tree_sitter_rust()).unwrap();
    }

    let tree = parser.parse(fixtures::BASIC_SCOPE_TEST_RUST, None).unwrap();
    let mut builder = ScopeBuilder::new("rust".to_string());

    let scope_tree = builder
        .build_from_ast(tree.root_node(), fixtures::BASIC_SCOPE_TEST_RUST)
        .unwrap();

    // Should have global scope
    assert!(scope_tree.get_scope(0).is_some());

    // Should have multiple function scopes
    let function_scopes: Vec<_> = scope_tree
        .scopes
        .values()
        .filter(|s| matches!(s.scope_type, ScopeType::Function))
        .collect();
    assert!(function_scopes.len() >= 3); // main, outer_function, inner_function, module_function, private_function

    // Should have block scopes
    let block_scopes: Vec<_> = scope_tree
        .scopes
        .values()
        .filter(|s| matches!(s.scope_type, ScopeType::Block))
        .collect();
    assert!(!block_scopes.is_empty());

    // Should have module scopes
    let module_scopes: Vec<_> = scope_tree
        .scopes
        .values()
        .filter(|s| matches!(s.scope_type, ScopeType::Module))
        .collect();
    assert!(!module_scopes.is_empty());
}

#[test]
fn test_typescript_class_interface_scope_building() {
    let mut parser = Parser::new();
    unsafe {
        parser.set_language(&tree_sitter_typescript()).unwrap();
    }

    let tree = parser
        .parse(fixtures::COMPLEX_TYPESCRIPT_SCOPE, None)
        .unwrap();
    let mut builder = ScopeBuilder::new("typescript".to_string());

    let scope_tree = builder
        .build_from_ast(tree.root_node(), fixtures::COMPLEX_TYPESCRIPT_SCOPE)
        .unwrap();

    // Should have class scopes
    let class_scopes: Vec<_> = scope_tree
        .scopes
        .values()
        .filter(|s| matches!(s.scope_type, ScopeType::Class))
        .collect();
    assert!(!class_scopes.is_empty());

    // Should have interface scopes
    let interface_scopes: Vec<_> = scope_tree
        .scopes
        .values()
        .filter(|s| matches!(s.scope_type, ScopeType::Interface))
        .collect();
    assert!(!interface_scopes.is_empty());

    // Should have function scopes
    let function_scopes: Vec<_> = scope_tree
        .scopes
        .values()
        .filter(|s| matches!(s.scope_type, ScopeType::Function))
        .collect();
    assert!(function_scopes.len() >= 3); // constructor, method, innerFunction, globalFunction
}

#[test]
fn test_rust_impl_trait_scope_building() {
    let mut parser = Parser::new();
    unsafe {
        parser.set_language(&tree_sitter_rust()).unwrap();
    }

    let tree = parser.parse(fixtures::RUST_IMPL_TRAIT_SCOPE, None).unwrap();
    let mut builder = ScopeBuilder::new("rust".to_string());

    let scope_tree = builder
        .build_from_ast(tree.root_node(), fixtures::RUST_IMPL_TRAIT_SCOPE)
        .unwrap();

    // Should have impl scopes
    let impl_scopes: Vec<_> = scope_tree
        .scopes
        .values()
        .filter(|s| matches!(s.scope_type, ScopeType::Impl))
        .collect();
    assert!(impl_scopes.len() >= 2); // impl MyStruct, impl MyTrait for MyStruct

    // Should have trait scopes
    let trait_scopes: Vec<_> = scope_tree
        .scopes
        .values()
        .filter(|s| matches!(s.scope_type, ScopeType::Trait))
        .collect();
    assert!(!trait_scopes.is_empty());

    // Should have function scopes within impl blocks
    let function_scopes: Vec<_> = scope_tree
        .scopes
        .values()
        .filter(|s| matches!(s.scope_type, ScopeType::Function))
        .collect();
    assert!(function_scopes.len() >= 3); // new, get_field, trait_method
}

#[test]
fn test_scope_position_detection_detailed() {
    let mut builder = ScopeBuilder::new("rust".to_string());
    let scope_tree = &mut builder.scope_tree;

    let func_scope_id = scope_tree.create_scope(
        Some(0),
        ScopeType::Function,
        Position {
            start_line: 2,
            start_column: 1,
            end_line: 2,
            end_column: 1,
        },
        Position {
            start_line: 10,
            start_column: 1,
            end_line: 10,
            end_column: 1,
        },
    );

    let block_scope_id = scope_tree.create_scope(
        Some(func_scope_id),
        ScopeType::Block,
        Position {
            start_line: 5,
            start_column: 5,
            end_line: 5,
            end_column: 5,
        },
        Position {
            start_line: 8,
            start_column: 5,
            end_line: 8,
            end_column: 5,
        },
    );

    // Test position in block scope
    let position_in_block = Position {
        start_line: 6,
        start_column: 8,
        end_line: 6,
        end_column: 8,
    };
    let found_scope = scope_tree.find_scope_at_position(&position_in_block);
    assert_eq!(found_scope, Some(block_scope_id));

    // Test position in function scope but not in block
    let position_in_function = Position {
        start_line: 3,
        start_column: 5,
        end_line: 3,
        end_column: 5,
    };
    let found_scope = scope_tree.find_scope_at_position(&position_in_function);
    assert_eq!(found_scope, Some(func_scope_id));

    // Test position in global scope
    let position_in_global = Position {
        start_line: 15,
        start_column: 1,
        end_line: 15,
        end_column: 1,
    };
    let found_scope = scope_tree.find_scope_at_position(&position_in_global);
    assert_eq!(found_scope, Some(0));

    // Test boundary conditions
    let position_at_function_start = Position {
        start_line: 2,
        start_column: 1,
        end_line: 2,
        end_column: 1,
    };
    let found_scope = scope_tree.find_scope_at_position(&position_at_function_start);
    assert_eq!(found_scope, Some(func_scope_id));

    let position_at_function_end = Position {
        start_line: 10,
        start_column: 1,
        end_line: 10,
        end_column: 1,
    };
    let found_scope = scope_tree.find_scope_at_position(&position_at_function_end);
    assert_eq!(found_scope, Some(func_scope_id));
}
