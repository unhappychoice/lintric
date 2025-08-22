use crate::dependency_resolver::DependencyResolver;
use crate::method_resolver::{MethodResolutionResult, MethodResolver};
use crate::models::{
    scope::{ScopeId, SymbolTable},
    Definition, Dependency, Type, Usage, UsageKind,
};
use crate::module_resolver::ModuleResolver;
use crate::nested_scope_resolver::NestedScopeResolver;
use std::collections::HashMap;
use tree_sitter::Node;

// Generic Type System Structures

#[derive(Debug, Clone, PartialEq)]
pub struct TypeParam {
    pub name: String,
    pub bounds: Vec<TraitBound>,
    pub default: Option<Type>,
    pub variance: Variance,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LifetimeParam {
    pub name: String,
    pub bounds: Vec<LifetimeBound>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Variance {
    Covariant,
    Contravariant,
    Invariant,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TraitBound {
    pub trait_name: String,
    pub type_args: Vec<Type>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LifetimeBound {
    pub lifetime: LifetimeId,
    pub outlives: LifetimeId,
}

#[derive(Debug, Clone, PartialEq)]
pub enum LifetimeId {
    Named(String),
    Anonymous(u32),
    Static,
    Infer(u32),
}

pub type TypeVarId = u32;
pub type TraitId = u32;

#[derive(Debug, Clone)]
pub enum Constraint {
    TraitBound {
        type_var: TypeVarId,
        trait_def: TraitId,
    },
    Equality {
        left: Type,
        right: Type,
    },
    Lifetime {
        lifetime: LifetimeId,
        outlives: LifetimeId,
    },
    Associated {
        type_var: TypeVarId,
        trait_def: TraitId,
        assoc_type: String,
    },
}

#[derive(Debug, Clone)]
pub struct TypeSubstitution {
    pub type_vars: HashMap<TypeVarId, Type>,
    pub lifetimes: HashMap<LifetimeId, LifetimeId>,
}

#[derive(Debug, Clone)]
pub struct AssociatedType {
    pub name: String,
    pub trait_def: TraitId,
    pub bounds: Vec<TraitBound>,
    pub default: Option<Type>,
}

#[derive(Debug, Clone)]
pub struct LifetimeScope {
    pub lifetimes: HashMap<String, LifetimeId>,
    pub parent: Option<ScopeId>,
}

/// Generic Type Resolver for handling complex generic scenarios
#[derive(Debug, Clone)]
pub struct GenericTypeResolver {
    pub type_parameters: HashMap<ScopeId, Vec<TypeParam>>,
    pub lifetime_parameters: HashMap<ScopeId, Vec<LifetimeParam>>,
    pub constraint_solver: ConstraintSolver,
}

/// Constraint solving engine for generic type constraints
#[derive(Debug, Clone)]
pub struct ConstraintSolver {
    pub active_constraints: HashMap<TypeVarId, Vec<Constraint>>,
    pub trait_database: TraitDatabase,
}

/// Database for trait definitions and implementations
#[derive(Debug, Clone)]
pub struct TraitDatabase {
    pub traits: HashMap<TraitId, TraitDefinition>,
    pub implementations: HashMap<TraitId, Vec<TraitImplementation>>,
}

#[derive(Debug, Clone)]
pub struct TraitDefinition {
    pub id: TraitId,
    pub name: String,
    pub type_params: Vec<TypeParam>,
    pub associated_types: Vec<AssociatedType>,
    pub methods: Vec<Definition>,
}

#[derive(Debug, Clone)]
pub struct TraitImplementation {
    pub trait_id: TraitId,
    pub target_type: Type,
    pub type_args: Vec<Type>,
    pub associated_type_mappings: HashMap<String, Type>,
}

/// Associated Type Resolver
#[derive(Debug, Clone)]
pub struct AssociatedTypeResolver {
    pub trait_database: TraitDatabase,
    pub impl_database: ImplDatabase,
}

#[derive(Debug, Clone)]
pub struct ImplDatabase {
    pub implementations: HashMap<String, Vec<TraitImplementation>>,
}

/// Lifetime Resolver for handling lifetime parameters
#[derive(Debug, Clone)]
pub struct LifetimeResolver {
    pub lifetime_scopes: HashMap<ScopeId, LifetimeScope>,
    pub borrow_checker: BorrowChecker,
}

#[derive(Debug, Clone)]
pub struct BorrowChecker {
    // Placeholder for borrow checking functionality
}

#[derive(Debug)]
pub enum ConstraintError {
    UnsatisfiedTraitBound(String),
    LifetimeError(String),
    TypeMismatch(String),
    RecursiveConstraint(String),
}

impl Default for GenericTypeResolver {
    fn default() -> Self {
        Self::new()
    }
}

impl GenericTypeResolver {
    pub fn new() -> Self {
        Self {
            type_parameters: HashMap::new(),
            lifetime_parameters: HashMap::new(),
            constraint_solver: ConstraintSolver::new(),
        }
    }

    pub fn add_type_parameters(&mut self, scope_id: ScopeId, params: Vec<TypeParam>) {
        self.type_parameters.insert(scope_id, params);
    }

    pub fn add_lifetime_parameters(&mut self, scope_id: ScopeId, params: Vec<LifetimeParam>) {
        self.lifetime_parameters.insert(scope_id, params);
    }

    pub fn resolve_generic_type(&self, type_name: &str, scope_id: ScopeId) -> Option<TypeParam> {
        if let Some(params) = self.type_parameters.get(&scope_id) {
            params.iter().find(|p| p.name == type_name).cloned()
        } else {
            None
        }
    }
}

impl Default for ConstraintSolver {
    fn default() -> Self {
        Self::new()
    }
}

impl ConstraintSolver {
    pub fn new() -> Self {
        Self {
            active_constraints: HashMap::new(),
            trait_database: TraitDatabase::new(),
        }
    }

    pub fn add_constraint(&mut self, type_var: TypeVarId, constraint: Constraint) {
        self.active_constraints
            .entry(type_var)
            .or_default()
            .push(constraint);
    }

    pub fn solve_constraints(&mut self) -> Result<TypeSubstitution, ConstraintError> {
        // Basic constraint solving implementation
        let substitution = TypeSubstitution {
            type_vars: HashMap::new(),
            lifetimes: HashMap::new(),
        };

        // Simplified solving logic - to be enhanced
        for constraints in self.active_constraints.values() {
            for constraint in constraints {
                match constraint {
                    Constraint::TraitBound { .. } => {
                        // Check trait bounds
                    }
                    Constraint::Equality { .. } => {
                        // Handle type equality
                    }
                    Constraint::Lifetime { .. } => {
                        // Handle lifetime constraints
                    }
                    Constraint::Associated { .. } => {
                        // Handle associated type constraints
                    }
                }
            }
        }

        Ok(substitution)
    }

    pub fn check_trait_bounds(&self, type_args: &[Type], bounds: &[TraitBound]) -> bool {
        // Basic trait bound checking - to be enhanced
        for _bound in bounds {
            // Check if each type argument satisfies the bound
            for _type_arg in type_args {
                // Implement trait bound validation
            }
        }
        true
    }
}

impl Default for TraitDatabase {
    fn default() -> Self {
        Self::new()
    }
}

impl TraitDatabase {
    pub fn new() -> Self {
        Self {
            traits: HashMap::new(),
            implementations: HashMap::new(),
        }
    }

    pub fn add_trait(&mut self, trait_def: TraitDefinition) {
        self.traits.insert(trait_def.id, trait_def);
    }

    pub fn add_implementation(&mut self, trait_id: TraitId, implementation: TraitImplementation) {
        self.implementations
            .entry(trait_id)
            .or_default()
            .push(implementation);
    }
}

impl Default for AssociatedTypeResolver {
    fn default() -> Self {
        Self::new()
    }
}

impl AssociatedTypeResolver {
    pub fn new() -> Self {
        Self {
            trait_database: TraitDatabase::new(),
            impl_database: ImplDatabase::new(),
        }
    }

    pub fn resolve_associated_type(
        &self,
        trait_impl: &TraitImplementation,
        assoc_name: &str,
    ) -> Option<Type> {
        trait_impl.associated_type_mappings.get(assoc_name).cloned()
    }

    pub fn project_type(
        &self,
        _base_type: &Type,
        _trait_def: TraitId,
        _assoc_name: &str,
    ) -> Option<Type> {
        // Implement type projection for associated types
        None
    }
}

impl Default for ImplDatabase {
    fn default() -> Self {
        Self::new()
    }
}

impl ImplDatabase {
    pub fn new() -> Self {
        Self {
            implementations: HashMap::new(),
        }
    }
}

impl Default for LifetimeResolver {
    fn default() -> Self {
        Self::new()
    }
}

impl LifetimeResolver {
    pub fn new() -> Self {
        Self {
            lifetime_scopes: HashMap::new(),
            borrow_checker: BorrowChecker::new(),
        }
    }

    pub fn add_lifetime_scope(&mut self, scope_id: ScopeId, scope: LifetimeScope) {
        self.lifetime_scopes.insert(scope_id, scope);
    }
}

impl Default for BorrowChecker {
    fn default() -> Self {
        Self::new()
    }
}

impl BorrowChecker {
    pub fn new() -> Self {
        Self {}
    }
}

/// Enhanced dependency resolver that combines:
/// - Base dependency resolution
/// - Scope awareness (#107)
/// - Nested scope resolution (#102/#108)  
/// - Shadowing resolution (#103)
/// - Module system integration and visibility rules (#104)
/// - Method resolution system (#105)
/// - Generic type parameters and advanced trait support (#106)
pub struct EnhancedDependencyResolver {
    symbol_table: SymbolTable,
    nested_scope_resolver: NestedScopeResolver,
    module_resolver: ModuleResolver,
    method_resolver: MethodResolver,
    generic_type_resolver: GenericTypeResolver,
    associated_type_resolver: AssociatedTypeResolver,
    lifetime_resolver: LifetimeResolver,
    #[allow(dead_code)]
    language: String,
}

#[derive(Debug, Clone)]
pub struct ShadowingWarning {
    pub message: String,
    pub shadowing_definition: Definition,
    pub shadowed_definition: Definition,
}

#[derive(Debug, Clone)]
pub struct ResolutionCandidate {
    pub definition: Definition,
    pub scope_distance: usize,
    pub shadowing_level: usize,
    pub priority_score: f64,
}

impl ResolutionCandidate {
    pub fn new(
        definition: Definition,
        _scope_id: ScopeId,
        scope_distance: usize,
        shadowing_level: usize,
    ) -> Self {
        let priority_score = Self::calculate_priority_score(scope_distance, shadowing_level);
        Self {
            definition,
            scope_distance,
            shadowing_level,
            priority_score,
        }
    }

    fn calculate_priority_score(scope_distance: usize, shadowing_level: usize) -> f64 {
        let distance_weight = 1.0 / (scope_distance as f64 + 1.0);
        let shadowing_weight = 10.0 / (shadowing_level as f64 + 1.0);
        distance_weight + shadowing_weight
    }
}

impl EnhancedDependencyResolver {
    pub fn new(symbol_table: SymbolTable, language: String) -> Self {
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
            language,
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
        source_code: &str,
        root_node: Node,
    ) -> Result<(), String> {
        if self.language != "Rust" {
            return Ok(());
        }

        let mut impl_collector =
            crate::languages::rust::rust_impl_collector::RustImplCollector::new();

        // Collect impl blocks
        let impl_blocks = impl_collector.collect_impl_blocks(source_code, root_node)?;
        for impl_block in impl_blocks {
            self.method_resolver
                .impl_block_analyzer
                .add_impl_block(impl_block);
        }

        // Collect traits
        let traits = impl_collector.collect_traits(source_code, root_node)?;
        for trait_def in traits {
            self.method_resolver.trait_resolver.add_trait(trait_def);
        }

        // Collect trait implementations
        let trait_impls = impl_collector.collect_trait_impl_blocks(source_code, root_node)?;
        for trait_impl in trait_impls {
            self.method_resolver
                .trait_resolver
                .add_trait_impl(trait_impl);
        }

        Ok(())
    }

    /// Analyze generic type parameters in Rust code
    pub fn analyze_generic_parameters(
        &mut self,
        source_code: &str,
        root_node: Node,
    ) -> Result<(), String> {
        if self.language != "Rust" {
            return Ok(());
        }

        // Collect generic parameters from function definitions
        let function_generics = self.collect_function_generics(source_code, root_node)?;
        for (scope_id, params) in function_generics {
            self.generic_type_resolver
                .add_type_parameters(scope_id, params);
        }

        // Collect generic parameters from struct/enum definitions
        let type_generics = self.collect_type_generics(source_code, root_node)?;
        for (scope_id, params) in type_generics {
            self.generic_type_resolver
                .add_type_parameters(scope_id, params);
        }

        // Collect generic parameters from impl blocks
        let impl_generics = self.collect_impl_generics(source_code, root_node)?;
        for (scope_id, params) in impl_generics {
            self.generic_type_resolver
                .add_type_parameters(scope_id, params);
        }

        Ok(())
    }

    fn collect_function_generics(
        &self,
        _source_code: &str,
        _root_node: Node,
    ) -> Result<Vec<(ScopeId, Vec<TypeParam>)>, String> {
        // Placeholder implementation
        // TODO: Parse function signatures and extract generic parameters
        Ok(Vec::new())
    }

    fn collect_type_generics(
        &self,
        _source_code: &str,
        _root_node: Node,
    ) -> Result<Vec<(ScopeId, Vec<TypeParam>)>, String> {
        // Placeholder implementation
        // TODO: Parse struct/enum definitions and extract generic parameters
        Ok(Vec::new())
    }

    fn collect_impl_generics(
        &self,
        _source_code: &str,
        _root_node: Node,
    ) -> Result<Vec<(ScopeId, Vec<TypeParam>)>, String> {
        // Placeholder implementation
        // TODO: Parse impl blocks and extract generic parameters
        Ok(Vec::new())
    }

    /// Resolve generic method calls with type parameter constraints
    pub fn resolve_generic_method_call(
        &self,
        usage: &Usage,
        source_code: &str,
        root_node: Node,
        definitions: &[Definition],
    ) -> Option<MethodResolutionResult> {
        // First try normal method resolution
        if let Some(result) =
            self.method_resolver
                .resolve_method_call(usage, source_code, root_node, definitions)
        {
            // Enhanced with generic constraint checking
            if self.validate_generic_constraints(&result) {
                return Some(result);
            }
        }

        None
    }

    fn validate_generic_constraints(&self, _result: &MethodResolutionResult) -> bool {
        // Placeholder for generic constraint validation
        // TODO: Check that all type parameters satisfy their trait bounds
        true
    }

    /// Resolve associated types in generic contexts
    pub fn resolve_associated_type_usage(&self, usage: &Usage, _scope_id: ScopeId) -> Option<Type> {
        // Check if this usage refers to an associated type
        if usage.name.contains("::") {
            let parts: Vec<&str> = usage.name.split("::").collect();
            if parts.len() >= 2 {
                let _trait_name = parts[0];
                let _assoc_type_name = parts[1];
                // TODO: Implement associated type resolution
            }
        }

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
        // Placeholder for HRTB checking
        // TODO: Implement for<'a> Fn(&'a str) -> &'a str patterns
        Ok(true)
    }

    /// Resolve symbol with shadowing awareness
    pub fn resolve_shadowed_symbol(&self, usage: &Usage) -> Option<Definition> {
        // Find the scope containing this usage
        let usage_scope_id = self
            .symbol_table
            .scopes
            .find_scope_at_position(&usage.position)?;

        // Get all candidates for this symbol
        let candidates = self.resolve_name_candidates(usage, usage_scope_id);

        // Select the best candidate based on priority
        self.select_best_candidate(&candidates)
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
                        index, // Shadowing level within this scope
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

    /// Select the best candidate from available options
    fn select_best_candidate<'a>(
        &self,
        candidates: &'a [ResolutionCandidate],
    ) -> Option<&'a ResolutionCandidate> {
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
        // Placeholder for nested scope analysis - to be integrated properly
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
}

impl DependencyResolver for EnhancedDependencyResolver {
    fn resolve_dependencies(
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

        // Try generic method resolution for method calls (Rust only)
        if self.language == "Rust" && self.is_method_call(usage_node) {
            if let Some(resolution_result) =
                self.resolve_generic_method_call(usage_node, source_code, root_node, definitions)
            {
                let dependency = Dependency {
                    source_line: usage_node.position.start_line,
                    target_line: resolution_result.resolved_method.position.start_line,
                    symbol: resolution_result.resolved_method.name,
                    dependency_type: crate::models::DependencyType::FunctionCall,
                    context: Some(format!(
                        "generic_method_call::{}",
                        resolution_result.receiver_type.name()
                    )),
                };
                dependencies.push(dependency);
                return dependencies;
            }
        }

        // Try associated type resolution for Rust
        if self.language == "Rust" {
            if let Some(associated_type) =
                self.resolve_associated_type_usage(usage_node, usage_scope_id)
            {
                let dependency = Dependency {
                    source_line: usage_node.position.start_line,
                    target_line: usage_node.position.start_line, // Associated types reference the usage location
                    symbol: usage_node.name.clone(),
                    dependency_type: crate::models::DependencyType::TypeReference,
                    context: Some(format!("associated_type::{}", associated_type.name())),
                };
                dependencies.push(dependency);
                return dependencies;
            }
        }

        // Try generic type parameter resolution
        if self.language == "Rust" {
            if let Some(type_param) = self
                .generic_type_resolver
                .resolve_generic_type(&usage_node.name, usage_scope_id)
            {
                let dependency = Dependency {
                    source_line: usage_node.position.start_line,
                    target_line: usage_node.position.start_line, // Type parameters reference the usage location
                    symbol: usage_node.name.clone(),
                    dependency_type: crate::models::DependencyType::TypeReference,
                    context: Some(format!("generic_type_param::{}", type_param.name)),
                };
                dependencies.push(dependency);
                return dependencies;
            }
        }

        // Try method resolution for method calls (Rust only) - fallback to non-generic
        if self.language == "Rust" && self.is_method_call(usage_node) {
            if let Some(resolution_result) = self.method_resolver.resolve_method_call(
                usage_node,
                source_code,
                root_node,
                definitions,
            ) {
                let dependency = Dependency {
                    source_line: usage_node.position.start_line,
                    target_line: resolution_result.resolved_method.position.start_line,
                    symbol: resolution_result.resolved_method.name,
                    dependency_type: crate::models::DependencyType::FunctionCall,
                    context: Some(format!(
                        "method_call::{}",
                        resolution_result.receiver_type.name()
                    )),
                };
                dependencies.push(dependency);
                return dependencies;
            }
        }

        // Try shadowing-aware resolution
        if let Some(resolved_def) = self.resolve_shadowed_symbol(usage_node) {
            let dependency = Dependency {
                source_line: usage_node.position.start_line,
                target_line: resolved_def.position.start_line,
                symbol: usage_node.name.clone(),
                dependency_type: self.get_dependency_type(usage_node),
                context: self.get_context(usage_node),
            };
            dependencies.push(dependency);
            return dependencies;
        }

        // Fallback to simple name matching
        for definition in definitions {
            if definition.name == usage_node.name {
                let dependency = Dependency {
                    source_line: usage_node.position.start_line,
                    target_line: definition.position.start_line,
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{scope::ScopeType, DefinitionType, Position, UsageKind};

    fn create_test_position(line: usize, column: usize) -> Position {
        Position {
            start_line: line,
            start_column: column,
            end_line: line,
            end_column: column + 1,
        }
    }

    fn create_test_definition(name: &str, line: usize) -> Definition {
        Definition {
            name: name.to_string(),
            definition_type: DefinitionType::VariableDefinition,
            position: create_test_position(line, 1),
        }
    }

    fn create_test_usage(name: &str, line: usize) -> Usage {
        Usage {
            name: name.to_string(),
            kind: UsageKind::Identifier,
            position: create_test_position(line, 5),
        }
    }

    #[test]
    fn test_enhanced_resolver_basic_resolution() {
        let mut symbol_table = SymbolTable::new();

        let func_scope_id = symbol_table.scopes.create_scope(
            Some(0),
            ScopeType::Function,
            create_test_position(5, 1),
            create_test_position(15, 1),
        );

        let outer_def = create_test_definition("x", 1);
        let inner_def = create_test_definition("x", 8);

        symbol_table.add_symbol(
            "x".to_string(),
            outer_def,
            0,
            crate::models::scope::Accessibility::ScopeLocal,
            false,
        );
        symbol_table.add_symbol(
            "x".to_string(),
            inner_def.clone(),
            func_scope_id,
            crate::models::scope::Accessibility::ScopeLocal,
            false,
        );

        let resolver = EnhancedDependencyResolver::new(symbol_table, "rust".to_string());
        let usage = create_test_usage("x", 10);

        let resolved = resolver.resolve_shadowed_symbol(&usage);
        assert!(resolved.is_some());
        assert_eq!(resolved.unwrap().position.start_line, 8);
    }

    #[test]
    fn test_enhanced_resolver_shadowing_detection() {
        let mut symbol_table = SymbolTable::new();

        let func_scope_id = symbol_table.scopes.create_scope(
            Some(0),
            ScopeType::Function,
            create_test_position(5, 1),
            create_test_position(15, 1),
        );

        let outer_def = create_test_definition("var", 1);
        let inner_def = create_test_definition("var", 8);

        symbol_table.add_symbol(
            "var".to_string(),
            outer_def,
            0,
            crate::models::scope::Accessibility::ScopeLocal,
            false,
        );
        symbol_table.add_symbol(
            "var".to_string(),
            inner_def,
            func_scope_id,
            crate::models::scope::Accessibility::ScopeLocal,
            false,
        );

        let resolver = EnhancedDependencyResolver::new(symbol_table, "rust".to_string());
        let warnings = resolver.check_shadowing_conflicts(func_scope_id);

        assert!(!warnings.is_empty());
        assert!(warnings[0].message.contains("shadows outer scope"));
    }

    #[test]
    fn test_generic_type_resolver_basic() {
        let mut resolver = GenericTypeResolver::new();
        let scope_id = 1;

        let type_param = TypeParam {
            name: "T".to_string(),
            bounds: vec![TraitBound {
                trait_name: "Clone".to_string(),
                type_args: vec![],
            }],
            default: None,
            variance: Variance::Invariant,
        };

        resolver.add_type_parameters(scope_id, vec![type_param.clone()]);

        let resolved = resolver.resolve_generic_type("T", scope_id);
        assert!(resolved.is_some());
        assert_eq!(resolved.unwrap().name, "T");
    }

    #[test]
    fn test_constraint_solver_basic() {
        let mut solver = ConstraintSolver::new();
        let type_var = 1;
        let constraint = Constraint::TraitBound {
            type_var,
            trait_def: 1,
        };

        solver.add_constraint(type_var, constraint);
        let result = solver.solve_constraints();
        assert!(result.is_ok());
    }

    #[test]
    fn test_trait_database_operations() {
        let mut db = TraitDatabase::new();
        let trait_def = TraitDefinition {
            id: 1,
            name: "Clone".to_string(),
            type_params: vec![],
            associated_types: vec![],
            methods: vec![],
        };

        db.add_trait(trait_def);
        assert!(db.traits.contains_key(&1));
    }

    #[test]
    fn test_enhanced_resolver_with_generics() {
        let symbol_table = SymbolTable::new();
        let resolver = EnhancedDependencyResolver::new(symbol_table, "Rust".to_string());

        // Test that generic type resolver is properly initialized
        assert!(resolver
            .get_generic_type_resolver()
            .type_parameters
            .is_empty());

        // Test accessing different resolvers
        assert!(resolver
            .get_associated_type_resolver()
            .trait_database
            .traits
            .is_empty());
        assert!(resolver.get_lifetime_resolver().lifetime_scopes.is_empty());
    }

    #[test]
    fn test_generic_context_container() {
        // Test case from Issue #106: Generic Container with trait bounds
        let symbol_table = SymbolTable::new();
        let mut resolver = EnhancedDependencyResolver::new(symbol_table, "Rust".to_string());

        // Simulate adding generic type parameters for Container<T: Clone + Debug>
        let container_scope_id = 1;
        let type_param = TypeParam {
            name: "T".to_string(),
            bounds: vec![
                TraitBound {
                    trait_name: "Clone".to_string(),
                    type_args: vec![],
                },
                TraitBound {
                    trait_name: "Debug".to_string(),
                    type_args: vec![],
                },
            ],
            default: None,
            variance: Variance::Invariant,
        };

        resolver
            .get_generic_type_resolver_mut()
            .add_type_parameters(container_scope_id, vec![type_param]);

        // Test resolving the generic type parameter
        let resolved = resolver
            .get_generic_type_resolver()
            .resolve_generic_type("T", container_scope_id);
        assert!(resolved.is_some());
        assert_eq!(resolved.unwrap().bounds.len(), 2);
    }

    #[test]
    fn test_associated_type_resolver() {
        let resolver = AssociatedTypeResolver::new();

        // Create a trait implementation with associated type mapping
        let trait_impl = TraitImplementation {
            trait_id: 1,
            target_type: Type::Concrete("Vec".to_string()),
            type_args: vec![Type::Concrete("i32".to_string())],
            associated_type_mappings: {
                let mut map = HashMap::new();
                map.insert("Item".to_string(), Type::Concrete("i32".to_string()));
                map
            },
        };

        let resolved = resolver.resolve_associated_type(&trait_impl, "Item");
        assert!(resolved.is_some());
        assert_eq!(resolved.unwrap(), Type::Concrete("i32".to_string()));
    }

    #[test]
    fn test_lifetime_scope_management() {
        let mut resolver = LifetimeResolver::new();
        let scope_id = 1;

        let lifetime_scope = LifetimeScope {
            lifetimes: {
                let mut map = HashMap::new();
                map.insert("'a".to_string(), LifetimeId::Named("'a".to_string()));
                map
            },
            parent: None,
        };

        resolver.add_lifetime_scope(scope_id, lifetime_scope);
        assert!(resolver.lifetime_scopes.contains_key(&scope_id));
    }

    #[test]
    fn test_trait_bound_validation() {
        let resolver = GenericTypeResolver::new();
        let type_arg = Type::Concrete("i32".to_string());
        let bounds = vec![TraitBound {
            trait_name: "Clone".to_string(),
            type_args: vec![],
        }];

        // For now, this should return true (basic implementation)
        let valid = resolver
            .constraint_solver
            .check_trait_bounds(&[type_arg], &bounds);
        assert!(valid);
    }

    #[test]
    fn test_constraint_error_handling() {
        let error =
            ConstraintError::UnsatisfiedTraitBound("T does not implement Clone".to_string());
        match error {
            ConstraintError::UnsatisfiedTraitBound(msg) => {
                assert!(msg.contains("Clone"));
            }
            _ => panic!("Expected UnsatisfiedTraitBound"),
        }
    }
}
