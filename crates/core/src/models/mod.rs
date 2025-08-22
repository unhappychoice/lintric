pub mod definition;
pub mod dependency;
pub mod intermediate_representation;
pub mod language;
pub mod metrics;
pub mod position;
pub mod scope;
pub mod usage;

// Re-export all public types for convenient access
pub use definition::{Definition, DefinitionType};
pub use dependency::{Dependency, DependencyType};
pub use intermediate_representation::{AnalysisMetadata, IntermediateRepresentation};
pub use language::Language;
pub use metrics::{AnalysisResult, LineMetrics, OverallAnalysisReport};
pub use position::Position;
pub use scope::{Accessibility, Scope, ScopeId, ScopeTree, ScopeType, SymbolEntry, SymbolTable};
pub use usage::{Usage, UsageKind};
