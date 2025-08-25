use crate::models::{Definition, ScopeId};

#[derive(Debug, Clone)]
pub struct ShadowingWarning {
    pub message: String,
    pub shadowing_definition: Definition,
    pub shadowed_definition: Definition,
}

#[derive(Debug, Clone)]
pub struct ResolutionCandidate {
    pub definition: Definition,
    pub scope_distance: usize,
    pub shadowing_level: usize,
    pub priority_score: f64,
}

impl ResolutionCandidate {
    pub fn new(
        definition: Definition,
        _scope_id: ScopeId,
        scope_distance: usize,
        shadowing_level: usize,
    ) -> Self {
        let priority_score = Self::calculate_priority_score(scope_distance, shadowing_level);
        Self {
            definition,
            scope_distance,
            shadowing_level,
            priority_score,
        }
    }

    fn calculate_priority_score(scope_distance: usize, shadowing_level: usize) -> f64 {
        let distance_weight = 1.0 / (scope_distance as f64 + 1.0);
        // For Rust, prioritize definitions within the same function scope
        // Lower shadowing_level means closer to the usage context (higher priority)
        let shadowing_weight = 100.0 / (shadowing_level as f64 + 1.0);
        distance_weight + shadowing_weight
    }
}
