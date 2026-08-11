use serde::{Deserialize, Serialize};
use std::fmt;

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
    /// A reference to a variant of an enum, as in `Direction::Left`.
    EnumVariantReference,
    /// A method implementation satisfying the declaration of the trait it implements.
    TraitImplementation,
    Other(String),
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Dependency {
    pub source_line: usize,
    pub target_line: usize,
    pub symbol: String,
    pub dependency_type: DependencyType,
    pub context: Option<String>,
}

impl fmt::Debug for Dependency {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Dependency {{ source_line: {}, target_line: {}, symbol: {:?}, dependency_type: {:?}, context: {:?} }}",
               self.source_line, self.target_line, self.symbol, self.dependency_type, self.context)
    }
}
