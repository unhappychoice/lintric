use crate::models::{DefinitionType, ScopeId, ScopeTree, ScopeType};

/// Resolves module paths against the same scope tree that owns import definitions.
pub(super) struct ModuleScopes<'a> {
    scopes: &'a ScopeTree,
}

impl<'a> ModuleScopes<'a> {
    pub(super) fn new(scopes: &'a ScopeTree) -> Self {
        Self { scopes }
    }

    pub(super) fn resolve(&self, path: &[String], usage_scope: ScopeId) -> Option<ScopeId> {
        let (head, rest) = path.split_first()?;
        let module = match head.as_str() {
            "crate" => Some(self.scopes.root),
            "self" => self.enclosing_module(usage_scope),
            "super" => self.parent_module(self.enclosing_module(usage_scope)?),
            name => self.lexical_module(name, usage_scope),
        }?;
        rest.iter()
            .try_fold(module, |scope, name| match name.as_str() {
                "super" => self.parent_module(scope),
                name => self.child_module(name, scope),
            })
    }

    fn enclosing_module(&self, scope: ScopeId) -> Option<ScopeId> {
        std::iter::once(scope)
            .chain(self.scopes.get_parent_scopes(scope))
            .find(|id| {
                self.scopes.get_scope(*id).is_some_and(|scope| {
                    matches!(scope.scope_type, ScopeType::Module | ScopeType::Global)
                })
            })
    }

    fn parent_module(&self, scope: ScopeId) -> Option<ScopeId> {
        self.enclosing_module(self.scopes.get_scope(scope)?.parent?)
    }

    fn lexical_module(&self, name: &str, scope: ScopeId) -> Option<ScopeId> {
        let current = self.scopes.get_scope(scope)?;
        if let Some(module) = self.child_module(name, scope) {
            return Some(module);
        }
        if self.shadows_module(name, scope)
            || matches!(current.scope_type, ScopeType::Module | ScopeType::Global)
        {
            return None;
        }
        self.lexical_module(name, current.parent?)
    }

    fn shadows_module(&self, name: &str, scope: ScopeId) -> bool {
        let Some(current) = self.scopes.get_scope(scope) else {
            return false;
        };
        std::iter::once(scope)
            .chain(current.children.iter().copied())
            .any(|id| {
                self.scopes
                    .get_scope(id)
                    .and_then(|scope| scope.symbols.get(name))
                    .is_some_and(|defs| {
                        defs.iter().any(|def| match def.definition_type {
                            DefinitionType::StructDefinition | DefinitionType::EnumDefinition => {
                                true
                            }
                            DefinitionType::ImportDefinition | DefinitionType::TypeDefinition => {
                                id == scope
                            }
                            _ => false,
                        })
                    })
            })
    }

    fn child_module(&self, name: &str, parent: ScopeId) -> Option<ScopeId> {
        self.scopes
            .get_scope(parent)?
            .children
            .iter()
            .copied()
            .find(|id| {
                self.scopes.get_scope(*id).is_some_and(|scope| {
                    scope.scope_type == ScopeType::Module
                        && scope.symbols.get(name).is_some_and(|definitions| {
                            definitions.iter().any(|def| {
                                def.definition_type == DefinitionType::ModuleDefinition
                                    && def.scope_id == Some(*id)
                            })
                        })
                })
            })
    }
}
