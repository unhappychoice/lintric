use super::nested_scope_resolver::ScopeUtilities;
use super::{
    AssociatedTypeResolver, GenericTypeResolver, LifetimeResolver, MethodResolver, ModuleResolver,
    NestedScopeResolver, TraitBound,
};
use crate::dependency_resolver::DependencyResolverTrait;
use crate::models::{
    scope::{CodeAnalysisContext, SymbolTable},
    Definition, Dependency, ScopeId, ScopeType, Type, Usage, UsageKind,
};
use tree_sitter::Node;

/// Rust-specific dependency resolver that implements comprehensive dependency resolution
/// including generics, lifetimes, traits, and Rust-specific language features
pub struct RustDependencyResolver {
    symbol_table: SymbolTable,
    nested_scope_resolver: NestedScopeResolver,
    module_resolver: ModuleResolver,
    pub method_resolver: MethodResolver,
    generic_type_resolver: GenericTypeResolver,
    associated_type_resolver: AssociatedTypeResolver,
    lifetime_resolver: LifetimeResolver,
}

impl RustDependencyResolver {
    pub fn new(symbol_table: SymbolTable) -> Self {
        let nested_scope_resolver = NestedScopeResolver::new(symbol_table.scopes.clone());
        let module_resolver = ModuleResolver::new();
        let method_resolver = MethodResolver::new();
        let generic_type_resolver = GenericTypeResolver::new();
        let associated_type_resolver = AssociatedTypeResolver::new();
        let lifetime_resolver = LifetimeResolver::new();

        Self {
            symbol_table,
            nested_scope_resolver,
            module_resolver,
            method_resolver,
            generic_type_resolver,
            associated_type_resolver,
            lifetime_resolver,
        }
    }

    pub fn new_from_context(context: CodeAnalysisContext) -> Self {
        // Create a SymbolTable from the new context for backward compatibility
        let mut symbol_table = SymbolTable::new();

        // Copy scope structure
        symbol_table.scopes = context.scopes;

        // Add definitions to the symbol table
        for (name, definitions) in context.definitions.get_all_definitions() {
            for definition in definitions {
                symbol_table.add_enhanced_symbol(name.clone(), definition.clone());
            }
        }

        Self::new(symbol_table)
    }

    pub fn get_module_resolver(&self) -> &ModuleResolver {
        &self.module_resolver
    }

    pub fn get_module_resolver_mut(&mut self) -> &mut ModuleResolver {
        &mut self.module_resolver
    }

    pub fn get_method_resolver(&self) -> &MethodResolver {
        &self.method_resolver
    }

    pub fn get_generic_type_resolver(&self) -> &GenericTypeResolver {
        &self.generic_type_resolver
    }

    pub fn get_generic_type_resolver_mut(&mut self) -> &mut GenericTypeResolver {
        &mut self.generic_type_resolver
    }

    pub fn get_associated_type_resolver(&self) -> &AssociatedTypeResolver {
        &self.associated_type_resolver
    }

    pub fn get_lifetime_resolver(&self) -> &LifetimeResolver {
        &self.lifetime_resolver
    }

    /// Validate trait bounds for a given type
    pub fn validate_trait_bounds(
        &self,
        type_arg: &Type,
        bounds: &[TraitBound],
        _scope_id: ScopeId,
    ) -> bool {
        self.generic_type_resolver
            .constraint_solver
            .check_trait_bounds(std::slice::from_ref(type_arg), bounds)
    }

