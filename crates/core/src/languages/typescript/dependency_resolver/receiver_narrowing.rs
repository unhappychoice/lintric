//! Pointing `receiver.member` at the member the receiver's type declares.
//!
//! Two types may declare a member of the same name, and matching on the name alone links an access
//! to both — inventing a relationship with a type the code never mentions. The receiver's type is
//! what tells them apart, and a single file knows it only where it is written down: an annotated
//! parameter or variable, or `this` inside a class. Where it is not written down, nothing is
//! claimed.

use crate::models::{Definition, Usage};
use crate::query::{self, NamedSpan};
use std::collections::HashMap;
use tree_sitter::Node;

const MEMBER_OWNERS: &str = include_str!("../../../../queries/typescript/member_owners.scm");
const RECEIVER_TYPES: &str = include_str!("../../../../queries/typescript/receiver_types.scm");
const ENCLOSING_CLASSES: &str =
    include_str!("../../../../queries/typescript/enclosing_classes.scm");
const MEMBER_ACCESSES: &str = include_str!("../../../../queries/typescript/member_accesses.scm");

/// What each member access can be narrowed to, read off the file once.
pub struct ReceiverNarrowing {
    owner_by_member_position: HashMap<(usize, usize), String>,
    types_by_binding: HashMap<String, Vec<String>>,
    class_spans: Vec<NamedSpan>,
    receiver_by_access_position: HashMap<(usize, usize), String>,
}

impl ReceiverNarrowing {
    pub fn new(source_code: &str, root_node: Node) -> Result<Self, String> {
        Ok(Self {
            owner_by_member_position: query::text_by_position(
                MEMBER_OWNERS,
                source_code,
                root_node,
                "owner",
                "member",
            )?,
            types_by_binding: query::map_pairs(
                RECEIVER_TYPES,
                source_code,
                root_node,
                "binding",
                "annotated",
                |binding, annotation| {
                    Some((
                        binding.utf8_text(source_code.as_bytes()).ok()?.to_string(),
                        annotated_type_names(&annotation, source_code),
                    ))
                },
            )?
            .into_iter()
            .collect(),
            class_spans: query::text_by_span(
                ENCLOSING_CLASSES,
                source_code,
                root_node,
                "owner",
                "body",
            )?,
            receiver_by_access_position: query::text_by_position(
                MEMBER_ACCESSES,
                source_code,
                root_node,
                "receiver",
                "accessed",
            )?,
        })
    }

    /// Keep the candidates the receiver's type declares.
    ///
    /// One candidate is left alone: there is nothing to tell apart, and the receiver's type may
    /// well be unknowable. Several candidates with an unknown receiver type yield nothing, since
    /// every answer would be a guess and the wrong ones are edges to types the line never names.
    pub fn narrow<'a>(
        &self,
        usage: &Usage,
        candidates: Vec<&'a Definition>,
    ) -> Vec<&'a Definition> {
        if candidates.len() <= 1 {
            return candidates;
        }

        let Some(owners) = self.receiver_types(usage) else {
            return Vec::new();
        };

        candidates
            .into_iter()
            .filter(|candidate| {
                self.owner_of(candidate)
                    .is_some_and(|owner| owners.contains(owner))
            })
            .collect()
    }

    /// The types of what the access reads from, where the file states them. A union states several,
    /// and the member may be declared by any of them.
    ///
    /// A usage names only the member, so the receiver is found by the position the two share.
    fn receiver_types(&self, usage: &Usage) -> Option<Vec<String>> {
        let position = (usage.position.start_line, usage.position.start_column);

        match self.receiver_by_access_position.get(&position)?.as_str() {
            "this" => self
                .enclosing_class(usage.position.start_line)
                .map(|class_name| vec![class_name]),
            binding => self.types_by_binding.get(binding).cloned(),
        }
    }

    /// The innermost class whose body contains the line, so a nested class wins over the one it
    /// sits in.
    fn enclosing_class(&self, line: usize) -> Option<String> {
        self.class_spans
            .iter()
            .filter(|(_, (start, end))| *start <= line && line <= *end)
            .min_by_key(|(_, (start, end))| end - start)
            .map(|(owner, _)| owner.clone())
    }

    fn owner_of(&self, definition: &Definition) -> Option<&String> {
        self.owner_by_member_position.get(&(
            definition.position.start_line,
            definition.position.start_column,
        ))
    }
}

/// The type names an annotation states.
///
/// Type arguments are not descended into: `Wrapper<Reader>` states `Wrapper`, and its members are
/// the ones a receiver of that type reaches.
fn annotated_type_names(node: &Node, source_code: &str) -> Vec<String> {
    if node.kind() == "type_identifier" {
        return node
            .utf8_text(source_code.as_bytes())
            .map(|name| vec![name.to_string()])
            .unwrap_or_default();
    }

    (0..node.child_count())
        .filter_map(|index| node.child(index))
        .filter(|child| child.kind() != "type_arguments")
        .flat_map(|child| annotated_type_names(&child, source_code))
        .collect()
}
