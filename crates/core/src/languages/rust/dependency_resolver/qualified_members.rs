use super::import_lookup::path_segments;
use super::module_scopes::ModuleScopes;
use super::rust_dependency_resolver::RustDependencyResolver;
use crate::models::{Definition, DefinitionType, Position, Usage, UsageKind};
use crate::query;
use std::collections::{HashMap, HashSet};
use tree_sitter::Node;

type Location = (usize, usize);
type References = HashMap<Location, TypeReference>;

/// AST receiver narrowing for associated items, with declaration identity as the owner.
pub(super) struct QualifiedMembers {
    qualifiers: References,
    owners: References,
    aliases: References,
    parameters: References,
    trait_impls: Vec<(TypeReference, TypeReference)>,
}

impl QualifiedMembers {
    pub(super) fn new(source: &str, root: Node) -> Result<Self, String> {
        Ok(Self {
            qualifiers: references(QUALIFIERS, source, root)?,
            owners: references(OWNERS, source, root)?,
            aliases: references(
                "(type_item name: (_) @member type: (_) @owner)",
                source,
                root,
            )?,
            parameters: references("(type_parameter name: (_) @owner @member)", source, root)?,
            trait_impls: trait_implementations(source, root)?,
        })
    }

    pub(super) fn narrow<'a>(
        &self,
        resolver: &RustDependencyResolver,
        usage: &Usage,
        candidates: Vec<&'a Definition>,
        definitions: &[Definition],
    ) -> Vec<&'a Definition> {
        let Some(qualifier) = self.qualifiers.get(&location(&usage.position)) else {
            return candidates;
        };
        // Self retains its existing scope-aware resolution.
        if qualifier.usage.name == "Self" {
            return candidates;
        }
        let owner = match self.resolve(resolver, qualifier, definitions, &mut HashSet::new()) {
            Resolution::Resolved(owner) => owner,
            // Unsupported access syntax keeps the fallback, but an opaque alias target
            // does not establish ownership of any same-named member.
            Resolution::Unsupported if qualifier.path.is_none() => return candidates,
            Resolution::Unsupported | Resolution::Unresolved => return Vec::new(),
        };
        if owner.uses_fallback(&self.parameters) {
            return candidates;
        }
        candidates
            .into_iter()
            .filter(|candidate| {
                self.owners
                    .get(&location(&candidate.position))
                    .and_then(|reference| {
                        self.resolve(resolver, reference, definitions, &mut HashSet::new())
                            .owner()
                    })
                    .is_some_and(|declared| {
                        declared == owner
                            || self.implements(resolver, &owner, &declared, definitions)
                    })
            })
            .collect()
    }

    fn implements(
        &self,
        resolver: &RustDependencyResolver,
        owner: &Owner,
        declared: &Owner,
        definitions: &[Definition],
    ) -> bool {
        self.trait_impls.iter().any(|(target, implemented)| {
            self.resolve(resolver, target, definitions, &mut HashSet::new())
                .owner()
                .is_some_and(|resolved| &resolved == owner)
                && self
                    .resolve(resolver, implemented, definitions, &mut HashSet::new())
                    .owner()
                    .is_some_and(|resolved| &resolved == declared)
        })
    }

    fn resolve(
        &self,
        resolver: &RustDependencyResolver,
        reference: &TypeReference,
        definitions: &[Definition],
        visited: &mut HashSet<Location>,
    ) -> Resolution {
        let Some((name, path)) = reference.path.as_ref().and_then(|path| path.split_last()) else {
            return Resolution::Unsupported;
        };
        if path.is_empty() && self.is_primitive(reference) {
            return Resolution::Resolved(Owner::Primitive(name.clone()));
        }
        let candidates: Vec<_> = definitions
            .iter()
            .filter(|def| {
                &def.name == name
                    && matches!(
                        def.definition_type,
                        DefinitionType::StructDefinition
                            | DefinitionType::EnumDefinition
                            | DefinitionType::TypeDefinition
                            | DefinitionType::ModuleDefinition
                            | DefinitionType::ImportDefinition
                    )
            })
            .collect();
        let Some(definition) = lookup(resolver, &reference.usage, path, &candidates) else {
            return Resolution::Unresolved;
        };
        if !visited.insert(location(&definition.position))
            || definition.definition_type == DefinitionType::ImportDefinition
        {
            return Resolution::Unresolved;
        }
        match self.aliases.get(&location(&definition.position)) {
            Some(target) => self.resolve(resolver, target, definitions, visited),
            None => Resolution::Resolved(Owner::Declaration(
                location(&definition.position),
                definition.definition_type.clone(),
            )),
        }
    }

    fn is_primitive(&self, reference: &TypeReference) -> bool {
        reference.usage.context.as_deref() == Some("primitive_type")
            || self.trait_impls.iter().any(|(target, _)| {
                target.usage.context.as_deref() == Some("primitive_type")
                    && target.usage.name == reference.usage.name
            })
    }
}

