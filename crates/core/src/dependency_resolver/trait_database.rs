use super::type_system::{AssociatedType, TraitId, TypeParam};
use crate::models::{Definition, Type};
use std::collections::HashMap;

/// Database for trait definitions and implementations
#[derive(Debug, Clone)]
pub struct TraitDatabase {
    pub traits: HashMap<TraitId, TraitDefinition>,
    pub implementations: HashMap<TraitId, Vec<TraitImplementation>>,
}

#[derive(Debug, Clone)]
pub struct TraitDefinition {
    pub id: TraitId,
    pub name: String,
    pub type_params: Vec<TypeParam>,
    pub associated_types: Vec<AssociatedType>,
    pub methods: Vec<Definition>,
}

#[derive(Debug, Clone)]
pub struct TraitImplementation {
    pub trait_id: TraitId,
    pub target_type: Type,
    pub type_args: Vec<Type>,
    pub associated_type_mappings: HashMap<String, Type>,
}

impl Default for TraitDatabase {
    fn default() -> Self {
        Self::new()
    }
}

impl TraitDatabase {
    pub fn new() -> Self {
        Self {
            traits: HashMap::new(),
            implementations: HashMap::new(),
        }
    }

    pub fn add_trait(&mut self, trait_def: TraitDefinition) {
        self.traits.insert(trait_def.id, trait_def);
    }

    pub fn add_implementation(&mut self, trait_id: TraitId, implementation: TraitImplementation) {
        self.implementations
            .entry(trait_id)
            .or_default()
            .push(implementation);
    }
}
