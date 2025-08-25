use crate::models::{ScopeId, Type};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub struct TypeParam {
    pub name: String,
    pub bounds: Vec<TraitBound>,
    pub default: Option<Type>,
    pub variance: Variance,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LifetimeParam {
    pub name: String,
    pub bounds: Vec<LifetimeBound>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Variance {
    Covariant,
    Contravariant,
    Invariant,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TraitBound {
    pub trait_name: String,
    pub type_args: Vec<Type>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LifetimeBound {
    pub lifetime: LifetimeId,
    pub outlives: LifetimeId,
}

#[derive(Debug, Clone, PartialEq)]
pub enum LifetimeId {
    Named(String),
    Anonymous(u32),
    Static,
    Infer(u32),
}

pub type TypeVarId = u32;
pub type TraitId = u32;

#[derive(Debug, Clone)]
pub enum Constraint {
    TraitBound {
        type_var: TypeVarId,
        trait_def: TraitId,
    },
    Equality {
        left: Type,
        right: Type,
    },
    Lifetime {
        lifetime: LifetimeId,
        outlives: LifetimeId,
    },
    Associated {
        type_var: TypeVarId,
        trait_def: TraitId,
        assoc_type: String,
    },
}

#[derive(Debug, Clone)]
pub struct TypeSubstitution {
    pub type_vars: HashMap<TypeVarId, Type>,
    pub lifetimes: HashMap<LifetimeId, LifetimeId>,
}

#[derive(Debug, Clone)]
pub struct AssociatedType {
    pub name: String,
    pub trait_def: TraitId,
    pub bounds: Vec<TraitBound>,
    pub default: Option<Type>,
}

#[derive(Debug, Clone)]
pub struct LifetimeScope {
    pub lifetimes: HashMap<String, LifetimeId>,
    pub parent: Option<ScopeId>,
}

/// Generic Type Resolver for handling complex generic scenarios
#[derive(Debug, Clone)]
pub struct GenericTypeResolver {
    pub type_parameters: HashMap<ScopeId, Vec<TypeParam>>,
    pub lifetime_parameters: HashMap<ScopeId, Vec<LifetimeParam>>,
    pub constraint_solver: super::constraint_solver::ConstraintSolver,
}

impl Default for GenericTypeResolver {
    fn default() -> Self {
        Self::new()
    }
}

impl GenericTypeResolver {
    pub fn new() -> Self {
        Self {
            type_parameters: HashMap::new(),
            lifetime_parameters: HashMap::new(),
            constraint_solver: super::constraint_solver::ConstraintSolver::new(),
        }
    }

    pub fn add_type_parameters(&mut self, scope_id: ScopeId, params: Vec<TypeParam>) {
        self.type_parameters.insert(scope_id, params);
    }

    pub fn add_lifetime_parameters(&mut self, scope_id: ScopeId, params: Vec<LifetimeParam>) {
        self.lifetime_parameters.insert(scope_id, params);
    }

    pub fn resolve_generic_type(&self, type_name: &str, scope_id: ScopeId) -> Option<TypeParam> {
        if let Some(params) = self.type_parameters.get(&scope_id) {
            params.iter().find(|p| p.name == type_name).cloned()
        } else {
            None
        }
    }
}

#[derive(Debug)]
pub enum ConstraintError {
    UnsatisfiedTraitBound(String),
    LifetimeError(String),
    TypeMismatch(String),
    RecursiveConstraint(String),
}
