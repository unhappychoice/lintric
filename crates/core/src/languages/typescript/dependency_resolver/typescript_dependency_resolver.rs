use crate::dependency_resolver::DependencyResolverTrait;
use crate::models::{
    scope::{CodeAnalysisContext, SymbolTable},
    Definition, Dependency, Usage,
};
use tree_sitter::Node;

use super::accessor_direction::AccessorDirection;
use super::method_resolver::MethodResolver;
use super::module_resolver::ModuleResolver;
use crate::dependency_resolver::receiver_narrowing::ReceiverNarrowing;
use crate::dependency_resolver::self_reference::SelfReference;
use crate::query;

/// Names an export clause exposes, which belong to either namespace.
const EXPORT_SPECIFIERS: &str =
    include_str!("../../../../queries/typescript/export_specifiers.scm");

/// Each declarator paired with its initializer, so a binding stays out of the names it reads.
const OWN_INITIALIZERS: &str = include_str!("../../../../queries/typescript/own_initializers.scm");

/// TypeScript-specific dependency resolver
pub struct TypeScriptDependencyResolver {
    symbol_table: SymbolTable,
    method_resolver: MethodResolver,
    module_resolver: ModuleResolver,
}

impl TypeScriptDependencyResolver {
    pub fn new(symbol_table: SymbolTable) -> Self {
        Self {
            symbol_table,
            method_resolver: MethodResolver::new(),
            module_resolver: ModuleResolver::new(),
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

    /// TypeScript-specific field access resolution
    fn resolve_typescript_field_access(
        &self,
        narrowing: &ReceiverNarrowing,
        usage_node: &Usage,
        definitions: &[Definition],
    ) -> Vec<Dependency> {
        self.method_resolver
            .resolve_struct_field_access(usage_node, definitions, narrowing)
    }

    /// Whether this declaration can be named from the position this usage sits in.
    ///
    /// TypeScript keeps types and values in separate namespaces, so an interface and a `const` may
    /// share a name and each is invisible where the other belongs. A class, an enum and a namespace
    /// declare in both, which is why the rule is about what a declaration introduces rather than
    /// about matching a type usage to a type declaration.
    fn is_in_usage_namespace(
        exported: &std::collections::HashSet<(usize, usize)>,
        usage: &Usage,
        definition: &Definition,
    ) -> bool {
        use crate::models::DefinitionType::*;

        // An export names a local declaration of either kind, and `export type { X }` reads as an
        // ordinary identifier, so the two namespaces are not kept apart there.
        if exported.contains(&(usage.position.start_line, usage.position.start_column)) {
            return true;
        }

        match definition.definition_type {
            InterfaceDefinition | TypeDefinition => {
                usage.kind == crate::models::UsageKind::TypeIdentifier
            }
            VariableDefinition | ConstDefinition | FunctionDefinition => {
                usage.kind != crate::models::UsageKind::TypeIdentifier
            }
            _ => true,
        }
    }

    /// Whether a member would be reached by its bare name.
    ///
    /// `receiver.member` reaches only what the receiver's type declares, which the field access
    /// path above decides. Matching the name here as well would undo that decision and resolve
    /// `first.id` to any type's `id`.
    fn is_member_reached_by_name(usage: &Usage, definition: &Definition) -> bool {
        usage.kind == crate::models::UsageKind::FieldExpression
            && matches!(
                definition.definition_type,
                crate::models::DefinitionType::StructFieldDefinition
                    | crate::models::DefinitionType::PropertyDefinition
            )
    }

    /// Check if definition is accessible from usage (TypeScript-specific rules)
    fn is_accessible_basic(&self, usage: &Usage, definition: &Definition) -> bool {
        // A function, class, interface, enum, type alias or namespace is visible before its own
        // line, so where it sits does not restrict who can name it.
        if self.is_hoisted_basic(definition) {
            return true;
        }

        // A member is reached through a receiver rather than by a name in scope, so where it sits
        // does not restrict who can name it either.
        if Self::is_member(definition) {
            return true;
        }

        // Everything else is reachable only from inside the scope that declares it. `const` and
        // `let` are block-scoped, so a binding inside a block is invisible outside it.
        self.is_in_scope_chain(usage, definition)
    }

    fn is_member(definition: &Definition) -> bool {
        use crate::models::DefinitionType;
        matches!(
            definition.definition_type,
            DefinitionType::MethodDefinition | DefinitionType::PropertyDefinition
        )
    }

    /// Whether the definition sits in the usage's scope or one enclosing it.
    ///
    /// A declaration that opens a scope has its own name recorded inside that scope rather than
    /// beside it, so the parent counts too — otherwise no such declaration would look reachable.
    fn is_in_scope_chain(&self, usage: &Usage, definition: &Definition) -> bool {
        let chain = self.usage_scope_chain(usage);

        definition
            .scope_id
            .is_some_and(|def_scope| chain.contains(&def_scope))
    }

    /// The usage's own scope followed by its enclosing scopes.
    fn usage_scope_chain(&self, usage: &Usage) -> Vec<crate::models::ScopeId> {
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
        match definition.definition_type {
            // In TypeScript, function declarations and classes are hoisted
            DefinitionType::FunctionDefinition => true,
            DefinitionType::TypeDefinition => true,
            DefinitionType::InterfaceDefinition => true,
            DefinitionType::ClassDefinition => true,
            DefinitionType::EnumDefinition => true,
            DefinitionType::ModuleDefinition => true,
            _ => false,
        }
    }

    fn select_preferred_definition_typescript_aware<'a>(
        &self,
        usage_node: &Usage,
        matching_definitions: &[&'a Definition],
    ) -> Option<&'a Definition> {
        self.module_resolver
            .select_preferred_definition(usage_node, matching_definitions)
    }

    /// Select the closest type parameter definition for TypeScript type identifiers
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

        // Find the closest preceding type parameter definition
        type_defs
            .iter()
            .filter(|def| def.position.start_line <= usage_node.position.start_line)
            .max_by_key(|def| def.position.start_line)
            .copied()
    }

