use super::nested_scope_resolver::ScopeUtilities;
use crate::dependency_resolver::receiver_narrowing::ReceiverNarrowing;
use crate::dependency_resolver::self_reference::SelfReference;
use crate::dependency_resolver::DependencyResolverTrait;
use crate::models::{
    scope::{CodeAnalysisContext, SymbolTable},
    Definition, DefinitionType, Dependency, Usage, UsageKind,
};
use tree_sitter::Node;

/// Each `let` paired with its initializer, so a binding stays out of the names it reads.
const OWN_INITIALIZERS: &str = include_str!("../../../../queries/rust/own_initializers.scm");

/// Rust-specific dependency resolver that implements comprehensive dependency resolution
/// including generics, lifetimes, traits, and Rust-specific language features
pub struct RustDependencyResolver {
    pub(super) symbol_table: SymbolTable,
}

impl RustDependencyResolver {
    pub fn new(symbol_table: SymbolTable) -> Self {
        Self { symbol_table }
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
        source_code: &str,
        root_node: Node,
        usage_nodes: &[Usage],
        definitions: &[Definition],
    ) -> Result<Vec<Dependency>, String> {
        // Read off the file once rather than per usage: every method call asks the same questions of
        // it, and a malformed query must fail rather than quietly resolve nothing.
        let narrowing =
            ReceiverNarrowing::new(&super::receiver_narrowing::DIALECT, source_code, root_node)?;
        let own = SelfReference::new(OWN_INITIALIZERS, source_code, root_node)?;

        Ok(usage_nodes
            .iter()
            .flat_map(|usage_node| {
                self.resolve_single_dependency_with_scope_aware_external_filtering(
                    &narrowing,
                    &own,
                    usage_node,
                    definitions,
                    usage_nodes,
                )
            })
            .collect())
    }

    fn resolve_single_dependency_with_scope_aware_external_filtering(
        &self,
        narrowing: &ReceiverNarrowing,
        own: &SelfReference,
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
        if let Some(def) = self.find_closest_accessible_definition_basic(
            narrowing,
            own,
            usage_node,
            definitions,
            all_usage_nodes,
        ) {
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

    /// Whether this candidate is a value the usage cannot be naming because something is reached
    /// through it.
    fn is_value_reached_through(
        &self,
        usage: &Usage,
        definition: &Definition,
        all_usage_nodes: &[Usage],
    ) -> bool {
        use crate::models::DefinitionType::{FunctionDefinition, VariableDefinition};

        matches!(
            definition.definition_type,
            FunctionDefinition | VariableDefinition
        ) && self.is_path_head(usage, all_usage_nodes)
    }

    fn find_closest_accessible_definition_basic<'a>(
        &self,
        narrowing: &ReceiverNarrowing,
        own: &SelfReference,
        usage: &Usage,
        definitions: &'a [Definition],
        all_usage_nodes: &[Usage],
    ) -> Option<&'a Definition> {
        // Simple approach: find all matching definitions and apply priority logic
        // This matches the old implementation behavior more closely
        let matching_definitions: Vec<&Definition> = definitions
            .iter()
            .filter(|d| d.name == usage.name && self.is_accessible_basic(usage, d))
            // A binding is not among the candidates for its own initializer, so `let w = w + 1`
            // looks past it and finds the previous `w`.
            .filter(|d| !own.declares(usage, d))
            .filter(|d| !self.is_value_reached_through(usage, d, all_usage_nodes))
            .collect();

        // `receiver.method()` reaches only what the receiver's type declares, so the priority logic
        // below chooses among those rather than among every method sharing the name.
        let matching_definitions = narrowing.narrow(usage, matching_definitions);

        if matching_definitions.is_empty() {
            return None;
        }

        self.select_best_definition_by_priority(&matching_definitions, usage)
    }

