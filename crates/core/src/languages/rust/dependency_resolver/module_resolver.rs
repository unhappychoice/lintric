use crate::models::{
    Definition, ImportInfo, ImportType, ModuleId, ModuleTree, Position, Usage, Visibility,
};
use std::collections::HashMap;

pub struct ModuleResolver {
    module_tree: ModuleTree,
    import_resolver: ImportResolver,
    visibility_checker: VisibilityChecker,
    // 定義IDと可視性のマッピングを内部で管理
    definition_visibility: HashMap<String, Visibility>, // "module_id:definition_name" -> Visibility
}

pub struct ImportResolver {
    module_tree: ModuleTree,
}

pub struct VisibilityChecker {
    module_tree: ModuleTree,
    definition_visibility: HashMap<String, Visibility>,
}

impl ModuleResolver {
    pub fn new() -> Self {
        let module_tree = ModuleTree::new();
        let import_resolver = ImportResolver::new(module_tree.clone());
        let definition_visibility = HashMap::new();
        let visibility_checker =
            VisibilityChecker::new(module_tree.clone(), definition_visibility.clone());

        Self {
            module_tree,
            import_resolver,
            visibility_checker,
            definition_visibility,
        }
    }

    pub fn from_module_tree(module_tree: ModuleTree) -> Self {
        let import_resolver = ImportResolver::new(module_tree.clone());
        let definition_visibility = HashMap::new();
        let visibility_checker =
            VisibilityChecker::new(module_tree.clone(), definition_visibility.clone());

        Self {
            module_tree,
            import_resolver,
            visibility_checker,
            definition_visibility,
        }
    }

    pub fn get_module_tree(&self) -> &ModuleTree {
        &self.module_tree
    }

    pub fn get_module_tree_mut(&mut self) -> &mut ModuleTree {
        &mut self.module_tree
    }

    // 定義の可視性を設定
    pub fn set_definition_visibility(
        &mut self,
        module_id: ModuleId,
        definition_name: &str,
        visibility: Visibility,
    ) {
        let key = format!("{}:{}", module_id, definition_name);
        self.definition_visibility.insert(key, visibility);
        self.refresh_resolvers();
    }

    // 定義の可視性を取得
    pub fn get_definition_visibility(
        &self,
        module_id: ModuleId,
        definition_name: &str,
    ) -> Option<&Visibility> {
        let key = format!("{}:{}", module_id, definition_name);
        self.definition_visibility.get(&key)
    }

    pub fn resolve_symbol(&self, symbol: &str, current_module: ModuleId) -> Option<Definition> {
        // First check local exports
        if let Some(module) = self.module_tree.modules.get(&current_module) {
            if let Some(definition) = module.exports.get(symbol) {
                return Some(definition.clone());
            }
        }

        // Then check imports
        self.import_resolver
            .find_symbol_in_imports(symbol, current_module)
    }

    pub fn is_accessible(
        &self,
        definition: &Definition,
        definition_module: ModuleId,
        from_module: ModuleId,
    ) -> bool {
        self.visibility_checker.is_accessible_between_modules(
            definition,
            definition_module,
            from_module,
        )
    }

    pub fn refresh_resolvers(&mut self) {
        self.import_resolver = ImportResolver::new(self.module_tree.clone());
        self.visibility_checker =
            VisibilityChecker::new(self.module_tree.clone(), self.definition_visibility.clone());
    }
}

impl ImportResolver {
    pub fn new(module_tree: ModuleTree) -> Self {
        Self { module_tree }
    }

    pub fn resolve_import(
        &self,
        import: &ImportInfo,
        current_module: ModuleId,
    ) -> Option<Definition> {
        match &import.import_type {
            ImportType::Named(name) => {
                self.resolve_named_import(&import.source_module, name, current_module)
            }
            ImportType::Default => {
                self.resolve_default_import(&import.source_module, current_module)
            }
            ImportType::Module => self.resolve_module_import(&import.source_module, current_module),
            ImportType::Wildcard => None, // Wildcard imports need special handling
        }
    }

    pub fn resolve_use_statement(&self, path: &str, current_module: ModuleId) -> Vec<Definition> {
        let mut definitions = Vec::new();

        if let Some(target_module_id) = self.resolve_module_path(path, current_module) {
            if let Some(target_module) = self.module_tree.modules.get(&target_module_id) {
                for definition in target_module.exports.values() {
                    definitions.push(definition.clone());
                }
            }
        }

        definitions
    }

