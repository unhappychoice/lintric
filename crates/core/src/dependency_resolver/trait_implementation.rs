use crate::models::{Dependency, DependencyType};
use std::collections::{HashMap, HashSet, VecDeque};
use streaming_iterator::StreamingIterator;
use tree_sitter::{Language, Node, Query, QueryCursor, QueryMatch};

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
    // Taking the language from the tree keeps callers from having to name it, and makes dialects
    // such as TSX work without a separate entry point.
    let language = &*root_node.language();
    let declared = declarations(language, queries.declarations, source_code, root_node)?;
    let inherited = supertypes(language, queries.supertypes, source_code, root_node)?;

    Ok(
        methods(language, queries.implementations, source_code, root_node)?
            .iter()
            .filter_map(|method| {
                declaration_line(&declared, &inherited, method).map(|line| dependency(method, line))
            })
            .filter(|dependency| dependency.source_line != dependency.target_line)
            .collect(),
    )
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
    language: &Language,
    query_source: &str,
    source_code: &str,
    root_node: Node,
) -> Result<HashMap<(String, String), usize>, String> {
    Ok(methods(language, query_source, source_code, root_node)?
        .into_iter()
        .map(|method| ((method.type_name, method.name), method.line))
        .collect())
}

fn supertypes(
    language: &Language,
    query_source: &str,
    source_code: &str,
    root_node: Node,
) -> Result<HashMap<String, Vec<String>>, String> {
    let pairs = map_pairs(
        language,
        query_source,
        source_code,
        root_node,
        "super",
        |type_node, super_node| {
            Some((
                text(source_code, type_node)?,
                text(source_code, super_node)?,
            ))
        },
    )?;

    let mut inherited: HashMap<String, Vec<String>> = HashMap::new();
    for (type_name, super_name) in pairs {
        inherited.entry(type_name).or_default().push(super_name);
    }

    Ok(inherited)
}

fn methods(
    language: &Language,
    query_source: &str,
    source_code: &str,
    root_node: Node,
) -> Result<Vec<Method>, String> {
    map_pairs(
        language,
        query_source,
        source_code,
        root_node,
        "method",
        |type_node, method_node| method(source_code, type_node, method_node),
    )
}

/// Map the `@type` node paired with the other named capture, for every match of the query.
///
/// The mapping happens inside the loop because a captured node's lifetime is tied to the query
/// cursor, so nodes cannot outlive this call.
fn map_pairs<T>(
    language: &Language,
    query_source: &str,
    source_code: &str,
    root_node: Node,
    second: &str,
    mut map: impl FnMut(Node, Node) -> Option<T>,
) -> Result<Vec<T>, String> {
    let query = Query::new(language, query_source)
        .map_err(|error| format!("Failed to create trait implementation query: {error}"))?;

    let type_index = capture_index(&query, "type")?;
    let second_index = capture_index(&query, second)?;

    let mut cursor = QueryCursor::new();
    let mut matches = cursor.matches(&query, root_node, source_code.as_bytes());
    let mut mapped = Vec::new();

    while let Some(query_match) = matches.next() {
        let type_node = capture(query_match, type_index);
        let second_node = capture(query_match, second_index);

        if let (Some(type_node), Some(second_node)) = (type_node, second_node) {
            mapped.extend(map(type_node, second_node));
        }
    }

    Ok(mapped)
}

fn capture_index(query: &Query, name: &str) -> Result<u32, String> {
    query
        .capture_index_for_name(name)
        .ok_or_else(|| format!("Query is missing the @{name} capture"))
}

fn capture<'a>(query_match: &QueryMatch<'a, 'a>, index: u32) -> Option<Node<'a>> {
    query_match
        .captures
        .iter()
        .find(|capture| capture.index == index)
        .map(|capture| capture.node)
}

fn method(source_code: &str, type_node: Node, method_node: Node) -> Option<Method> {
    Some(Method {
        type_name: text(source_code, type_node)?,
        name: text(source_code, method_node)?,
        line: method_node.start_position().row + 1,
    })
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