    /// Get nested scope information using nested scope resolver
    pub fn analyze_nested_scopes(&self, scope_id: ScopeId) -> bool {
        self.nested_scope_resolver
            .scope_tree
            .get_scope(scope_id)
            .is_some()
    }

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
    fn is_accessible_basic(&self, usage: &Usage, definition: &Definition) -> bool {
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
    fn select_nearest_in_scope_chain<'a>(
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
            .filter(|def| def.position.start_line < usage.position.start_line)
            .max_by_key(|def| def.position.start_line)
            .copied()
            .or_else(|| in_scope.first().copied())
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

    fn is_hoisted_basic(&self, definition: &Definition) -> bool {
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

    /// Resolve dependencies for import definitions (use statements)
    fn resolve_import_dependencies(&self, definitions: &[Definition]) -> Vec<Dependency> {
        let mut import_dependencies = Vec::new();

        for import_def in definitions.iter().filter(|def| {
            matches!(
                def.definition_type,
                crate::models::DefinitionType::ImportDefinition
            )
        }) {
            if let Some(original_def) = definitions.iter().find(|def| {
                def.name == import_def.name
                    && !matches!(
                        def.definition_type,
                        crate::models::DefinitionType::ImportDefinition
                    )
                    && def.position != import_def.position
            }) {
                let dependency = Dependency {
                    source_line: import_def.position.start_line,
                    target_line: original_def.position.start_line,
                    symbol: import_def.name.clone(),
                    dependency_type: crate::models::DependencyType::Import,
                    context: Some(format!(
                        "ImportDefinition:{}:{}",
                        import_def.position.start_line, import_def.position.start_column
                    )),
                };
                import_dependencies.push(dependency);
            }
        }

        import_dependencies
    }
}

impl DependencyResolverTrait for RustDependencyResolver {
    fn resolve_dependencies(
        &self,
        source_code: &str,
        root_node: Node,
        usage_nodes: &[Usage],
        definitions: &[Definition],
    ) -> Result<Vec<Dependency>, String> {
        // Use basic resolution with fixed priorities
        let mut dependencies =
            self.resolve_basic_dependencies(source_code, root_node, usage_nodes, definitions)?;

        // Add import definition dependencies (ImportDefinition -> original definition)
        let import_deps = self.resolve_import_dependencies(definitions);
        dependencies.extend(import_deps);

        // Add trait implementation dependencies (impl method -> trait declaration), which have no
        // usage to resolve and are derived from the impl block's structure instead
        let trait_impl_deps =
            super::trait_implementation_resolver::resolve(source_code, root_node)?;
        dependencies.extend(trait_impl_deps);

        Ok(dependencies)
    }
}

impl RustDependencyResolver {
    /// Basic fallback resolution for cases where advanced resolution fails
    fn resolve_basic_dependencies(
        &self,
        _source_code: &str,
        _root_node: Node,
        usage_nodes: &[Usage],
        definitions: &[Definition],
    ) -> Result<Vec<Dependency>, String> {
        let mut all_dependencies = Vec::new();

        for usage_node in usage_nodes {
            let mut deps = self.resolve_single_dependency_with_scope_aware_external_filtering(
                usage_node,
                definitions,
                usage_nodes,
            );
            all_dependencies.append(&mut deps);
        }

        Ok(all_dependencies)
    }

    fn resolve_single_dependency_with_scope_aware_external_filtering(
        &self,
        usage_node: &Usage,
        definitions: &[Definition],
        all_usage_nodes: &[Usage],
    ) -> Vec<Dependency> {
        let mut dependencies = Vec::new();

        // Check if this usage is a method name in a qualified call that has no accessible definition
        // But don't skip if it's a type reference (like in use statements or type annotations)
        if self.is_method_name_in_qualified_call(usage_node, all_usage_nodes)
            && self.is_method_in_scoped_identifier_without_definition(
                usage_node,
                definitions,
                all_usage_nodes,
            )
            && !self.is_type_reference_in_scoped_identifier(usage_node)
        {
            // Skip creating dependency for method calls that are not defined in accessible scopes
            return dependencies;
        }

        // Skip creating dependencies for TypeIdentifiers that are part of qualified paths
        // (like "future" in "std::future::Future")
        if matches!(usage_node.kind, UsageKind::TypeIdentifier)
            && self.is_part_of_qualified_path(usage_node, all_usage_nodes)
        {
            return dependencies;
        }

        // Proceed with normal resolution
        if let Some(def) = self.find_closest_accessible_definition_basic(usage_node, definitions) {
            let source_line = usage_node.position.line_number();
            let target_line = def.line_number();

            // Simplified approach: allow all variable dependencies for now
            // The old implementation was more permissive

            if source_line != target_line {
                dependencies.push(Dependency {
                    source_line,
                    target_line,
                    symbol: usage_node.name.clone(),
                    dependency_type: self.get_dependency_type(usage_node, def),
                    context: self.get_context(usage_node),
                });
            }
        }

        dependencies
    }

    fn find_closest_accessible_definition_basic<'a>(
        &self,
        usage: &Usage,
        definitions: &'a [Definition],
    ) -> Option<&'a Definition> {
        // Simple approach: find all matching definitions and apply priority logic
        // This matches the old implementation behavior more closely
        let matching_definitions: Vec<&Definition> = definitions
            .iter()
            .filter(|d| d.name == usage.name && self.is_accessible_basic(usage, d))
            .collect();

        if matching_definitions.is_empty() {
            return None;
        }

        self.select_best_definition_by_priority(&matching_definitions, usage)
    }

