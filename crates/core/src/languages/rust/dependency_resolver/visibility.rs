//! Which declarations a usage can see, and which of them is nearest.
//!
//! A declaration is reachable when it is hoisted, when the usage sits inside the scope declaring it,
//! or when it precedes the usage in a scope the usage's chain passes through. Among the reachable
//! ones, the nearest scope wins, and within a scope the last declaration before the usage does —
//! which is what makes `let x = 1; let x = x + 1;` read the first `x`.

use super::nested_scope_resolver::ScopeUtilities;
use super::rust_dependency_resolver::RustDependencyResolver;
use crate::models::{Definition, ScopeId, ScopeType, Usage};

impl RustDependencyResolver {
    /// Calculate scope distance between two scopes
    pub fn calculate_scope_distance(
        &self,
        from_scope: ScopeId,
        to_scope: ScopeId,
    ) -> Option<usize> {
        if from_scope == to_scope {
            return Some(0);
        }

        let mut distance = 0;
        let mut current_scope = from_scope;

        while let Some(scope) = self.symbol_table.scopes.get_scope(current_scope) {
            if current_scope == to_scope {
                return Some(distance);
            }

            if let Some(parent_id) = scope.parent {
                current_scope = parent_id;
                distance += 1;
            } else {
                break;
            }
        }

        None
    }

    // Helper methods for Rust-specific dependency resolution
    pub(super) fn is_accessible_basic(&self, usage: &Usage, definition: &Definition) -> bool {
        // ImportDefinitions and ModuleDefinitions are always accessible from any scope
        if matches!(
            definition.definition_type,
            crate::models::DefinitionType::ImportDefinition
                | crate::models::DefinitionType::ModuleDefinition
        ) {
            return true;
        }

        // ConstDefinitions are also accessible from any scope (like module-level constants)
        if matches!(
            definition.definition_type,
            crate::models::DefinitionType::ConstDefinition
        ) {
            return true;
        }

        // StructFieldDefinitions are accessible from any scope within the same module
        if matches!(
            definition.definition_type,
            crate::models::DefinitionType::StructFieldDefinition
        ) {
            return true;
        }

        // A hoisted definition is visible ahead of its own declaration, but only within the scope
        // that declares it: a `fn` nested inside another function is not reachable from outside.
        //
        // This applies to bare names only. A member reached through a path or a receiver is
        // reachable by qualification rather than by lexical nesting, so scope must not exclude it.
        if self.is_hoisted_basic(definition) {
            return match self.is_reached_lexically(usage, definition) {
                true => self.is_in_scope_chain(usage, definition),
                false => true,
            };
        }

        // For variable definitions, check scope accessibility
        if matches!(
            definition.definition_type,
            crate::models::DefinitionType::VariableDefinition
        ) {
            let usage_scope_id = self
                .symbol_table
                .scopes
                .find_scope_at_position(&usage.position)
                .unwrap_or(0);

            let def_scope_id = definition.scope_id.unwrap_or(0);

            // In Rust, nested functions cannot access variables from outer functions
            // Check if usage is in a nested function scope
            if self.is_usage_in_nested_function(usage_scope_id, def_scope_id) {
                return false;
            }

            // Variables are only accessible within the same scope or descendant scopes
            return ScopeUtilities::is_scope_accessible(
                &self.symbol_table,
                usage_scope_id,
                def_scope_id,
            ) || usage_scope_id == def_scope_id;
        }

        true
    }

    /// Whether lexical scope governs how this usage finds this definition.
    ///
    /// A bare identifier is looked up lexically. A path or a receiver reaches its target by
    /// qualification instead, and so does a method: a method is never found by bare lexical
    /// lookup, which matters inside macro token trees where `c.get()` is not parsed as a call and
    /// its method name arrives as a bare identifier.
    fn is_reached_lexically(&self, usage: &Usage, definition: &Definition) -> bool {
        let bare_name = matches!(usage.kind, crate::models::UsageKind::Identifier)
            && !matches!(
                usage.context.as_deref(),
                Some("scoped_identifier") | Some("field_expression")
            );

        bare_name
            && !matches!(
                definition.definition_type,
                crate::models::DefinitionType::MethodDefinition
            )
    }

    /// Whether the definition sits in the usage's scope or one enclosing it.
    ///
    /// An item that creates a scope has its own name recorded inside that scope rather than
    /// alongside it, so a top-level `fn` is registered in the function's own scope. The parent
    /// therefore counts too, otherwise no such item would ever look reachable.
    fn is_in_scope_chain(&self, usage: &Usage, definition: &Definition) -> bool {
        let chain = self.usage_scope_chain(usage);

        definition.scope_id.is_some_and(|def_scope| {
            chain.contains(&def_scope)
                || self
                    .parent_scope(def_scope)
                    .is_some_and(|parent| chain.contains(&parent))
        })
    }

