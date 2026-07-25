use crate::models::{Dependency, DependencyType};
use std::collections::HashMap;
use streaming_iterator::StreamingIterator;
use tree_sitter::{Node, Query, QueryCursor, QueryMatch};

/// Methods an `impl Trait for Type` block provides.
const IMPLEMENTED_METHODS: &str = r#"
    (impl_item
      trait: (type_identifier) @trait
      body: (declaration_list
        (function_item name: (identifier) @method)))
"#;

/// Methods a trait declares. An implementation can satisfy a required signature or override a
/// method the trait already provides a body for, so both count as declarations.
const TRAIT_DECLARATIONS: &str = r#"
    (trait_item
      name: (type_identifier) @trait
      body: (declaration_list [
        (function_signature_item name: (identifier) @method)
        (function_item name: (identifier) @method)
      ]))
"#;

/// Resolve dependencies from a trait method implementation to the declaration it satisfies.
///
/// `impl Trait for Type { fn method() {} }` couples the implementation to the trait's contract:
/// changing the declaration forces every implementation to follow. The implementation contains no
/// usage of the declared name, so this relationship is derived from the structure of the impl
/// block rather than from name resolution.
pub fn resolve(source_code: &str, root_node: Node) -> Result<Vec<Dependency>, String> {
    let declarations = trait_declarations(source_code, root_node)?;

    Ok(methods(IMPLEMENTED_METHODS, source_code, root_node)?
        .iter()
        .filter_map(|method| {
            declarations
                .get(&(method.trait_name.clone(), method.name.clone()))
                .map(|line| dependency(method, *line))
        })
        .filter(|dependency| dependency.source_line != dependency.target_line)
        .collect())
}

/// A method named within a trait or one of its implementations.
struct Method {
    trait_name: String,
    name: String,
    line: usize,
}

fn trait_declarations(
    source_code: &str,
    root_node: Node,
) -> Result<HashMap<(String, String), usize>, String> {
    Ok(methods(TRAIT_DECLARATIONS, source_code, root_node)?
        .into_iter()
        .map(|method| ((method.trait_name, method.name), method.line))
        .collect())
}

fn methods(query_source: &str, source_code: &str, root_node: Node) -> Result<Vec<Method>, String> {
    let language: tree_sitter::Language = tree_sitter_rust::LANGUAGE.into();
    let query = Query::new(&language, query_source)
        .map_err(|error| format!("Failed to create trait implementation query: {error}"))?;

    let trait_index = capture_index(&query, "trait")?;
    let method_index = capture_index(&query, "method")?;

    let mut cursor = QueryCursor::new();
    let mut matches = cursor.matches(&query, root_node, source_code.as_bytes());
    let mut found = Vec::new();

    while let Some(query_match) = matches.next() {
        let trait_node = capture(query_match, trait_index);
        let method_node = capture(query_match, method_index);

        if let (Some(trait_node), Some(method_node)) = (trait_node, method_node) {
            if let Some(method) = method(source_code, trait_node, method_node) {
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

fn method(source_code: &str, trait_node: Node, method_node: Node) -> Option<Method> {
    Some(Method {
        trait_name: text(source_code, trait_node)?,
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
        context: Some(format!("trait_implementation::{}", method.trait_name)),
    }
}