    #[allow(dead_code)]
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

impl DependencyResolverTrait for TypeScriptDependencyResolver {
    fn resolve_dependencies(
        &self,
        source_code: &str,
        root_node: Node,
        usage_nodes: &[Usage],
        definitions: &[Definition],
    ) -> Result<Vec<Dependency>, String> {
        // Read off the file once rather than per usage: every member access asks the same questions
        // of it, and a malformed query must fail rather than quietly resolve nothing.
        let narrowing =
            ReceiverNarrowing::new(&super::receiver_narrowing::DIALECT, source_code, root_node)?;
        let direction = AccessorDirection::new(source_code, root_node)?;
        let own = SelfReference::new(OWN_INITIALIZERS, source_code, root_node)?;
        let exported =
            query::captured_positions(EXPORT_SPECIFIERS, source_code, root_node, "exported")?;

        let mut all_dependencies: Vec<Dependency> = usage_nodes
            .iter()
            .flat_map(|usage| {
                self.resolve_single_dependency(
                    &narrowing,
                    &direction,
                    &own,
                    &exported,
                    usage,
                    definitions,
                )
            })
            .collect();

        // Add interface implementation dependencies (class method -> interface declaration), which
        // have no usage to resolve and are derived from the class heritage instead
        let implementation_deps =
            super::interface_implementation_resolver::resolve(source_code, root_node)?;
        all_dependencies.extend(implementation_deps);

        Ok(all_dependencies)
    }
}

impl TypeScriptDependencyResolver {
    /// Resolve one usage. Internal to this resolver: the Rust side reaches its own equivalent by a
    /// different route, so there is nothing for the trait to abstract over.
    fn resolve_single_dependency(
        &self,
        narrowing: &ReceiverNarrowing,
        direction: &AccessorDirection,
        own: &SelfReference,
        exported: &std::collections::HashSet<(usize, usize)>,
        usage_node: &Usage,
        definitions: &[Definition],
    ) -> Vec<Dependency> {
        let mut dependencies = Vec::new();

        // Try TypeScript-specific field access resolution
        if usage_node.kind == crate::models::UsageKind::FieldExpression {
            let field_dependencies =
                self.resolve_typescript_field_access(narrowing, usage_node, definitions);
            if !field_dependencies.is_empty() {
                dependencies.extend(field_dependencies);
                return dependencies;
            }
        }

        // Find matching definitions with TypeScript-specific filtering
        let all_matching_definitions: Vec<&Definition> = definitions
            .iter()
            .filter(|def| def.name == usage_node.name)
            .collect();

        let accessible: Vec<&Definition> = all_matching_definitions
            .into_iter()
            // A binding is not among the candidates for its own initializer, so `let x = x + 1`
            // looks past it and finds the previous `x`.
            .filter(|def| !own.declares(usage_node, def))
            .filter(|def| Self::is_in_usage_namespace(exported, usage_node, def))
            .filter(|def| !Self::is_member_reached_by_name(usage_node, def))
            .filter(|def| self.is_accessible_basic(usage_node, def))
            .filter(|def| self.module_resolver.is_valid_dependency(usage_node, def))
            .collect();

        // A getter and a setter share a name, so which one is reached is decided by whether this
        // access reads or writes rather than by any preference among them.
        let matching_definitions = direction.narrow(usage_node, accessible);

        // Apply TypeScript-specific preference logic
        let preferred_definition = if usage_node.kind == crate::models::UsageKind::TypeIdentifier {
            // For type identifiers, prefer the most local type parameter definition
            self.select_closest_type_parameter(usage_node, &matching_definitions)
                .or_else(|| {
                    self.select_preferred_definition_typescript_aware(
                        usage_node,
                        &matching_definitions,
                    )
                })
        } else {
            self.select_preferred_definition_typescript_aware(usage_node, &matching_definitions)
        };

        if let Some(definition) = preferred_definition {
            let source_line = usage_node.position.start_line;
            let target_line = definition.position.start_line;

            if source_line != target_line {
                let dependency = Dependency {
                    source_line,
                    target_line,
                    symbol: usage_node.name.clone(),
                    dependency_type: self.get_dependency_type(usage_node, definition),
                    context: self.get_context(usage_node),
                };
                dependencies.push(dependency);
            }
        }

        dependencies
    }
}
