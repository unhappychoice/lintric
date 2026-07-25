//! Choosing between a getter and a setter by what the access does.
//!
//! A getter and a setter of the same name are one member declared in two places, and the receiver's
//! type cannot tell them apart — both belong to the same class. What tells them apart is direction:
//! reading reaches the getter and assigning reaches the setter.
//!
//! `o.p += v` reads the old value before writing the new one, so it reaches both. Only the setter is
//! recorded, because the resolver records one target per usage; the getter edge is the cost of that
//! and is pinned as such in the fixtures.

use crate::models::{Definition, Usage};
use crate::query;
use std::collections::HashSet;
use tree_sitter::Node;

const ACCESSORS: &str = include_str!("../../../../queries/typescript/accessors.scm");
const WRITTEN_ACCESSES: &str = include_str!("../../../../queries/typescript/written_accesses.scm");

type Positions = HashSet<(usize, usize)>;

/// Which declarations are accessors, and which accesses write, read off the file once.
pub struct AccessorDirection {
    getters: Positions,
    setters: Positions,
    written: Positions,
    modified: Positions,
}

impl AccessorDirection {
    pub fn new(source_code: &str, root_node: Node) -> Result<Self, String> {
        Ok(Self {
            getters: positions(ACCESSORS, source_code, root_node, "getter")?,
            setters: positions(ACCESSORS, source_code, root_node, "setter")?,
            written: positions(WRITTEN_ACCESSES, source_code, root_node, "written")?,
            modified: positions(WRITTEN_ACCESSES, source_code, root_node, "modified")?,
        })
    }

    /// Keep the accessor this access reaches.
    ///
    /// Only a getter and setter sharing a name are ambiguous in this way, so anything else is left
    /// alone — including a lone accessor, which an access reaches whichever way it goes.
    pub fn narrow<'a>(
        &self,
        usage: &Usage,
        candidates: Vec<&'a Definition>,
    ) -> Vec<&'a Definition> {
        let position = (usage.position.start_line, usage.position.start_column);
        if !self.is_accessor_pair(&candidates) {
            return candidates;
        }

        let writes = self.written.contains(&position) || self.modified.contains(&position);
        let unreached = if writes { &self.getters } else { &self.setters };

        candidates
            .into_iter()
            .filter(|candidate| !unreached.contains(&position_of(candidate)))
            .collect()
    }

    fn is_accessor_pair(&self, candidates: &[&Definition]) -> bool {
        let positions = || candidates.iter().copied().map(position_of);

        positions().any(|position| self.getters.contains(&position))
            && positions().any(|position| self.setters.contains(&position))
    }
}

fn positions(
    query_source: &str,
    source_code: &str,
    root_node: Node,
    capture: &str,
) -> Result<Positions, String> {
    query::captured_positions(query_source, source_code, root_node, capture)
}

fn position_of(definition: &Definition) -> (usize, usize) {
    (
        definition.position.start_line,
        definition.position.start_column,
    )
}