enum Resolution {
    Resolved(Owner),
    Unsupported,
    Unresolved,
}

impl Resolution {
    fn owner(self) -> Option<Owner> {
        match self {
            Self::Resolved(owner) => Some(owner),
            Self::Unsupported | Self::Unresolved => None,
        }
    }
}

#[derive(PartialEq)]
enum Owner {
    Declaration(Location, DefinitionType),
    Primitive(String),
}

impl Owner {
    fn uses_fallback(&self, parameters: &References) -> bool {
        match self {
            Self::Declaration(position, kind) => {
                *kind == DefinitionType::ModuleDefinition || parameters.contains_key(position)
            }
            Self::Primitive(_) => false,
        }
    }
}

struct TypeReference {
    usage: Usage,
    path: Option<Vec<String>>,
}

const QUALIFIERS: &str = "[
    (scoped_identifier path: (_) @owner name: (_) @member)
    (scoped_type_identifier path: (_) @owner name: (_) @member)
]";
const OWNERS: &str = include_str!("../../../../queries/rust/associated_owners.scm");

fn trait_implementations(
    source: &str,
    root: Node,
) -> Result<Vec<(TypeReference, TypeReference)>, String> {
    query::map_pairs(
        "(impl_item trait: (_) @trait type: (_) @owner)",
        source,
        root,
        "owner",
        "trait",
        |owner, implemented| {
            Some((
                type_reference(owner, source),
                type_reference(implemented, source),
            ))
        },
    )
}

fn references(query: &str, source: &str, root: Node) -> Result<References, String> {
    Ok(
        query::map_pairs(query, source, root, "owner", "member", |owner, member| {
            Some((
                location(&Position::from_node(&member)),
                type_reference(owner, source),
            ))
        })?
        .into_iter()
        .collect(),
    )
}

fn type_reference(node: Node, source: &str) -> TypeReference {
    let node = match node.kind() {
        "generic_type" => node.child_by_field_name("type").unwrap_or(node),
        _ => node,
    };
    let mut reference = Usage::new(&node, source, UsageKind::TypeIdentifier);
    reference.context = Some(node.kind().to_string());
    TypeReference {
        path: match node.kind() {
            "primitive_type" => Some(vec![reference.name.clone()]),
            _ => path_segments(node, source).filter(|path| {
                !(path.len() > 1 && path.first().is_some_and(|head| head == "Self"))
            }),
        },
        usage: reference,
    }
}

fn lookup<'a>(
    resolver: &RustDependencyResolver,
    usage: &Usage,
    path: &[String],
    candidates: &[&'a Definition],
) -> Option<&'a Definition> {
    if path.is_empty() {
        return resolver.select_nearest_in_scope_chain(usage, candidates);
    }
    let scopes = &resolver.symbol_table.scopes;
    let scope = scopes.find_scope_at_position(&usage.position)?;
    let module = ModuleScopes::new(scopes).resolve(path, scope)?;
    candidates.iter().copied().find(|definition| {
        definition
            .scope_id
            .and_then(|id| scopes.get_scope(id))
            .is_some_and(|scope| {
                scope.id == module
                    || (scope.parent == Some(module)
                        && !matches!(
                            scope.scope_type,
                            crate::models::ScopeType::Module | crate::models::ScopeType::Global
                        ))
            })
    })
}

fn location(position: &Position) -> Location {
    (position.start_line, position.start_column)
}