    fn select_best_definition_by_priority<'a>(
        &self,
        matching_definitions: &[&'a Definition],
        usage: &Usage,
    ) -> Option<&'a Definition> {
        // Apply context-aware priority logic based on usage type

        // For main function usages, prefer ImportDefinition (imported symbols) FIRST
        // Check if this usage is within main function and there's an import available
        if self.is_usage_in_main_function(usage) {
            for &def in matching_definitions {
                if matches!(
                    def.definition_type,
                    crate::models::DefinitionType::ImportDefinition
                ) {
                    return Some(def);
                }
            }
        }

        // For method calls (CallExpression), prioritize methods over fields
        if matches!(usage.kind, crate::models::UsageKind::CallExpression) {
            // Prefer MethodDefinition and FunctionDefinition for method calls
            for &def in matching_definitions {
                if matches!(
                    def.definition_type,
                    crate::models::DefinitionType::MethodDefinition
                        | crate::models::DefinitionType::FunctionDefinition
                ) {
                    return Some(def);
                }
            }
        }

        // A field named by a struct literal can only refer to a field declaration, so unlike a
        // field expression there is no method to prefer over it
        if matches!(usage.kind, crate::models::UsageKind::FieldInitializer) {
            for &def in matching_definitions {
                if matches!(
                    def.definition_type,
                    crate::models::DefinitionType::StructFieldDefinition
                ) {
                    return Some(def);
                }
            }
        }

        // For field expressions, first check if these are actually method calls
        // In case of StructFieldAccess dependency_type, prefer methods over fields (due to potential misclassification)
        if matches!(usage.kind, crate::models::UsageKind::FieldExpression) {
            // First try to find MethodDefinition in impl blocks (more specific)
            for &def in matching_definitions {
                if matches!(
                    def.definition_type,
                    crate::models::DefinitionType::MethodDefinition
                ) {
                    return Some(def);
                }
            }
            // Then try StructFieldDefinition for actual field access
            for &def in matching_definitions {
                if matches!(
                    def.definition_type,
                    crate::models::DefinitionType::StructFieldDefinition
                ) {
                    return Some(def);
                }
            }
        }

        // A bare identifier names the nearest binding in scope, and a type parameter belongs to the
        // generic item that declares it, so proximity decides before the definition-type ladder
        // below gets a say.
        if matches!(
            usage.kind,
            crate::models::UsageKind::Identifier | crate::models::UsageKind::TypeIdentifier
        ) {
            if let Some(nearest) = self.select_nearest_in_scope_chain(usage, matching_definitions) {
                return Some(nearest);
            }
        }

        // General priority for other cases (import statements themselves)
        // For module references, prefer ModuleDefinition
        for &def in matching_definitions {
            if matches!(
                def.definition_type,
                crate::models::DefinitionType::ModuleDefinition
            ) {
                return Some(def);
            }
        }

        // For function references, prefer FunctionDefinition
        for &def in matching_definitions {
            if matches!(
                def.definition_type,
                crate::models::DefinitionType::FunctionDefinition
            ) {
                return Some(def);
            }
        }

        // For methods, prefer MethodDefinition
        for &def in matching_definitions {
            if matches!(
                def.definition_type,
                crate::models::DefinitionType::MethodDefinition
            ) {
                return Some(def);
            }
        }

        // For constants, prefer ConstDefinition
        for &def in matching_definitions {
            if matches!(
                def.definition_type,
                crate::models::DefinitionType::ConstDefinition
            ) {
                return Some(def);
            }
        }

        // For structs, prefer StructDefinition
        for &def in matching_definitions {
            if matches!(
                def.definition_type,
                crate::models::DefinitionType::StructDefinition
            ) {
                return Some(def);
            }
        }

