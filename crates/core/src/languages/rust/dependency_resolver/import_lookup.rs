use crate::models::{Usage, UsageKind};
use crate::query;
use std::collections::HashSet;
use tree_sitter::Node;

const QUALIFIED_NAMES: &str = include_str!("../../../../queries/rust/qualified_names.scm");

/// AST roles that prevent a usage from naming a lexical import.
pub(super) struct ImportLookup {
    qualified_names: HashSet<(usize, usize)>,
}

impl ImportLookup {
    pub(super) fn new(source_code: &str, root_node: Node) -> Result<Self, String> {
        Ok(Self {
            qualified_names: query::captured_positions(
                QUALIFIED_NAMES,
                source_code,
                root_node,
                "qualified",
            )?,
        })
    }

    pub(super) fn allows(&self, usage: &Usage) -> bool {
        // The extractor records the member as FieldExpression and its receiver separately.
        usage.kind != UsageKind::FieldExpression
            && !self
                .qualified_names
                .contains(&(usage.position.start_line, usage.position.start_column))
    }
}
