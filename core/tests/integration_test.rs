use lintric_core::analyze_code;

#[test]
fn test_analyze_code_basic() {
    let code = "let a = 1;\nlet b = a + 1;";
    let file_path = "test.rs".to_string();
    let result = analyze_code(code, file_path).unwrap();

    assert_eq!(result.line_metrics.len(), 2);

    let line1_metrics = &result.line_metrics[0];
    assert_eq!(line1_metrics.line_number, 1);
    assert_eq!(line1_metrics.total_dependencies, 0);
    assert_eq!(line1_metrics.dependency_distance_cost, 0.0);
    assert_eq!(line1_metrics.depth, 0);
    assert_eq!(line1_metrics.transitive_dependencies, 0);

    let line2_metrics = &result.line_metrics[1];
    assert_eq!(line2_metrics.line_number, 2);
    assert_eq!(line2_metrics.total_dependencies, 1);
    assert_eq!(line2_metrics.dependency_distance_cost, 0.5);
    assert_eq!(line2_metrics.depth, 1);
    assert_eq!(line2_metrics.transitive_dependencies, 1);

    // Overall complexity score will vary based on weights, so just check it's not zero
    assert!(result.overall_complexity_score > 0.0);
}

#[test]
fn test_analyze_code_typescript() {
    let code = "const x = 1;\nlet y = x + 2;\nfunction foo() { return y; }\n";
    let file_path = "test.ts".to_string();
    let result = analyze_code(code, file_path).unwrap();

    // Expected metrics for TypeScript might differ slightly based on AST structure
    // This is a basic check to ensure it processes without error and produces some metrics
    assert!(result.line_metrics.len() > 0);
    assert!(result.overall_complexity_score > 0.0);

    // More specific assertions can be added here once TypeScript AST behavior is fully understood
    // For example, checking line 2 for dependency on line 1 (y depends on x)
    let line2_metrics = result.line_metrics.iter().find(|m| m.line_number == 2).unwrap();
    assert!(line2_metrics.total_dependencies >= 1);
}

#[test]
fn test_rust_function_call_dependency() {
    let code = "fn add(a: i32, b: i32) -> i32 { a + b }\nfn main() { let x = add(1, 2); }\n";
    let file_path = "test.rs".to_string();
    let result = analyze_code(code, file_path).unwrap();

    // Expect main to depend on add
    let main_line_metrics = result.line_metrics.iter().find(|m| m.line_number == 2).unwrap();
    assert!(main_line_metrics.total_dependencies >= 1);
}

#[test]
fn test_typescript_class_method_dependency() {
    let code = "class MyClass { constructor(public value: number) {} \n greet() { console.log(this.value); } }\nlet instance = new MyClass(10);\ninstance.greet();\n";
    let file_path = "test.ts".to_string();
    let result = analyze_code(code, file_path).unwrap();

    // Expect instance.greet() line to depend on greet method definition
    let greet_call_line_metrics = result.line_metrics.iter().find(|m| m.line_number == 4).unwrap();
    assert!(greet_call_line_metrics.total_dependencies >= 1);
}

#[test]
fn test_rust_struct_field_access_dependency() {
    let code = "struct Point { x: i32, y: i32 }\nfn main() {\n    let p = Point { x: 1, y: 2 };\n    let val = p.x;\n}\n";
    let file_path = "test.rs".to_string();
    let result = analyze_code(code, file_path).unwrap();

    let access_line_metrics = result.line_metrics.iter().find(|m| m.line_number == 4).unwrap();
    assert_eq!(access_line_metrics.total_dependencies, 1);
}

#[test]
fn test_typescript_import_dependency() {
    let code = "import { someFunction } from './module';\nconst result = someFunction();\n";
    let file_path = "test.ts".to_string();
    let result = analyze_code(code, file_path).unwrap();

    let call_line_metrics = result.line_metrics.iter().find(|m| m.line_number == 2).unwrap();
    assert!(call_line_metrics.total_dependencies >= 1);
}
