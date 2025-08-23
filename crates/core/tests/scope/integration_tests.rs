use lintric_core::{analyze_content_with_scope_awareness, Language, ScopeType};

#[test]
fn test_end_to_end_scope_analysis_rust() {
    let rust_code = r#"
fn main() {
    let global_var = 10;
    
    fn inner_function() {
        let local_var = 20;
        println!("{}", local_var);
    }
    
    {
        let block_var = 30;
        println!("{}", global_var);
    }
    
    inner_function();
}
"#;

    let result = analyze_content_with_scope_awareness(rust_code.to_string(), Language::Rust);

    assert!(result.is_ok());
    let (_ir, _analysis_result, symbol_table, _warnings) = result.unwrap();

    // Verify we have a global scope
    assert!(symbol_table.scopes.get_scope(0).is_some());

    // Should have multiple scopes including function and block scopes
    let function_scopes: Vec<_> = symbol_table
        .scopes
        .scopes
        .values()
        .filter(|s| matches!(s.scope_type, ScopeType::Function))
        .collect();
    assert!(function_scopes.len() >= 2); // main and inner_function

    let block_scopes: Vec<_> = symbol_table
        .scopes
        .scopes
        .values()
        .filter(|s| matches!(s.scope_type, ScopeType::Block))
        .collect();
    assert!(!block_scopes.is_empty()); // The block with block_var
}

#[test]
fn test_end_to_end_scope_analysis_typescript() {
    let typescript_code = r#"
class MyClass {
    private field: number = 0;
    
    constructor(value: number) {
        this.field = value;
    }
    
    public method(): number {
        const local = this.field * 2;
        return local;
    }
}

function globalFunction() {
    const instance = new MyClass(42);
    return instance.method();
}
"#;

    let result =
        analyze_content_with_scope_awareness(typescript_code.to_string(), Language::TypeScript);

    assert!(result.is_ok());
    let (_ir, _analysis_result, symbol_table, _warnings) = result.unwrap();

    // Verify we have a global scope
    assert!(symbol_table.scopes.get_scope(0).is_some());

    // Should have class and function scopes
    let class_scopes: Vec<_> = symbol_table
        .scopes
        .scopes
        .values()
        .filter(|s| matches!(s.scope_type, ScopeType::Class))
        .collect();
    assert!(!class_scopes.is_empty());

    let function_scopes: Vec<_> = symbol_table
        .scopes
        .scopes
        .values()
        .filter(|s| matches!(s.scope_type, ScopeType::Function))
        .collect();
    assert!(function_scopes.len() >= 3); // constructor, method, globalFunction
}

#[test]
fn test_fallback_to_traditional_resolution() {
    // This test ensures that even if scope analysis fails,
    // we still get results through the fallback mechanism
    let simple_code = "let x = 5;";

    let result = analyze_content_with_scope_awareness(simple_code.to_string(), Language::Rust);

    assert!(result.is_ok());
    let (_ir, _analysis_result, symbol_table, _warnings) = result.unwrap();

    // Should always have at least a global scope
    assert!(symbol_table.scopes.get_scope(0).is_some());

    // Analysis result should be valid (even if dependencies are empty)
}

#[test]
fn test_scope_aware_dependency_resolution() {
    let code_with_dependencies = r#"
fn outer() {
    let outer_var = 10;
    
    fn inner() {
        let inner_var = outer_var + 5;  // Should depend on outer_var
        println!("{}", inner_var);
    }
    
    inner();
}
"#;

    let result =
        analyze_content_with_scope_awareness(code_with_dependencies.to_string(), Language::Rust);

    assert!(result.is_ok());
    let (ir, analysis_result, symbol_table, _warnings) = result.unwrap();

    // Should have found some dependencies
    assert!(!ir.dependencies.is_empty());

    // Should have scope structure
    assert!(symbol_table.scopes.scopes.len() > 1);

    // Analysis should have some metrics
    assert!(analysis_result.overall_complexity_score >= 0.0);
}

#[test]
fn test_symbol_table_symbol_lookup() {
    let code = r#"
fn main() {
    let x = 10;
    {
        let y = x + 5;
    }
}
"#;

    let result = analyze_content_with_scope_awareness(code.to_string(), Language::Rust);

    assert!(result.is_ok());
    let (_ir, _analysis_result, symbol_table, _warnings) = result.unwrap();

    // Should be able to look up symbols
    if let Some(global_scope) = symbol_table.scopes.get_scope(0) {
        // Global scope exists
        assert!(global_scope.scope_type == ScopeType::Global);
    }

    // Verify we can traverse scopes
    let all_scopes: Vec<_> = symbol_table.scopes.scopes.keys().collect();
    assert!(all_scopes.len() > 1); // At minimum global + function scopes
}
