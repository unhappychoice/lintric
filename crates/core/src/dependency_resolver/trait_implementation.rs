use crate::models::{Dependency, DependencyType};
use crate::query::map_pairs;
use std::collections::{HashMap, HashSet, VecDeque};
use tree_sitter::Node;

/// Queries locating the parts of an implementation relationship in one language.
///
/// Each captures `@type` for the trait, interface or class being named, plus one other capture
/// named below. Everything else about the relationship is language independent.
pub struct Queries {
    /// Methods an implementing block provides, as `@method`, with the type it implements.
    pub implementations: &'static str,
    /// Methods a trait or interface declares, as `@method`, with its own name.
    pub declarations: &'static str,
    /// Types a trait, interface or class inherits, as `@super`, with the inheriting type's name.
    pub supertypes: &'static str,
}

/// Resolve dependencies from a method implementation to the declaration it satisfies.
///
/// An implementation couples itself to the contract it satisfies: changing the declaration forces
/// every implementation to follow. The implementation contains no usage of the declared name, so
/// this relationship is derived from the structure of the implementing block rather than from name
/// resolution.
///
/// Matching is on the pair of declaring type and method name, so two traits declaring the same
/// method resolve independently.
pub fn resolve(
    queries: &Queries,
    source_code: &str,
    root_node: Node,
) -> Result<Vec<Dependency>, String> {
    let declared = declarations(queries.declarations, source_code, root_node)?;
    let inherited = supertypes(queries.supertypes, source_code, root_node)?;

    Ok(methods(queries.implementations, source_code, root_node)?
        .iter()
        .filter_map(|method| {
            declaration_line(&declared, &inherited, method).map(|line| dependency(method, line))
        })
        .filter(|dependency| dependency.source_line != dependency.target_line)
        .collect())
}

/// A method named within a trait or interface, or within one of its implementations.
struct Method {
    type_name: String,
    name: String,
    line: usize,
}

/// The line declaring this method, looked up on the named type and then on what it inherits.
///
/// The search is breadth first, so the nearest declaration wins when a type and one of its
/// supertypes both declare the method. The visited set also keeps an inheritance cycle — invalid
/// but parseable — from looping.
fn declaration_line(
    declared: &HashMap<(String, String), usize>,
    inherited: &HashMap<String, Vec<String>>,
    method: &Method,
) -> Option<usize> {
    let mut visited: HashSet<&str> = HashSet::new();
    let mut pending: VecDeque<&str> = VecDeque::from([method.type_name.as_str()]);

    while let Some(type_name) = pending.pop_front() {
        if !visited.insert(type_name) {
            continue;
        }

        if let Some(line) = declared.get(&(type_name.to_string(), method.name.clone())) {
            return Some(*line);
        }

        if let Some(supertypes) = inherited.get(type_name) {
            pending.extend(supertypes.iter().map(String::as_str));
        }
    }

    None
}

fn declarations(
    query_source: &str,
    source_code: &str,
    root_node: Node,
) -> Result<HashMap<(String, String), usize>, String> {
    Ok(methods(query_source, source_code, root_node)?
        .into_iter()
        .map(|method| ((method.type_name, method.name), method.line))
        .collect())
}

fn supertypes(
    query_source: &str,
    source_code: &str,
    root_node: Node,
) -> Result<HashMap<String, Vec<String>>, String> {
    let pairs = map_pairs(
        query_source,
        source_code,
        root_node,
        "type",
        "super",
        |type_node, super_node| {
            Some((
                type_text(source_code, type_node)?,
                type_text(source_code, super_node)?,
            ))
        },
    )?;

    let mut inherited: HashMap<String, Vec<String>> = HashMap::new();
    for (type_name, super_name) in pairs {
        inherited.entry(type_name).or_default().push(super_name);
    }

    Ok(inherited)
}

fn methods(query_source: &str, source_code: &str, root_node: Node) -> Result<Vec<Method>, String> {
    map_pairs(
        query_source,
        source_code,
        root_node,
        "type",
        "method",
        |type_node, method_node| method(source_code, type_node, method_node),
    )
}

fn method(source_code: &str, type_node: Node, method_node: Node) -> Option<Method> {
    Some(Method {
        type_name: type_text(source_code, type_node)?,
        name: text(source_code, method_node)?,
        line: method_node.start_position().row + 1,
    })
}

/// The bare name of a captured type, looking through type arguments so that `Box<number>` pairs
/// with `Box`.
///
/// The two grammars disagree on which field holds the name, so both are tried.
fn type_text(source_code: &str, node: Node) -> Option<String> {
    let named = match node.kind() {
        "generic_type" => node
            .child_by_field_name("type")
            .or_else(|| node.child_by_field_name("name"))?,
        _ => node,
    };

    text(source_code, named)
}

fn text(source_code: &str, node: Node) -> Option<String> {
    node.utf8_text(source_code.as_bytes())
        .ok()
        .map(str::to_string)
}

fn dependency(method: &Method, target_line: usize) -> Dependency {
    Dependency {
        source_line: method.line,
        target_line,
        symbol: method.name.clone(),
        dependency_type: DependencyType::TraitImplementation,
        context: Some(format!("trait_implementation::{}", method.type_name)),
    }
}
