pub mod definition;
pub mod dependency;
pub mod intermediate_representation;
pub mod language;
pub mod metrics;
pub mod module;
pub mod position;
pub mod scope;
pub mod type_system;
pub mod usage;

// Re-export all public types for convenient access
pub use definition::{Accessibility, Definition, DefinitionType, ScopeId};
pub use dependency::{Dependency, DependencyType};
pub use intermediate_representation::{AnalysisMetadata, IntermediateRepresentation};
pub use language::Language;
pub use metrics::{AnalysisResult, LineMetrics, OverallAnalysisReport};
pub use module::{ImportInfo, ImportType, Module, ModuleId, ModuleTree, Visibility};
pub use position::Position;
pub use scope::{Scope, ScopeTree, ScopeType, SymbolEntry, SymbolTable};
pub use type_system::{InferenceContext, Type};
pub use usage::{Usage, UsageKind};
