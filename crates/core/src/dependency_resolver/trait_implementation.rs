use crate::models::{Dependency, DependencyType};
use std::collections::HashMap;
use streaming_iterator::StreamingIterator;
use tree_sitter::{Language, Node, Query, QueryCursor, QueryMatch};

/// Queries locating the two sides of an implementation relationship in one language.
///
/// Each must capture `@type` for the trait or interface being named, and `@method` for the method
/// name. Everything else about the relationship is language independent.
pub struct Queries {
    /// Methods an implementing block provides, captured with the type it implements.
    pub implementations: &'static str,
    /// Methods a trait or interface declares, captured with its own name.
    pub declarations: &'static str,
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

    Ok(
        methods(language, queries.implementations, source_code, root_node)?
            .iter()
            .filter_map(|method| {
                declared
                    .get(&(method.type_name.clone(), method.name.clone()))
                    .map(|line| dependency(method, *line))
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

fn methods(
    language: &Language,
    query_source: &str,
    source_code: &str,
    root_node: Node,
) -> Result<Vec<Method>, String> {
    let query = Query::new(language, query_source)
        .map_err(|error| format!("Failed to create trait implementation query: {error}"))?;

    let type_index = capture_index(&query, "type")?;
    let method_index = capture_index(&query, "method")?;

    let mut cursor = QueryCursor::new();
    let mut matches = cursor.matches(&query, root_node, source_code.as_bytes());
    let mut found = Vec::new();

    while let Some(query_match) = matches.next() {
        let type_node = capture(query_match, type_index);
        let method_node = capture(query_match, method_index);

        if let (Some(type_node), Some(method_node)) = (type_node, method_node) {
            if let Some(method) = method(source_code, type_node, method_node) {
                found.push(method);
            }
        }
    }

    Ok(found)
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