    fn parent_scope(&self, scope_id: ScopeId) -> Option<ScopeId> {
        self.symbol_table
            .scopes
            .get_scope(scope_id)
            .and_then(|scope| scope.parent)
    }

    /// The definition in the scope nearest the usage.
    ///
    /// A bare identifier names the nearest binding in scope, so proximity decides before
    /// definition type does: a `let` binding in the enclosing block shadows a function item of the
    /// same name declared further out.
    pub(super) fn select_nearest_in_scope_chain<'a>(
        &self,
        usage: &Usage,
        matching_definitions: &[&'a Definition],
    ) -> Option<&'a Definition> {
        self.usage_scope_chain(usage)
            .iter()
            .find_map(|scope_id| self.select_within_scope(usage, matching_definitions, *scope_id))
    }

    /// Within one scope, the binding named is the last one declared before the usage: a later
    /// `let` of the same name shadows an earlier one. A hoisted definition is visible even when
    /// declared afterwards, so it is the fallback when nothing precedes.
    fn select_within_scope<'a>(
        &self,
        usage: &Usage,
        matching_definitions: &[&'a Definition],
        scope_id: ScopeId,
    ) -> Option<&'a Definition> {
        // An item that creates a scope records its own name inside that scope, so a struct declared
        // in a module is registered one level in. Counting the parent too keeps it findable at the
        // level that actually declares it, instead of losing to a farther candidate.
        let in_scope: Vec<&'a Definition> = matching_definitions
            .iter()
            // A field is only reachable through a receiver, never by a name in scope. Inside a
            // macro token tree `x.field()` arrives as a bare identifier, so without this a field
            // could win over the method actually being called.
            .filter(|def| {
                !matches!(
                    def.definition_type,
                    crate::models::DefinitionType::StructFieldDefinition
                )
            })
            .filter(|def| {
                def.scope_id == Some(scope_id)
                    || def
                        .scope_id
                        .and_then(|def_scope| self.parent_scope(def_scope))
                        == Some(scope_id)
            })
            .copied()
            .collect();

        in_scope
            .iter()
            // Up to and including the usage's own line: a closure parameter and a generic are
            // declared on the line their body reads them from. A binding declared *later* is not
            // visible, so it is no longer the answer of last resort — only a hoisted declaration is.
            .filter(|def| def.position.start_line <= usage.position.start_line)
            .max_by_key(|def| def.position.start_line)
            .copied()
            .or_else(|| {
                in_scope
                    .iter()
                    .find(|def| self.is_hoisted_basic(def))
                    .copied()
            })
    }

    /// The usage's own scope followed by its enclosing scopes, nearest first.
    fn usage_scope_chain(&self, usage: &Usage) -> Vec<ScopeId> {
        let usage_scope = self
            .symbol_table
            .scopes
            .find_scope_at_position(&usage.position)
            .unwrap_or(0);

        std::iter::once(usage_scope)
            .chain(self.symbol_table.scopes.get_parent_scopes(usage_scope))
            .collect()
    }

    pub(super) fn is_hoisted_basic(&self, definition: &Definition) -> bool {
        use crate::models::DefinitionType;
        matches!(
            definition.definition_type,
            DefinitionType::FunctionDefinition
                | DefinitionType::MethodDefinition
                | DefinitionType::StructDefinition
                | DefinitionType::EnumDefinition
                | DefinitionType::TypeDefinition
                | DefinitionType::ModuleDefinition
                | DefinitionType::MacroDefinition
        )
    }

    fn is_usage_in_nested_function(&self, usage_scope_id: ScopeId, def_scope_id: ScopeId) -> bool {
        // Check if usage is in a nested function trying to access a variable from an outer function

        // Find the function scope that contains the usage
        let usage_function_scope = self.find_enclosing_function_scope(usage_scope_id);

        // Find the function scope that contains the definition
        let def_function_scope = self.find_enclosing_function_scope(def_scope_id);

        // If both are in function scopes and they're different, this is a nested function access
        if let (Some(usage_fn_scope), Some(def_fn_scope)) =
            (usage_function_scope, def_function_scope)
        {
            if usage_fn_scope != def_fn_scope {
                // Check if the usage function is nested within the definition function
                let mut current_scope = usage_fn_scope;
                while let Some(scope) = self.symbol_table.scopes.get_scope(current_scope) {
                    if let Some(parent_id) = scope.parent {
                        if parent_id == def_fn_scope {
                            // This is a nested function trying to access outer function variable
                            return true;
                        }
                        current_scope = parent_id;
                    } else {
                        break;
                    }
                }
            }
        }

        false
    }

    fn find_enclosing_function_scope(&self, scope_id: ScopeId) -> Option<ScopeId> {
        let mut current_scope = scope_id;
        while let Some(scope) = self.symbol_table.scopes.get_scope(current_scope) {
            if scope.scope_type == ScopeType::Function {
                return Some(current_scope);
            }
            if let Some(parent_id) = scope.parent {
                current_scope = parent_id;
            } else {
                break;
            }
        }
        None
    }
}
