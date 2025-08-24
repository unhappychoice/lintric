use lintric_core::models::{Dependency, DependencyType};

#[test]
fn test_dependency_creation() {
    let dependency = Dependency {
        source_line: 1,
        target_line: 5,
        symbol: "function_call".to_string(),
        dependency_type: DependencyType::FunctionCall,
        context: Some("test context".to_string()),
    };

    assert_eq!(dependency.source_line, 1);
    assert_eq!(dependency.target_line, 5);
    assert_eq!(dependency.symbol, "function_call");
    assert_eq!(dependency.dependency_type, DependencyType::FunctionCall);
    assert_eq!(dependency.context, Some("test context".to_string()));
}

#[test]
fn test_dependency_types() {
    let function_dep = Dependency {
        source_line: 1,
        target_line: 5,
        symbol: "caller".to_string(),
        dependency_type: DependencyType::FunctionCall,
        context: None,
    };

    let variable_dep = Dependency {
        source_line: 2,
        target_line: 6,
        symbol: "variable".to_string(),
        dependency_type: DependencyType::VariableUse,
        context: None,
    };

    let type_dep = Dependency {
        source_line: 3,
        target_line: 7,
        symbol: "Type".to_string(),
        dependency_type: DependencyType::TypeReference,
        context: None,
    };

    assert!(matches!(
        function_dep.dependency_type,
        DependencyType::FunctionCall
    ));
    assert!(matches!(
        variable_dep.dependency_type,
        DependencyType::VariableUse
    ));
    assert!(matches!(
        type_dep.dependency_type,
        DependencyType::TypeReference
    ));
}

#[test]
fn test_dependency_clone() {
    let original = Dependency {
        source_line: 1,
        target_line: 5,
        symbol: "test_symbol".to_string(),
        dependency_type: DependencyType::VariableUse,
        context: Some("context".to_string()),
    };

    let cloned = original.clone();

    assert_eq!(original.source_line, cloned.source_line);
    assert_eq!(original.target_line, cloned.target_line);
    assert_eq!(original.symbol, cloned.symbol);
    assert_eq!(original.dependency_type, cloned.dependency_type);
    assert_eq!(original.context, cloned.context);
}

#[test]
fn test_dependency_debug() {
    let dependency = Dependency {
        source_line: 10,
        target_line: 20,
        symbol: "debug_test".to_string(),
        dependency_type: DependencyType::Import,
        context: None,
    };

    let debug_str = format!("{:?}", dependency);
    assert!(debug_str.contains("debug_test"));
    assert!(debug_str.contains("10"));
    assert!(debug_str.contains("20"));
}
