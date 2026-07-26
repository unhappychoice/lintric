//! Everything one traversal of a file produced.

use serde::{Deserialize, Serialize};

use super::registry::{DefinitionRegistry, UsageRegistry};
use super::scope::ScopeTree;

/// Coordinated context for all code analysis data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeAnalysisContext {
    pub definitions: DefinitionRegistry,
    pub usages: UsageRegistry,
    pub scopes: ScopeTree,
}

impl CodeAnalysisContext {
    pub fn new() -> Self {
        Self {
            definitions: DefinitionRegistry::new(),
            usages: UsageRegistry::new(),
            scopes: ScopeTree::new(),
        }
    }
}

impl Default for CodeAnalysisContext {
    fn default() -> Self {
        Self::new()
    }
}