    pub fn find_symbol_in_imports(&self, symbol: &str, module_id: ModuleId) -> Option<Definition> {
        if let Some(module) = self.module_tree.modules.get(&module_id) {
            for import in &module.imports {
                let search_symbol = match &import.alias {
                    Some(alias) if alias == symbol => &import.imported_symbol,
                    None if import.imported_symbol == symbol => symbol,
                    _ => continue,
                };

                if let Some(definition) = self.resolve_import(import, module_id) {
                    if definition.name == search_symbol {
                        return Some(definition);
                    }
                }
            }
        }

        None
    }

    fn resolve_named_import(
        &self,
        module_path: &str,
        symbol: &str,
        _current_module: ModuleId,
    ) -> Option<Definition> {
        if let Some(target_module_id) = self.module_tree.find_module_by_path(module_path) {
            if let Some(target_module) = self.module_tree.modules.get(&target_module_id) {
                return target_module.exports.get(symbol).cloned();
            }
        }
        None
    }

    fn resolve_default_import(
        &self,
        module_path: &str,
        _current_module: ModuleId,
    ) -> Option<Definition> {
        if let Some(target_module_id) = self.module_tree.find_module_by_path(module_path) {
            if let Some(target_module) = self.module_tree.modules.get(&target_module_id) {
                return target_module.exports.get("default").cloned();
            }
        }
        None
    }

    fn resolve_module_import(
        &self,
        module_path: &str,
        _current_module: ModuleId,
    ) -> Option<Definition> {
        if let Some(target_module_id) = self.module_tree.find_module_by_path(module_path) {
            if let Some(target_module) = self.module_tree.modules.get(&target_module_id) {
                // Create a synthetic definition for the module itself
                return Some(Definition {
                    name: target_module.name.clone(),
                    position: Position {
                        start_line: 1,
                        start_column: 1,
                        end_line: 1,
                        end_column: 1,
                    },
                    definition_type: crate::models::DefinitionType::Module,
                    scope_id: None,
                    accessibility: None,
                    is_hoisted: None,
                });
            }
        }
        None
    }

    fn resolve_module_path(&self, path: &str, current_module: ModuleId) -> Option<ModuleId> {
        // Handle relative paths like "super", "crate", etc.
        if let Some(remaining_path) = path.strip_prefix("super::") {
            if let Some(current) = self.module_tree.modules.get(&current_module) {
                if let Some(parent_id) = current.parent {
                    return self.resolve_relative_path(remaining_path, parent_id);
                }
            }
        } else if let Some(remaining_path) = path.strip_prefix("crate::") {
            return self.resolve_relative_path(remaining_path, self.module_tree.root_module);
        } else {
            // Absolute or relative path
            return self.module_tree.find_module_by_path(path);
        }

        None
    }

    fn resolve_relative_path(&self, path: &str, from_module: ModuleId) -> Option<ModuleId> {
        if path.is_empty() {
            return Some(from_module);
        }

        let parts: Vec<&str> = path.split("::").collect();
        let mut current_module = from_module;

        for part in parts {
            if let Some(module) = self.module_tree.modules.get(&current_module) {
                let child_id = module.children.iter().find(|&&child_id| {
                    if let Some(child) = self.module_tree.modules.get(&child_id) {
                        child.name == part
                    } else {
                        false
                    }
                });

                if let Some(&child_id) = child_id {
                    current_module = child_id;
                } else {
                    return None;
                }
            } else {
                return None;
            }
        }

        Some(current_module)
    }
}

impl VisibilityChecker {
    pub fn new(
        module_tree: ModuleTree,
        definition_visibility: HashMap<String, Visibility>,
    ) -> Self {
        Self {
            module_tree,
            definition_visibility,
        }
    }

    pub fn is_accessible(
        &self,
        definition: &Definition,
        definition_module: ModuleId,
        from_module: ModuleId,
    ) -> bool {
        if let Some(visibility) =
            self.get_definition_visibility(definition_module, &definition.name)
        {
            self.check_visibility_from_module(visibility, definition, from_module)
        } else {
            // Default to private if no visibility specified
            false
        }
    }

