use super::module_scopes::ModuleScopes;
use super::nested_scope_resolver::ScopeUtilities;
use crate::models::{Definition, SymbolTable, Usage, UsageKind};
use crate::query;
use std::collections::HashMap;
use tree_sitter::Node;

const QUALIFIED_NAMES: &str = include_str!("../../../../queries/rust/qualified_names.scm");

/// The qualifier of each import reference, or None for an unsupported path.
pub(super) struct ImportLookup {
    qualifiers: HashMap<(usize, usize), Option<Vec<String>>>,
}

impl ImportLookup {
    pub(super) fn new(source_code: &str, root_node: Node) -> Result<Self, String> {
        Ok(Self {
            qualifiers: query::map_pairs(
                QUALIFIED_NAMES,
                source_code,
                root_node,
                "qualified",
                "path",
                |name, path| {
                    let position = name.start_position();
                    let qualifier = path
                        .child_by_field_name("path")
                        .and_then(|node| path_segments(node, source_code));
                    Some(((position.row + 1, position.column + 1), qualifier))
                },
            )?
            .into_iter()
            .collect(),
        })
    }

    pub(super) fn allows(
        &self,
        usage: &Usage,
        definition: &Definition,
        symbols: &SymbolTable,
    ) -> bool {
        // The extractor records the member as FieldExpression and its receiver separately.
        if usage.kind == UsageKind::FieldExpression {
            return false;
        }
        let Some(usage_scope) = symbols.scopes.find_scope_at_position(&usage.position) else {
            return false;
        };
        let Some(def_scope) = definition.scope_id else {
            return false;
        };
        match self
            .qualifiers
            .get(&(usage.position.start_line, usage.position.start_column))
        {
            Some(qualifier) => qualifier.as_ref().is_some_and(|path| {
                ModuleScopes::new(&symbols.scopes).resolve(path, usage_scope) == Some(def_scope)
            }),
            None => ScopeUtilities::is_scope_accessible(symbols, usage_scope, def_scope),
        }
    }

    pub(super) fn is_qualified(&self, usage: &Usage) -> bool {
        self.qualifiers
            .contains_key(&(usage.position.start_line, usage.position.start_column))
    }
}

fn path_segments(node: Node, source: &str) -> Option<Vec<String>> {
    match node.kind() {
        "identifier" | "type_identifier" | "self" | "super" | "crate" => {
            Some(vec![node.utf8_text(source.as_bytes()).ok()?.to_string()])
        }
        "scoped_identifier" | "scoped_type_identifier" => {
            let mut path = path_segments(node.child_by_field_name("path")?, source)?;
            path.extend(path_segments(node.child_by_field_name("name")?, source)?);
            Some(path)
        }
        _ => None,
    }
}
