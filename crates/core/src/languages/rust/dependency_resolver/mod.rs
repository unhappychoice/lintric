pub mod associated_type_resolver;
pub mod constraint_solver;
pub mod impl_collector;
pub mod lifetime_resolver;
pub mod method_resolver;
pub mod module_resolver;
pub mod nested_scope_resolver;
pub mod resolution_candidate;
pub mod rust_dependency_resolver;
pub mod trait_database;
pub mod type_system;

pub use associated_type_resolver::AssociatedTypeResolver;
pub use constraint_solver::ConstraintSolver;
pub use impl_collector::RustImplCollector;
pub use lifetime_resolver::LifetimeResolver;
pub use method_resolver::{MethodResolutionResult, MethodResolver};
pub use module_resolver::{ImportResolver, ModuleResolver, VisibilityChecker};
pub use nested_scope_resolver::{ClosureAnalyzer, NestedScopeResolver, ScopeChainWalker};
pub use resolution_candidate::{ResolutionCandidate, ShadowingWarning};
pub use rust_dependency_resolver::RustDependencyResolver;
pub use trait_database::{TraitDatabase, TraitDefinition, TraitImplementation};
pub use type_system::{
    AssociatedType, Constraint, ConstraintError, GenericTypeResolver, LifetimeBound, LifetimeId,
    LifetimeParam, LifetimeScope, TraitBound, TraitId, TypeParam, TypeSubstitution, TypeVarId,
    Variance,
};
