use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum DependencyType {
    FunctionCall,
    VariableUse,
    Import,
    StructFieldAccess,
    TypeReference,
    ModuleReference,
    MacroInvocation,
    MacroVariable,
    Other(String),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Dependency {
    pub source_line: usize,
    pub target_line: usize,
    pub symbol: String,
    pub dependency_type: DependencyType,
    pub context: Option<String>,
}

impl Dependency {
    pub fn new_with_scope(
        source_position: crate::models::Position,
        target_position: crate::models::Position,
        usage: crate::models::Usage,
        _definition: crate::models::Definition,
    ) -> Self {
        Self {
            source_line: source_position.start_line,
            target_line: target_position.start_line,
            symbol: usage.name,
            dependency_type: DependencyType::VariableUse, // Default, can be improved later
            context: None,
        }
    }
}