    /// The definition a usage names, chosen by what kind of usage it is.
    ///
    /// Preferences are tiers: every candidate is offered to the first tier before the second is
    /// tried, so `obj.field()` reaches a method rather than a field declared earlier in the file.
    /// Within a tier, declaration order decides.
    fn select_best_definition_by_priority<'a>(
        &self,
        matching_definitions: &[&'a Definition],
        usage: &Usage,
    ) -> Option<&'a Definition> {
        // An import is what a name in `main` reaches, since that is where a `use` was written for.
        if self.is_usage_in_main_function(usage) {
            if let Some(imported) = first_of(matching_definitions, IMPORTED) {
                return Some(imported);
            }
        }

        if let Some(preferred) = preference_for(usage.kind.clone())
            .and_then(|tiers| first_of(matching_definitions, tiers))
        {
            return Some(preferred);
        }

        // A bare identifier names the nearest binding in scope, and a type parameter belongs to the
        // generic item that declares it, so proximity decides before the ladder gets a say.
        if matches!(
            usage.kind,
            UsageKind::Identifier | UsageKind::TypeIdentifier
        ) {
            if let Some(nearest) = self.select_nearest_in_scope_chain(usage, matching_definitions) {
                return Some(nearest);
            }
        }

        first_of(matching_definitions, LADDER)
            .or_else(|| self.nearest_preceding_local(matching_definitions, usage))
            .or_else(|| first_of(matching_definitions, IMPORTED))
            .or_else(|| matching_definitions.first().copied())
    }

    /// The last local declared before the usage in its own function scope.
    ///
    /// Reached only once the ladder has declined, so this is about a plain variable rather than
    /// anything a kind preference would have claimed.
    fn nearest_preceding_local<'a>(
        &self,
        matching_definitions: &[&'a Definition],
        usage: &Usage,
    ) -> Option<&'a Definition> {
        matching_definitions
            .iter()
            .filter(|def| def.definition_type == DefinitionType::VariableDefinition)
            .filter(|def| {
                ScopeUtilities::are_in_same_function_scope(&self.symbol_table, usage, def)
            })
            .filter(|def| precedes(def, usage))
            .max_by_key(|def| (def.position.start_line, def.position.start_column))
            .copied()
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
}

/// Definition kinds to prefer, most specific tier first.
type Tiers = &'static [&'static [DefinitionType]];

/// What each usage kind reaches before the general ladder is consulted.
///
/// A call reaches a method or a plain function, whichever is declared first — both are things that
/// can be called. A field expression is not symmetrical that way: a method wins over a field of the
/// same name wherever each is declared, because a call is what the syntax says.
fn preference_for(kind: UsageKind) -> Option<Tiers> {
    match kind {
        UsageKind::CallExpression => Some(&[&[
            DefinitionType::MethodDefinition,
            DefinitionType::FunctionDefinition,
        ]]),
        // A field named by a struct literal can only be a field declaration, so there is no method
        // to prefer over it.
        UsageKind::FieldInitializer => Some(&[&[DefinitionType::StructFieldDefinition]]),
        UsageKind::FieldExpression => Some(&[
            &[DefinitionType::MethodDefinition],
            &[DefinitionType::StructFieldDefinition],
        ]),
        _ => None,
    }
}

/// What anything reaches once its own kind has no preference: the most specific declaration first.
const LADDER: Tiers = &[
    &[DefinitionType::ModuleDefinition],
    &[DefinitionType::FunctionDefinition],
    &[DefinitionType::MethodDefinition],
    &[DefinitionType::ConstDefinition],
    &[DefinitionType::StructDefinition],
];

/// An import is both the first thing a name in `main` reaches and the last resort elsewhere.
const IMPORTED: Tiers = &[&[DefinitionType::ImportDefinition]];

/// The first candidate matching the earliest tier that matches anything.
fn first_of<'a>(definitions: &[&'a Definition], tiers: Tiers) -> Option<&'a Definition> {
    tiers.iter().find_map(|tier| {
        definitions
            .iter()
            .find(|def| tier.contains(&def.definition_type))
            .copied()
    })
}

/// Whether this declaration comes before the usage in the file.
fn precedes(definition: &Definition, usage: &Usage) -> bool {
    (
        definition.position.start_line,
        definition.position.start_column,
    ) < (usage.position.start_line, usage.position.start_column)
}