        // First, try to find variable definitions in the same function scope
        let mut same_scope_defs = Vec::new();
        for &def in matching_definitions {
            if matches!(
                def.definition_type,
                crate::models::DefinitionType::VariableDefinition
            ) && ScopeUtilities::are_in_same_function_scope(&self.symbol_table, usage, def)
            {
                // Among same-scope definitions, only consider those defined before the usage
                if def.position.start_line < usage.position.start_line
                    || (def.position.start_line == usage.position.start_line
                        && def.position.start_column < usage.position.start_column)
                {
                    same_scope_defs.push(def);
                }
            }
        }

        if !same_scope_defs.is_empty() {
            // Return the closest preceding definition in the same scope
            same_scope_defs.sort_by_key(|def| {
                (
                    std::cmp::Reverse(def.position.start_line),
                    std::cmp::Reverse(def.position.start_column),
                )
            });
            return same_scope_defs.first().copied();
        }

        // Only fall back to ImportDefinition if no original definition is found
        for &def in matching_definitions {
            if matches!(
                def.definition_type,
                crate::models::DefinitionType::ImportDefinition
            ) {
                return Some(def);
            }
        }

        // As absolute fallback, return any remaining definition
        matching_definitions.first().copied()
    }

    fn is_usage_in_main_function(&self, usage: &Usage) -> bool {
        // Find the main function definition
        for scope in self.symbol_table.scopes.scopes.values() {
            if let Some(main_defs) = scope.symbols.get("main") {
                for def in main_defs {
                    if matches!(
                        def.definition_type,
                        crate::models::DefinitionType::FunctionDefinition
                    ) {
                        // Find function body scope that contains this main function
                        let main_line = def.position.start_line;
                        for body_scope in self.symbol_table.scopes.scopes.values() {
                            // Look for a scope that starts right after the main function definition
                            if body_scope.position.start_line == main_line + 1
                                || (body_scope.position.start_line <= main_line + 1
                                    && body_scope.position.end_line > main_line)
                            {
                                // Check if usage is within this function body scope
                                if usage.position.start_line > main_line
                                    && usage.position.start_line <= body_scope.position.end_line
                                {
                                    return true;
                                }
                            }
                        }
                    }
                }
            }
        }
        false
    }

    /// Check if this usage should be skipped because it has no definition
    /// in the qualifier's scope
    fn is_method_in_scoped_identifier_without_definition(
        &self,
        usage_node: &Usage,
        definitions: &[Definition],
        all_usage_nodes: &[Usage],
    ) -> bool {
        // Only apply to scoped identifiers
        if usage_node.context.as_ref() != Some(&"scoped_identifier".to_string()) {
            return false;
        }

        // Find the qualifier (type part) of this scoped identifier
        let qualifier = all_usage_nodes
            .iter()
            .filter(|u| {
                u.position.start_line == usage_node.position.start_line
                    && u.position.end_column < usage_node.position.start_column
                    && u.context.as_ref() == Some(&"scoped_identifier".to_string())
                    && matches!(u.kind, crate::models::UsageKind::Identifier)
            })
            .max_by_key(|u| u.position.start_column);

        if let Some(qualifier) = qualifier {
            // Find the qualifier's definition in symbol_table
            let qualifier_scope_id = self
                .symbol_table
                .scopes
                .find_scope_at_position(&qualifier.position)
                .unwrap_or(0);

            let mut current_scope_id = qualifier_scope_id;
            while let Some(scope) = self.symbol_table.scopes.get_scope(current_scope_id) {
                if let Some(qualifier_definitions) = scope.symbols.get(&qualifier.name) {
                    // Look for the method in definitions that are related to this qualifier
                    let has_method_definition = definitions.iter().any(|def| {
                        def.name == usage_node.name
                            && qualifier_definitions.iter().any(|qual_def| {
                                // Check if this method definition is related to the qualifier's scope
                                match qual_def.definition_type {
                                    crate::models::DefinitionType::StructDefinition
                                    | crate::models::DefinitionType::EnumDefinition
                                    | crate::models::DefinitionType::TypeDefinition => {
                                        // For local types, check if method is in nearby lines (impl block)
                                        (def.position.start_line as i32
                                            - qual_def.position.start_line as i32)
                                            .abs()
                                            < 20
                                    }
                                    _ => false, // For imports, no local method definitions
                                }
                            })
                    });

                    return !has_method_definition;
                }
                if let Some(parent_id) = scope.parent {
                    current_scope_id = parent_id;
                } else {
                    break;
                }
            }
        }

        // If we can't find qualifier or determine scope, don't skip
        false
    }

    /// Check if this usage is likely a method name in a qualified call (Type::method)
    fn is_method_name_in_qualified_call(
        &self,
        usage_node: &Usage,
        all_usage_nodes: &[Usage],
    ) -> bool {
        // Must be in scoped_identifier context
        if usage_node.context.as_ref() != Some(&"scoped_identifier".to_string()) {
            return false;
        }

        // Must be an identifier, not a type identifier
        if !matches!(usage_node.kind, crate::models::UsageKind::Identifier) {
            return false;
        }

        // The qualifier must be the segment immediately before this one. Accepting anything
        // earlier on the line confused two separate paths, so `crate::V + crate::W` treated `V`
        // as the qualifier of `W`.
        all_usage_nodes.iter().any(|other| {
            is_in_path(other)
                && is_adjacent_segment(other, usage_node)
                && matches!(
                    other.kind,
                    crate::models::UsageKind::Identifier | crate::models::UsageKind::TypeIdentifier
                )
        })
    }

    /// Check if this usage is a type reference in a scoped identifier context
    fn is_type_reference_in_scoped_identifier(&self, usage_node: &Usage) -> bool {
        // If it's a TypeIdentifier, it's definitely a type reference
        matches!(usage_node.kind, crate::models::UsageKind::TypeIdentifier)
    }

    /// Check if the usage represents a closure capturing a variable from an outer scope
    #[allow(dead_code)]
    fn is_closure_capture(&self, _usage_node: &Usage, _def: &Definition) -> bool {
        // Find the closest enclosing function-like scope for the usage
        let usage_scope_id = self
            .symbol_table
            .scopes
            .find_scope_at_position(&_usage_node.position);

        if let Some(scope_id) = usage_scope_id {
            // Walk up the scope chain to find if we're inside a closure
            let mut current_scope_id = scope_id;
            while let Some(scope) = self.symbol_table.scopes.get_scope(current_scope_id) {
                if matches!(scope.scope_type, ScopeType::Closure) {
                    // We're inside a closure, so cross-function capture is allowed
                    return true;
                }

                if let Some(parent_id) = scope.parent {
                    current_scope_id = parent_id;
                } else {
                    break;
                }
            }
        }

        // Not inside a closure, so cross-function capture is not allowed
        false
    }

    /// Check if a TypeIdentifier is part of a qualified path (like "future" in "std::future::Future")
    /// Whether this name qualifies a later segment of a path rather than being what the path names.
    ///
    /// `Foo` in `std::Foo::Bar` qualifies `Bar`, so resolving it against a local definition of the
    /// same name would be wrong. `Foo` in `Foo::Bar` is not a qualifier of that kind — it is the
    /// type the associated item belongs to, and does resolve.
    ///
    /// Being inside a path at all is a fact about the tree, carried by the usage's context. Only
    /// which segment it is comes from position, and then only from adjacency across the `::`, so
    /// two names that merely share a line are never mistaken for one path.
    fn is_part_of_qualified_path(&self, usage_node: &Usage, all_usage_nodes: &[Usage]) -> bool {
        if !is_in_path(usage_node) {
            return false;
        }

        let has_preceding = all_usage_nodes
            .iter()
            .any(|other| is_adjacent_segment(other, usage_node));
        let has_following = all_usage_nodes
            .iter()
            .any(|other| is_adjacent_segment(usage_node, other));

        has_preceding && has_following
    }
}

/// Width of the `::` between path segments.
const PATH_SEPARATOR: usize = 2;

fn is_in_path(usage: &Usage) -> bool {
    matches!(
        usage.context.as_deref(),
        Some("scoped_identifier") | Some("scoped_type_identifier")
    )
}

/// Whether `earlier` is the segment immediately before `later` in one path.
fn is_adjacent_segment(earlier: &Usage, later: &Usage) -> bool {
    earlier.position.start_line == later.position.start_line
        && earlier.position.end_column + PATH_SEPARATOR == later.position.start_column
}
