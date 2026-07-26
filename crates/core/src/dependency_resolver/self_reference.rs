//! Keeping a binding out of the names its own initializer reads.
//!
//! `let w = w + 1` reads the **previous** `w`: the binding starts after the statement, so it is not
//! among the candidates for anything inside its own initializer. Where the previous one is in an
//! enclosing scope, resolution must be allowed to look past the inner declaration and find it.
//!
//! Position alone cannot express this. The declaration is on the same line as the usage, to its left,
//! in a scope the usage can see — exactly like a closure parameter in `|x| x + 1` and a generic in
//! `impl<const N: usize> Buffer<N>`, both of which the usage *does* name. What distinguishes them is
//! structural: the usage lies inside the declaration's own initializer. So it is read off the tree
//! and recorded as a pair of positions.

use crate::models::{Definition, Usage};
use crate::query;
use std::collections::{HashMap, HashSet};
use tree_sitter::Node;

type Position = (usize, usize);

/// Which declaration each usage must not resolve to.
pub struct SelfReference {
    declared_by_usage: HashMap<Position, HashSet<Position>>,
}

impl SelfReference {
    pub fn new(query_source: &str, source_code: &str, root_node: Node) -> Result<Self, String> {
        let pairs = query::map_pairs(
            query_source,
            source_code,
            root_node,
            "declared",
            "initializer",
            |declared, initializer| Some((position(declared), read_positions(initializer))),
        )?;

        Ok(Self {
            declared_by_usage: pairs.into_iter().fold(
                HashMap::new(),
                |mut by_usage, (declared, reads)| {
                    reads.into_iter().for_each(|read| {
                        by_usage.entry(read).or_default().insert(declared);
                    });
                    by_usage
                },
            ),
        })
    }

    /// Whether this definition is one the usage cannot be naming: the binding whose initializer the
    /// usage sits in, or the declaration the usage itself is.
    ///
    /// `use my_module::MY_CONST` both names the const and creates an import of it, at one position.
    /// The name refers to the declaration, so the import it creates is not a candidate for it.
    pub fn declares(&self, usage: &Usage, definition: &Definition) -> bool {
        let same_node = usage.position.start_line == definition.position.start_line
            && usage.position.start_column == definition.position.start_column;

        same_node || self.inside_initializer(usage, definition)
    }

    fn inside_initializer(&self, usage: &Usage, definition: &Definition) -> bool {
        self.declared_by_usage
            .get(&(usage.position.start_line, usage.position.start_column))
            .is_some_and(|declared| {
                declared.contains(&(
                    definition.position.start_line,
                    definition.position.start_column,
                ))
            })
    }
}

/// Every identifier an initializer reads, at any depth.
fn read_positions(node: Node) -> Vec<Position> {
    if matches!(node.kind(), "identifier" | "type_identifier") {
        return vec![position(node)];
    }

    (0..node.child_count())
        .filter_map(|index| node.child(index))
        .flat_map(read_positions)
        .collect()
}

fn position(node: Node) -> Position {
    (
        node.start_position().row + 1,
        node.start_position().column + 1,
    )
}
