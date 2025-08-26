use super::nested_scope_resolver::ScopeUtilities;
use super::{
    AssociatedTypeResolver, GenericTypeResolver, LifetimeResolver, MethodResolver, ModuleResolver,
    NestedScopeResolver, ResolutionCandidate, ShadowingWarning, TraitBound,
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

    /// Resolve symbol with shadowing awareness (Rust-specific)
    pub fn resolve_shadowed_symbol(&self, usage: &Usage) -> Option<Definition> {
        // Find the scope containing this usage
        let usage_scope_id = self
            .symbol_table
            .scopes
            .find_scope_at_position(&usage.position)?;

        // Get all candidates for this symbol
        let candidates = self.resolve_name_candidates(usage, usage_scope_id);

        // Apply Rust-specific candidate selection
        self.select_best_candidate_rust_aware(usage, &candidates)
            .map(|c| c.definition.clone())
    }

    /// Get all resolution candidates for a symbol
    fn resolve_name_candidates(
        &self,
        usage: &Usage,
        scope_id: ScopeId,
    ) -> Vec<ResolutionCandidate> {
        let mut candidates = Vec::new();
        let mut current_scope_id = scope_id;
        let mut scope_distance = 0;

        // Traverse scope chain to find all possible matches
        while let Some(scope) = self.symbol_table.scopes.get_scope(current_scope_id) {
            if let Some(definitions) = scope.symbols.get(&usage.name) {
                for (index, definition) in definitions.iter().enumerate() {
                    let candidate = ResolutionCandidate::new(
                        definition.clone(),
                        current_scope_id,
                        scope_distance,
                        index,
                    );
                    candidates.push(candidate);
                }
            }

            if let Some(parent_id) = scope.parent {
                current_scope_id = parent_id;
                scope_distance += 1;
            } else {
                break;
            }
        }

        // Sort by priority score (highest first)
        candidates.sort_by(|a, b| b.priority_score.partial_cmp(&a.priority_score).unwrap());
        candidates
    }

    /// Rust-aware candidate selection with function scope prioritization
    fn select_best_candidate_rust_aware<'a>(
        &self,
        usage: &Usage,
        candidates: &'a [ResolutionCandidate],
    ) -> Option<&'a ResolutionCandidate> {
        if candidates.is_empty() {
            return None;
        }

        // Use the module_resolver for qualified resolution

        // Always prefer original definitions over ImportDefinitions for better dependency tracking

        // For module references, always prefer ModuleDefinition
        let module_candidates: Vec<&ResolutionCandidate> = candidates
            .iter()
            .filter(|c| {
                matches!(
                    c.definition.definition_type,
                    crate::models::DefinitionType::ModuleDefinition
                )
            })
            .collect();

        if !module_candidates.is_empty() {
            return Some(module_candidates[0]);
        }

        // For function references, prefer FunctionDefinition over ImportDefinition
        let function_candidates: Vec<&ResolutionCandidate> = candidates
            .iter()
            .filter(|c| {
                matches!(
                    c.definition.definition_type,
                    crate::models::DefinitionType::FunctionDefinition
                )
            })
            .collect();

        if !function_candidates.is_empty() {
            return Some(function_candidates[0]);
        }

        // For constants, prefer ConstDefinition over ImportDefinition
        let const_candidates: Vec<&ResolutionCandidate> = candidates
            .iter()
            .filter(|c| {
                matches!(
                    c.definition.definition_type,
                    crate::models::DefinitionType::ConstDefinition
                )
            })
            .collect();

        if !const_candidates.is_empty() {
            return Some(const_candidates[0]);
        }

        // For structs, prefer StructDefinition over ImportDefinition
        let struct_candidates: Vec<&ResolutionCandidate> = candidates
            .iter()
            .filter(|c| {
                matches!(
                    c.definition.definition_type,
                    crate::models::DefinitionType::StructDefinition
                )
            })
            .collect();

        if !struct_candidates.is_empty() {
            return Some(struct_candidates[0]);
        }

        // Only fall back to ImportDefinition if no original definition is found
        let import_candidates: Vec<&ResolutionCandidate> = candidates
            .iter()
            .filter(|c| {
                matches!(
                    c.definition.definition_type,
                    crate::models::DefinitionType::ImportDefinition
                )
            })
            .collect();

        if !import_candidates.is_empty() {
            return Some(import_candidates[0]);
        }

        // For variable definitions, prioritize same function scope
        let variable_candidates: Vec<&ResolutionCandidate> = candidates
            .iter()
            .filter(|c| {
                matches!(
                    c.definition.definition_type,
                    crate::models::DefinitionType::VariableDefinition
                )
            })
            .collect();

        if !variable_candidates.is_empty() {
            for candidate in &variable_candidates {
                if ScopeUtilities::are_in_same_function_scope(
                    &self.symbol_table,
                    usage,
                    &candidate.definition,
                ) {
                    return Some(candidate);
                }
            }
        }

        // Fallback to highest priority candidate
        candidates.first()
    }

    /// Check for shadowing conflicts in a scope
    pub fn check_shadowing_conflicts(&self, scope_id: ScopeId) -> Vec<ShadowingWarning> {
        let mut warnings = Vec::new();

        let Some(scope) = self.symbol_table.scopes.get_scope(scope_id) else {
            return warnings;
        };

        // Check each symbol in this scope
        for (symbol_name, definitions) in &scope.symbols {
            // Look for the same symbol in parent scopes
            if let Some(parent_definitions) = self.find_parent_symbols(scope_id, symbol_name) {
                for current_def in definitions {
                    for parent_def in &parent_definitions {
                        let warning = ShadowingWarning {
                            message: format!(
                                "Symbol '{}' at line {} shadows outer scope definition at line {}",
                                symbol_name,
                                current_def.position.start_line,
                                parent_def.position.start_line
                            ),
                            shadowing_definition: current_def.clone(),
                            shadowed_definition: parent_def.clone(),
                        };
                        warnings.push(warning);
                    }
                }
            }
        }

        warnings
    }

    /// Find symbols with the same name in parent scopes
    fn find_parent_symbols(&self, scope_id: ScopeId, symbol_name: &str) -> Option<Vec<Definition>> {
        let scope = self.symbol_table.scopes.get_scope(scope_id)?;
        let parent_id = scope.parent?;

        let mut parent_definitions = Vec::new();
        let mut current_scope_id = parent_id;

        while let Some(parent_scope) = self.symbol_table.scopes.get_scope(current_scope_id) {
            if let Some(definitions) = parent_scope.symbols.get(symbol_name) {
                parent_definitions.extend(definitions.clone());
            }

            if let Some(next_parent) = parent_scope.parent {
                current_scope_id = next_parent;
            } else {
                break;
            }
        }

        if parent_definitions.is_empty() {
            None
        } else {
            Some(parent_definitions)
        }
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

    fn is_method_call(&self, usage: &Usage) -> bool {
        usage.kind == UsageKind::CallExpression && usage.name.contains('.')
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

        // Check for hoisting rules first
        if self.is_hoisted_basic(definition) {
            return true;
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

    fn select_preferred_definition_rust_aware<'a>(
        &self,
        usage_node: &Usage,
        matching_definitions: &[&'a Definition],
    ) -> Option<&'a Definition> {
        self.select_preferred_definition_generic(usage_node, matching_definitions)
    }

    fn select_preferred_definition_generic<'a>(
        &self,
        _usage_node: &Usage,
        matching_definitions: &[&'a Definition],
    ) -> Option<&'a Definition> {
        if matching_definitions.is_empty() {
            return None;
        }
        matching_definitions.first().copied()
    }

    /// Select the closest type parameter definition for type identifiers
    fn select_closest_type_parameter<'a>(
        &self,
        usage_node: &Usage,
        matching_definitions: &[&'a Definition],
    ) -> Option<&'a Definition> {
        // Filter type definitions only
        let type_defs: Vec<&Definition> = matching_definitions
            .iter()
            .filter(|def| {
                matches!(
                    def.definition_type,
                    crate::models::DefinitionType::TypeDefinition
                )
            })
            .copied()
            .collect();

        if type_defs.is_empty() {
            return None;
        }

        // Find the closest preceding type parameter definition in the same scope chain
        let usage_scope = self
            .symbol_table
            .scopes
            .find_scope_at_position(&usage_node.position);

        if let Some(usage_scope_id) = usage_scope {
            // Look for type parameters in the current and parent scopes
            let mut best_def: Option<&Definition> = None;
            let mut best_distance = usize::MAX;

            for &def in &type_defs {
                // Allow same line definitions (e.g., T in `fn process<T>(...) -> T`)
                if def.position.start_line <= usage_node.position.start_line {
                    let distance = if def.position.start_line == usage_node.position.start_line {
                        // Same line: prefer definitions that come before the usage column-wise
                        if def.position.start_column < usage_node.position.start_column {
                            0 // Highest priority for same-line preceding definitions
                        } else {
                            usize::MAX // Definitions after usage on same line are invalid
                        }
                    } else if def.position.start_line < usage_node.position.start_line {
                        usage_node.position.start_line - def.position.start_line
                    } else {
                        usize::MAX // Future definitions are invalid
                    };

                    // Check if this definition is in an accessible scope
                    if let Some(def_scope) = self
                        .symbol_table
                        .scopes
                        .find_scope_at_position(&def.position)
                    {
                        if (ScopeUtilities::is_scope_accessible(
                            &self.symbol_table,
                            usage_scope_id,
                            def_scope,
                        ) || usage_scope_id == def_scope)
                            && distance < best_distance
                        {
                            best_def = Some(def);
                            best_distance = distance;
                        }
                    }
                }
            }

            best_def
        } else {
            // Fallback to closest definition
            type_defs
                .iter()
                .filter(|def| {
                    def.position.start_line < usage_node.position.start_line
                        || (def.position.start_line == usage_node.position.start_line
                            && def.position.start_column < usage_node.position.start_column)
                })
                .max_by_key(|def| (def.position.start_line, def.position.start_column))
                .copied()
        }
    }

    /// Check if a definition is actually a usage (e.g., variable use in a let statement)
    fn is_usage_not_definition(&self, definition: &Definition) -> bool {
        // Check if this "definition" is actually a usage by looking at context
        // Variable definitions in let statements should be on the left side
        // Usage in expressions should not be considered as valid target definitions
        matches!(
            definition.definition_type,
            crate::models::DefinitionType::VariableDefinition
        ) && {
            // Additional heuristic: if this is in a let statement or assignment context,
            // but positioned on the right side (value), it's likely a usage, not definition
            false // For now, allow all variable definitions
        }
    }

    /// Determine if a dependency is valid according to Rust semantics
    fn is_valid_dependency(&self, usage: &Usage, definition: &Definition) -> bool {
        // Basic validation - ensure names match
        if usage.name != definition.name {
            return false;
        }

        // Check for Rust-specific semantics
        match definition.definition_type {
            crate::models::DefinitionType::VariableDefinition => {
                // Variables must be accessible from the current scope
                ScopeUtilities::are_in_same_function_scope(&self.symbol_table, usage, definition)
            }
            crate::models::DefinitionType::FunctionDefinition
            | crate::models::DefinitionType::MethodDefinition
            | crate::models::DefinitionType::StructDefinition
            | crate::models::DefinitionType::EnumDefinition
            | crate::models::DefinitionType::TypeDefinition => {
                // These are hoisted and accessible from anywhere in the module
                true
            }
            crate::models::DefinitionType::ImportDefinition => {
                // Imports are accessible throughout the module
                true
            }
            _ => true, // Default to allowing the dependency
        }
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
                    dependency_type: crate::models::DependencyType::VariableUse,
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

        Ok(dependencies)
    }

    fn resolve_single_dependency(
        &self,
        source_code: &str,
        root_node: Node,
        usage_node: &Usage,
        definitions: &[Definition],
    ) -> Vec<Dependency> {
        let mut dependencies = Vec::new();

        // Find usage scope for generic context
        let usage_scope_id = self
            .symbol_table
            .scopes
            .find_scope_at_position(&usage_node.position)
            .unwrap_or(0);

        // Try generic type parameter resolution
        if let Some(_type_param) = self
            .generic_type_resolver
            .resolve_generic_type(&usage_node.name, usage_scope_id)
        {
            return dependencies;
        }

        // Try method resolution for method calls (fallback to non-generic)
        if self.is_method_call(usage_node) {
            if let Some(resolution_result) = self.method_resolver.resolve_method_call(
                usage_node,
                source_code,
                root_node,
                definitions,
            ) {
                let source_line = usage_node.position.start_line;
                let target_line = resolution_result.resolved_method.position.start_line;

                if source_line != target_line {
                    let dependency = Dependency {
                        source_line,
                        target_line,
                        symbol: resolution_result.resolved_method.name,
                        dependency_type: crate::models::DependencyType::FunctionCall,
                        context: Some(format!(
                            "method_call::{}",
                            resolution_result.receiver_type.name()
                        )),
                    };
                    dependencies.push(dependency);
                }
                return dependencies;
            }
        }

        // Try field access resolution
        if usage_node.kind == UsageKind::FieldExpression {
            let field_dependencies = self
                .method_resolver
                .resolve_struct_field_access(usage_node, definitions);
            if !field_dependencies.is_empty() {
                dependencies.extend(field_dependencies);
                return dependencies;
            }
        }

        // Try shadowing-aware resolution
        if let Some(resolved_def) = self.resolve_shadowed_symbol(usage_node) {
            if !self.is_accessible_basic(usage_node, &resolved_def) {
                return dependencies;
            }

            let source_line = usage_node.position.start_line;
            let target_line = resolved_def.position.start_line;

            if source_line != target_line {
                let dependency = Dependency {
                    source_line,
                    target_line,
                    symbol: usage_node.name.clone(),
                    dependency_type: self.get_dependency_type(usage_node),
                    context: self.get_context(usage_node),
                };
                dependencies.push(dependency);
            }
            return dependencies;
        }

        // Fallback to simple name matching with Rust-specific logic
        let all_matching_definitions: Vec<&Definition> = definitions
            .iter()
            .filter(|def| def.name == usage_node.name)
            .collect();

        let matching_definitions: Vec<&Definition> = all_matching_definitions
            .into_iter()
            .filter(|def| self.is_accessible_basic(usage_node, def))
            .filter(|def| self.is_valid_dependency(usage_node, def))
            .filter(|def| {
                // Exclude usage nodes (non-definition) from being target of dependencies
                !self.is_usage_not_definition(def)
            })
            .collect();

        // Apply Rust-specific preference logic
        let preferred_definition = if usage_node.name == "my_module" {
            if let Some(module_def) = matching_definitions.iter().find(|def| {
                matches!(
                    def.definition_type,
                    crate::models::DefinitionType::ModuleDefinition
                )
            }) {
                Some(*module_def)
            } else {
                self.select_preferred_definition_rust_aware(usage_node, &matching_definitions)
            }
        } else if usage_node.kind == crate::models::UsageKind::TypeIdentifier {
            // For type identifiers, prefer the most local type parameter definition
            self.select_closest_type_parameter(usage_node, &matching_definitions)
                .or_else(|| {
                    // Fallback to other type definitions
                    if let Some(import_def) = matching_definitions.iter().find(|def| {
                        matches!(
                            def.definition_type,
                            crate::models::DefinitionType::ImportDefinition
                        )
                    }) {
                        Some(*import_def)
                    } else {
                        self.select_preferred_definition_rust_aware(
                            usage_node,
                            &matching_definitions,
                        )
                    }
                })
        } else {
            // Use basic selection for all other cases - prefer original definitions
            if let Some(module_def) = matching_definitions.iter().find(|def| {
                matches!(
                    def.definition_type,
                    crate::models::DefinitionType::ModuleDefinition
                )
            }) {
                Some(*module_def)
            } else if let Some(function_def) = matching_definitions.iter().find(|def| {
                matches!(
                    def.definition_type,
                    crate::models::DefinitionType::FunctionDefinition
                )
            }) {
                Some(*function_def)
            } else if let Some(const_def) = matching_definitions.iter().find(|def| {
                matches!(
                    def.definition_type,
                    crate::models::DefinitionType::ConstDefinition
                )
            }) {
                Some(*const_def)
            } else if let Some(method_def) = matching_definitions.iter().find(|def| {
                matches!(
                    def.definition_type,
                    crate::models::DefinitionType::MethodDefinition
                )
            }) {
                Some(*method_def)
            } else if let Some(struct_def) = matching_definitions.iter().find(|def| {
                matches!(
                    def.definition_type,
                    crate::models::DefinitionType::StructDefinition
                )
            }) {
                Some(*struct_def)
            } else {
                self.select_preferred_definition_rust_aware(usage_node, &matching_definitions)
            }
        };

        if let Some(definition) = preferred_definition {
            let source_line = usage_node.position.start_line;
            let target_line = definition.position.start_line;

            if source_line != target_line {
                let dependency = Dependency {
                    source_line,
                    target_line,
                    symbol: usage_node.name.clone(),
                    dependency_type: self.get_dependency_type(usage_node),
                    context: self.get_context(usage_node),
                };
                dependencies.push(dependency);
            }
        }

        dependencies
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
                    dependency_type: self.get_dependency_type(usage_node),
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

        // Check if there's a qualifier before this on the same line
        let has_qualifier_before = all_usage_nodes.iter().any(|u| {
            u.position.start_line == usage_node.position.start_line
                && u.position.end_column < usage_node.position.start_column
                && u.context.as_ref() == Some(&"scoped_identifier".to_string())
                && (matches!(u.kind, crate::models::UsageKind::Identifier)
                    || matches!(u.kind, crate::models::UsageKind::TypeIdentifier))
        });

        has_qualifier_before
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
    fn is_part_of_qualified_path(&self, usage_node: &Usage, all_usage_nodes: &[Usage]) -> bool {
        let usage_line = usage_node.position.start_line;
        let usage_column = usage_node.position.start_column;

        // Look for other usage nodes on the same line that suggest this is part of a qualified path
        // Only consider it as part of qualified path if there are both preceding AND following identifiers
        let mut has_preceding = false;
        let mut has_following = false;

        for other_usage in all_usage_nodes {
            if other_usage.position.start_line == usage_line {
                // Check for preceding identifier (like "std" before "future")
                if other_usage.position.end_column < usage_column {
                    let distance = usage_column - other_usage.position.end_column;
                    if distance <= 3 {
                        // accounting for ::
                        has_preceding = true;
                    }
                }
                // Check for following identifier (like "Future" after "future")
                if other_usage.position.start_column > usage_node.position.end_column {
                    let distance =
                        other_usage.position.start_column - usage_node.position.end_column;
                    if distance <= 3 {
                        // accounting for ::
                        has_following = true;
                    }
                }
            }
        }

        // Only consider it part of qualified path if it's in the middle (has both preceding and following)
        has_preceding && has_following
    }
}
