pub mod language_plugin;
pub mod plugins;
pub mod test_helpers;

use serde::Deserialize;
use std::collections::HashSet;

#[derive(Debug, Deserialize)]
pub struct NodeType {
    #[serde(rename = "type")]
    pub node_type: String,
}

#[derive(Debug)]
pub struct GenerationContext {
    pub used_names: HashSet<String>,
    pub nesting_level: usize,
    pub generated_types: HashSet<String>,
    pub excluded: HashSet<String>,
}

impl Default for GenerationContext {
    fn default() -> Self {
        Self::new()
    }
}

impl GenerationContext {
    pub fn new() -> Self {
        Self {
            used_names: HashSet::new(),
            nesting_level: 0,
            generated_types: HashSet::new(),
            excluded: HashSet::new(),
        }
    }

    pub fn get_unique_name(&mut self, base: &str) -> String {
        let mut counter = 0;
        let mut name = base.to_string();
        while self.used_names.contains(&name) {
            counter += 1;
            name = format!("{}{}", base, counter);
        }
        self.used_names.insert(name.clone());
        name
    }

    pub fn mark_excluded(&mut self, node_type: &str) -> Option<String> {
        self.excluded.insert(node_type.to_string());
        None
    }
}
