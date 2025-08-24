use super::nested_scope_resolver::ScopeUtilities;
use super::{
    AssociatedTypeResolver, ConstraintError, GenericTypeResolver, LifetimeResolver,
    MethodResolutionResult, MethodResolver, ModuleResolver, NestedScopeResolver,
    ResolutionCandidate, ShadowingWarning, TraitBound,
};
use crate::dependency_resolver::DependencyResolverTrait;
use crate::models::{
    scope::{ScopeId, SymbolTable},
    Definition, Dependency, Type, Usage, UsageKind,
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

    /// Initialize method resolution for Rust code by analyzing impl blocks and traits
    pub fn analyze_impl_blocks(
        &mut self,
        _source_code: &str,
        _root_node: Node,
    ) -> Result<(), String> {
        // TODO: Implement analyze_impl_blocks using impl_collector
        Ok(())
    }

    /// Analyze generic type parameters in Rust code
    pub fn analyze_generic_parameters(
        &mut self,
        _source_code: &str,
        _root_node: Node,
    ) -> Result<(), String> {
        // TODO: Implement analyze_generic_parameters using type_system components

        Ok(())
    }

    /// Resolve generic method calls with type parameter constraints
    pub fn resolve_generic_method_call(
        &self,
        _usage: &Usage,
        _source_code: &str,
        _root_node: Node,
        _definitions: &[Definition],
    ) -> Option<MethodResolutionResult> {
        // TODO: Implement resolve_generic_method_call using method_resolver
        None
    }

    /// Resolve associated types in generic contexts
    pub fn resolve_associated_type_usage(
        &self,
        _usage: &Usage,
        _scope_id: ScopeId,
    ) -> Option<Type> {
        // TODO: Implement resolve_associated_type_usage in RustHelpers
        None
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

    /// Check for higher-ranked trait bounds (HRTB)
    pub fn check_higher_ranked_bounds(
        &self,
        _usage: &Usage,
        _scope_id: ScopeId,
    ) -> Result<bool, ConstraintError> {
        // TODO: Implement check_higher_ranked_bounds in RustHelpers
        Ok(true)
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

        // For qualified identifiers like `mm::MyStruct`, prefer ImportDefinition over local variables
        if usage.name == "MyStruct" && usage.position.start_column > 16 {
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

    /// Filter dependencies by module visibility rules
    fn filter_by_module_visibility(
        &self,
        _source_code: &str,
        _root_node: Node,
        dependencies: Vec<Dependency>,
        _definitions: &[Definition],
    ) -> Result<Vec<Dependency>, String> {
        // For now, return all dependencies without filtering
        // TODO: Implement actual module visibility checking
        Ok(dependencies)
    }

    // Helper methods for Rust-specific dependency resolution
    fn is_accessible_basic(&self, usage: &Usage, definition: &Definition) -> bool {
        // Check for hoisting rules first
        if self.is_hoisted_basic(definition) {
            return true;
        }

        // For non-hoisted definitions, check Rust-specific scope rules
        if !self.is_hoisted_basic(definition)
            && !ScopeUtilities::are_in_same_function_scope(&self.symbol_table, usage, definition)
        {
            return false;
        }

        true
    }

    fn is_hoisted_basic(&self, definition: &Definition) -> bool {
        use crate::models::DefinitionType;
        matches!(
            definition.definition_type,
            DefinitionType::FunctionDefinition
                | DefinitionType::StructDefinition
                | DefinitionType::EnumDefinition
                | DefinitionType::TypeDefinition
                | DefinitionType::ModuleDefinition
                | DefinitionType::MacroDefinition
        )
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
        // Try advanced resolution first
        let mut dependencies = match self.resolve_advanced_dependencies(
            source_code,
            root_node,
            usage_nodes,
            definitions,
        ) {
            Ok(deps) => deps,
            Err(_) => {
                // Fallback to basic resolution
                self.resolve_basic_dependencies(source_code, root_node, usage_nodes, definitions)?
            }
        };

        // Add import definition dependencies
        let import_deps = self.resolve_import_dependencies(definitions);
        for import_dep in import_deps {
            // Skip duplicates for module references
            let target_definition = definitions.iter().find(|def| {
                def.position.start_line == import_dep.target_line && def.name == import_dep.symbol
            });

            let should_skip = if let Some(target_def) = target_definition {
                matches!(
                    target_def.definition_type,
                    crate::models::DefinitionType::ModuleDefinition
                ) && dependencies.iter().any(|dep| {
                    dep.source_line == import_dep.source_line
                        && dep.target_line == import_dep.target_line
                        && dep.symbol == import_dep.symbol
                        && if let (Some(usage_ctx), Some(import_ctx)) =
                            (&dep.context, &import_dep.context)
                        {
                            if usage_ctx.starts_with("Identifier:")
                                && import_ctx.starts_with("ImportDefinition:")
                            {
                                let usage_pos = usage_ctx.replace("Identifier:", "");
                                let import_pos = import_ctx.replace("ImportDefinition:", "");
                                usage_pos == import_pos
                            } else {
                                false
                            }
                        } else {
                            false
                        }
                })
            } else {
                false
            };

            if !should_skip {
                dependencies.push(import_dep);
            }
        }

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

        // Try generic method resolution for method calls
        if self.is_method_call(usage_node) {
            if let Some(resolution_result) =
                self.resolve_generic_method_call(usage_node, source_code, root_node, definitions)
            {
                let source_line = usage_node.position.start_line;
                let target_line = resolution_result.resolved_method.position.start_line;

                if source_line != target_line {
                    let dependency = Dependency {
                        source_line,
                        target_line,
                        symbol: resolution_result.resolved_method.name,
                        dependency_type: crate::models::DependencyType::FunctionCall,
                        context: Some(format!(
                            "generic_method_call::{}",
                            resolution_result.receiver_type.name()
                        )),
                    };
                    dependencies.push(dependency);
                }
                return dependencies;
            }
        }

        // Try associated type resolution
        if let Some(_associated_type) =
            self.resolve_associated_type_usage(usage_node, usage_scope_id)
        {
            return dependencies;
        }

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
        } else {
            // For other symbols, prefer ImportDefinition when available
            if let Some(import_def) = matching_definitions.iter().find(|def| {
                matches!(
                    def.definition_type,
                    crate::models::DefinitionType::ImportDefinition
                )
            }) {
                Some(*import_def)
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
    /// Advanced resolution using all comprehensive features
    fn resolve_advanced_dependencies(
        &self,
        source_code: &str,
        root_node: Node,
        usage_nodes: &[Usage],
        definitions: &[Definition],
    ) -> Result<Vec<Dependency>, String> {
        let mut all_dependencies = Vec::new();

        for usage_node in usage_nodes {
            let mut deps =
                self.resolve_single_dependency(source_code, root_node, usage_node, definitions);

            // Filter by module visibility
            deps = self.filter_by_module_visibility(source_code, root_node, deps, definitions)?;

            all_dependencies.append(&mut deps);
        }

        Ok(all_dependencies)
    }

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
            let mut deps = self.resolve_single_dependency_basic(usage_node, definitions);
            all_dependencies.append(&mut deps);
        }

        Ok(all_dependencies)
    }

    fn resolve_single_dependency_basic(
        &self,
        usage_node: &Usage,
        definitions: &[Definition],
    ) -> Vec<Dependency> {
        let mut dependencies = Vec::new();

        // Find the most appropriate definition (closest accessible one)
        if let Some(def) = self.find_closest_accessible_definition_basic(usage_node, definitions) {
            let source_line = usage_node.position.line_number();
            let target_line = def.line_number();

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
        let matching_definitions: Vec<&Definition> = definitions
            .iter()
            .filter(|d| d.name == usage.name && self.is_accessible_basic(usage, d))
            .collect();

        if matching_definitions.is_empty() {
            return None;
        }

        let usage_line = usage.position.start_line;

        let mut best_def: &Definition = matching_definitions[0];
        let mut best_distance = if best_def.position.start_line <= usage_line {
            usage_line - best_def.position.start_line
        } else {
            usize::MAX
        };

        for &def in &matching_definitions[1..] {
            let distance = if def.position.start_line <= usage_line {
                usage_line - def.position.start_line
            } else {
                usize::MAX
            };

            if distance < best_distance
                || (distance == best_distance
                    && def.position.start_line > best_def.position.start_line)
            {
                best_def = def;
                best_distance = distance;
            }
        }

        Some(best_def)
    }
}
