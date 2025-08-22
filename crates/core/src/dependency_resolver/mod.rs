pub mod associated_type_resolver;
pub mod base_resolver;
pub mod constraint_solver;
pub mod lifetime_resolver;
pub mod resolution_candidate;
pub mod trait_database;
pub mod type_system;

pub use associated_type_resolver::AssociatedTypeResolver;
pub use base_resolver::DependencyResolver;
pub use constraint_solver::ConstraintSolver;
pub use lifetime_resolver::LifetimeResolver;
pub use resolution_candidate::{ResolutionCandidate, ShadowingWarning};
pub use trait_database::{TraitDatabase, TraitDefinition, TraitImplementation};
pub use type_system::{
    AssociatedType, Constraint, ConstraintError, GenericTypeResolver, LifetimeBound,
    LifetimeId, LifetimeParam, LifetimeScope, TraitBound, TraitId, TypeParam, TypeSubstitution,
    TypeVarId, Variance,
};