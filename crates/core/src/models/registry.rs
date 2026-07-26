//! What a traversal collected, before anything is resolved.
//!
//! Definitions are indexed by name because resolution asks for candidates by name; usages are kept
//! in order because a later one may resolve differently from an earlier one of the same name.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::definition::ScopeId;
use super::{Definition, Usage};

/// Registry for managing definitions with single responsibility
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DefinitionRegistry {
    definitions: HashMap<String, Vec<Definition>>,
}

impl DefinitionRegistry {
    pub fn new() -> Self {
        Self {
            definitions: HashMap::new(),
        }
    }

    pub fn add_definition(&mut self, name: String, definition: Definition) {
        self.definitions.entry(name).or_default().push(definition);
    }

    pub fn get_all_definitions(&self) -> &HashMap<String, Vec<Definition>> {
        &self.definitions
    }
}

impl Default for DefinitionRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Registry for managing usages with single responsibility
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageRegistry {
    usages: Vec<Usage>,
    scope_indexed_usages: HashMap<ScopeId, Vec<usize>>, // Optional: for efficient lookup
}

impl UsageRegistry {
    pub fn new() -> Self {
        Self {
            usages: Vec::new(),
            scope_indexed_usages: HashMap::new(),
        }
    }

    pub fn add_usage(&mut self, usage: Usage) {
        let usage_index = self.usages.len();
        if let Some(scope_id) = usage.get_scope_id() {
            self.scope_indexed_usages
                .entry(scope_id)
                .or_default()
                .push(usage_index);
        }
        self.usages.push(usage);
    }

    pub fn get_all_usages(&self) -> &Vec<Usage> {
        &self.usages
    }
}

impl Default for UsageRegistry {
    fn default() -> Self {
        Self::new()
    }
}
