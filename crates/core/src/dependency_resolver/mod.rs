pub mod associated_type_resolver;
pub mod base_resolver;
pub mod constraint_solver;
pub mod lifetime_resolver;
pub mod main_resolver;
pub mod method_resolver;
pub mod module_resolver;
pub mod nested_scope_resolver;
pub mod resolution_candidate;
pub mod scope_aware_resolver;
pub mod scope_builder;
pub mod scope_integration;
pub mod trait_database;
pub mod type_system;

pub use associated_type_resolver::AssociatedTypeResolver;
pub use base_resolver::DependencyResolver as DependencyResolverTrait;
pub use constraint_solver::ConstraintSolver;
pub use lifetime_resolver::LifetimeResolver;
pub use main_resolver::DependencyResolver;
pub use method_resolver::{MethodResolutionResult, MethodResolver};
pub use module_resolver::{ImportResolver, ModuleResolver, VisibilityChecker};
pub use nested_scope_resolver::{ClosureAnalyzer, NestedScopeResolver, ScopeChainWalker};
pub use resolution_candidate::{ResolutionCandidate, ShadowingWarning};
pub use scope_aware_resolver::{
    DefaultScopeAwareResolver, ScopeAwareDependencyResolver, ScopeValidator,
};
pub use scope_builder::{ScopeAwareDefinitionCollector, ScopeBuilder};
pub use scope_integration::{create_scope_integrated_resolver, ScopeIntegratedResolver};
pub use trait_database::{TraitDatabase, TraitDefinition, TraitImplementation};
pub use type_system::{
    AssociatedType, Constraint, ConstraintError, GenericTypeResolver, LifetimeBound, LifetimeId,
    LifetimeParam, LifetimeScope, TraitBound, TraitId, TypeParam, TypeSubstitution, TypeVarId,
    Variance,
};