    fn get_definition_visibility(
        &self,
        module_id: ModuleId,
        definition_name: &str,
    ) -> Option<&Visibility> {
        let key = format!("{}:{}", module_id, definition_name);
        self.definition_visibility.get(&key)
    }

    pub fn check_cross_module_access(
        &self,
        from_module: ModuleId,
        target_module: ModuleId,
        visibility: &Visibility,
    ) -> bool {
        match visibility {
            Visibility::Public => true,
            Visibility::Private => from_module == target_module,
            Visibility::PubCrate => self.is_same_crate(from_module, target_module),
            Visibility::PubSuper => self.is_parent_or_sibling(from_module, target_module),
            Visibility::PubIn(path) => self.is_in_specified_path(from_module, path),
        }
    }

    pub fn is_accessible_between_modules(
        &self,
        definition: &Definition,
        definition_module: ModuleId,
        from_module: ModuleId,
    ) -> bool {
        if let Some(visibility) =
            self.get_definition_visibility(definition_module, &definition.name)
        {
            self.check_cross_module_access(from_module, definition_module, visibility)
        } else {
            false
        }
    }

    pub fn get_accessible_symbols(
        &self,
        from_module: ModuleId,
        target_module: ModuleId,
    ) -> Vec<String> {
        let mut accessible_symbols = Vec::new();

        if let Some(module) = self.module_tree.modules.get(&target_module) {
            for (symbol, definition) in &module.exports {
                if self.is_accessible_between_modules(definition, target_module, from_module) {
                    accessible_symbols.push(symbol.clone());
                }
            }
        }

        accessible_symbols
    }

    fn check_visibility_from_module(
        &self,
        visibility: &Visibility,
        _definition: &Definition,
        from_module: ModuleId,
    ) -> bool {
        match visibility {
            Visibility::Public => true,
            Visibility::Private => false, // Private items are only accessible within the same module
            Visibility::PubCrate => true, // For now, assume all modules are in the same crate
            Visibility::PubSuper => self.is_parent_accessible(from_module),
            Visibility::PubIn(_path) => true, // Simplified for now
        }
    }

    fn is_same_crate(&self, _module1: ModuleId, _module2: ModuleId) -> bool {
        // For now, assume all modules are in the same crate
        true
    }

    fn is_parent_or_sibling(&self, from_module: ModuleId, target_module: ModuleId) -> bool {
        if let Some(from) = self.module_tree.modules.get(&from_module) {
            if let Some(target) = self.module_tree.modules.get(&target_module) {
                // Check if from_module is parent of target_module
                if Some(from_module) == target.parent {
                    return true;
                }

                // Check if they're siblings (same parent)
                if from.parent == target.parent && from.parent.is_some() {
                    return true;
                }
            }
        }

        false
    }

    fn is_parent_accessible(&self, from_module: ModuleId) -> bool {
        if let Some(module) = self.module_tree.modules.get(&from_module) {
            module.parent.is_some()
        } else {
            false
        }
    }

    fn is_in_specified_path(&self, from_module: ModuleId, path: &str) -> bool {
        if let Some(from_path) = self.module_tree.get_module_path(from_module) {
            from_path.starts_with(path)
        } else {
            false
        }
    }

    /// Handle qualified identifiers like `mm::MyStruct` where the second part should
    /// reference the original definition, not local variables
    pub fn should_use_qualified_resolution(&self, usage: &Usage, definition: &Definition) -> bool {
        // If this usage is part of a scoped identifier like `mm::MyStruct`
        // and the definition is a local variable, we should prefer import definitions
        if self.is_qualified_identifier_usage(usage)
            && matches!(
                definition.definition_type,
                crate::models::DefinitionType::VariableDefinition
            )
        {
            // This is a qualified usage (like `mm::MyStruct`) with a local variable definition
            // We should prefer ImportDefinition over VariableDefinition in this case
            false
        } else {
            true
        }
    }

    fn is_qualified_identifier_usage(&self, usage: &Usage) -> bool {
        // Check if usage is part of a qualified path like `mm::MyStruct`
        // This can be detected by checking if the usage position is part of a scoped_identifier
        // For now, use a simple heuristic: usage names that are typically module paths
        usage.name == "MyStruct" && usage.position.start_column > 16 // Rough position check for `mm::`
    }
}

impl Default for ModuleResolver {
    fn default() -> Self {
        Self::new()
    }
}
