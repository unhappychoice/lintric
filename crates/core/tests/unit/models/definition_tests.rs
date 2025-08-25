use lintric_core::models::{Definition, DefinitionType, Position};

#[test]
fn test_definition_creation() {
    let position = Position {
        start_line: 1,
        start_column: 1,
        end_line: 1,
        end_column: 10,
    };

    let definition = Definition {
        name: "test_function".to_string(),
        position: position.clone(),
        definition_type: DefinitionType::FunctionDefinition,
        scope_id: None,
        accessibility: None,
        is_hoisted: None,
    };

    assert_eq!(definition.name, "test_function");
    assert_eq!(definition.position.start_line, position.start_line);
    assert_eq!(
        definition.definition_type,
        DefinitionType::FunctionDefinition
    );
}

#[test]
fn test_definition_types() {
    let position = Position {
        start_line: 1,
        start_column: 1,
        end_line: 1,
        end_column: 10,
    };

    let function_def = Definition {
        name: "func".to_string(),
        position: position.clone(),
        definition_type: DefinitionType::FunctionDefinition,
        scope_id: None,
        accessibility: None,
        is_hoisted: None,
    };

    let variable_def = Definition {
        name: "var".to_string(),
        position: position.clone(),
        definition_type: DefinitionType::VariableDefinition,
        scope_id: None,
        accessibility: None,
        is_hoisted: None,
    };

    let struct_def = Definition {
        name: "Struct".to_string(),
        position: position.clone(),
        definition_type: DefinitionType::StructDefinition,
        scope_id: None,
        accessibility: None,
        is_hoisted: None,
    };

    assert!(matches!(
        function_def.definition_type,
        DefinitionType::FunctionDefinition
    ));
    assert!(matches!(
        variable_def.definition_type,
        DefinitionType::VariableDefinition
    ));
    assert!(matches!(
        struct_def.definition_type,
        DefinitionType::StructDefinition
    ));
}

#[test]
fn test_definition_new_simple() {
    let position = Position {
        start_line: 5,
        start_column: 10,
        end_line: 5,
        end_column: 20,
    };

    let definition = Definition::new_simple(
        "test_var".to_string(),
        DefinitionType::VariableDefinition,
        position.clone(),
    );

    assert_eq!(definition.name, "test_var");
    assert_eq!(definition.position.start_line, position.start_line);
    assert_eq!(
        definition.definition_type,
        DefinitionType::VariableDefinition
    );
}

#[test]
fn test_definition_equality() {
    let position = Position {
        start_line: 1,
        start_column: 1,
        end_line: 1,
        end_column: 10,
    };

    let def1 = Definition {
        name: "test".to_string(),
        position: position.clone(),
        definition_type: DefinitionType::FunctionDefinition,
        scope_id: None,
        accessibility: None,
        is_hoisted: None,
    };

    let def2 = Definition {
        name: "test".to_string(),
        position: position.clone(),
        definition_type: DefinitionType::FunctionDefinition,
        scope_id: None,
        accessibility: None,
        is_hoisted: None,
    };

    let def3 = Definition {
        name: "different".to_string(),
        position: position.clone(),
        definition_type: DefinitionType::FunctionDefinition,
        scope_id: None,
        accessibility: None,
        is_hoisted: None,
    };

    assert_eq!(def1, def2);
    assert_ne!(def1, def3);
}
