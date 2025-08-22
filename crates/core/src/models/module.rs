use crate::models::{Definition, Position};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub type ModuleId = usize;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ModuleTree {
    pub root_module: ModuleId,
    pub modules: HashMap<ModuleId, Module>,
    pub module_paths: HashMap<String, ModuleId>,
    pub next_id: ModuleId,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Module {
    pub id: ModuleId,
    pub name: String,
    pub parent: Option<ModuleId>,
    pub children: Vec<ModuleId>,
    pub exports: HashMap<String, Definition>,
    pub imports: Vec<ImportInfo>,
    pub visibility: Visibility,
    pub file_path: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ImportInfo {
    pub imported_symbol: String,
    pub source_module: String,
    pub alias: Option<String>,
    pub import_type: ImportType,
    pub visibility: Visibility,
    pub position: Position,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ImportType {
    Named(String),
    Wildcard,
    Default,
    Module,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Visibility {
    Public,
    Private,
    PubCrate,
    PubSuper,
    PubIn(String),
}

impl ModuleTree {
    pub fn new() -> Self {
        let mut tree = ModuleTree {
            root_module: 0,
            modules: HashMap::new(),
            module_paths: HashMap::new(),
            next_id: 1,
        };

        // Create root module
        let root = Module {
            id: 0,
            name: "crate".to_string(),
            parent: None,
            children: Vec::new(),
            exports: HashMap::new(),
            imports: Vec::new(),
            visibility: Visibility::Public,
            file_path: None,
        };

        tree.modules.insert(0, root);
        tree.module_paths.insert("crate".to_string(), 0);
        tree
    }

    pub fn add_module(
        &mut self,
        name: String,
        parent_id: Option<ModuleId>,
        file_path: Option<String>,
    ) -> ModuleId {
        let id = self.next_id;
        self.next_id += 1;

        let module = Module {
            id,
            name: name.clone(),
            parent: parent_id,
            children: Vec::new(),
            exports: HashMap::new(),
            imports: Vec::new(),
            visibility: Visibility::Public,
            file_path,
        };

        // Add to parent's children
        if let Some(parent_id) = parent_id {
            if let Some(parent) = self.modules.get_mut(&parent_id) {
                parent.children.push(id);
            }
        }

        // Create module path
        let module_path = self.build_module_path(parent_id, &name);
        self.module_paths.insert(module_path, id);
        self.modules.insert(id, module);

        id
    }

    fn build_module_path(&self, parent_id: Option<ModuleId>, name: &str) -> String {
        match parent_id {
            Some(parent_id) if parent_id == self.root_module => name.to_string(),
            Some(parent_id) => {
                if let Some(_parent) = self.modules.get(&parent_id) {
                    let parent_path = self.get_module_path(parent_id).unwrap_or_default();
                    if parent_path == "crate" {
                        name.to_string()
                    } else {
                        format!("{}::{}", parent_path, name)
                    }
                } else {
                    name.to_string()
                }
            }
            None => name.to_string(),
        }
    }

    pub fn get_module_path(&self, module_id: ModuleId) -> Option<String> {
        self.module_paths
            .iter()
            .find(|(_, &id)| id == module_id)
            .map(|(path, _)| path.clone())
    }

    pub fn find_module_by_path(&self, path: &str) -> Option<ModuleId> {
        self.module_paths.get(path).copied()
    }

    pub fn add_import(&mut self, module_id: ModuleId, import: ImportInfo) {
        if let Some(module) = self.modules.get_mut(&module_id) {
            module.imports.push(import);
        }
    }

    pub fn add_export(&mut self, module_id: ModuleId, symbol: String, definition: Definition) {
        if let Some(module) = self.modules.get_mut(&module_id) {
            module.exports.insert(symbol, definition);
        }
    }
}

impl Default for ModuleTree {
    fn default() -> Self {
        Self::new()
    }
}

impl Visibility {
    pub fn is_public(&self) -> bool {
        matches!(self, Visibility::Public)
    }

    pub fn is_private(&self) -> bool {
        matches!(self, Visibility::Private)
    }

    pub fn is_crate_visible(&self) -> bool {
        matches!(self, Visibility::PubCrate)
    }

    pub fn is_super_visible(&self) -> bool {
        matches!(self, Visibility::PubSuper)
    }
}
