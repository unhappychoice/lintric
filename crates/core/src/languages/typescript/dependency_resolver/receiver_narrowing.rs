//! Pointing `receiver.member` at the member the receiver's type declares.
//!
//! Two types may declare a member of the same name, and matching on the name alone links an access
//! to both — inventing a relationship with a type the code never mentions. The receiver's type is
//! what tells them apart, and a single file knows it only where it is written down: an annotated
//! parameter or variable, an `as` at the access itself, or `this` inside a class. Where it is not
//! written down, nothing is claimed.

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
    /// The types the receiver of each access may have, keyed by the accessed member's position,
    /// which is what a `Usage` carries.
    receiver_types_by_access: HashMap<(usize, usize), Vec<String>>,
}

impl ReceiverNarrowing {
    pub fn new(source_code: &str, root_node: Node) -> Result<Self, String> {
        let annotations = Annotations::read(source_code, root_node)?;

        Ok(Self {
            owner_by_member_position: query::text_by_position(
                MEMBER_OWNERS,
                source_code,
                root_node,
                "owner",
                "member",
            )?,
            receiver_types_by_access: query::map_pairs(
                MEMBER_ACCESSES,
                source_code,
                root_node,
                "receiver",
                "accessed",
                |receiver, accessed| {
                    Some((
                        (
                            accessed.start_position().row + 1,
                            accessed.start_position().column + 1,
                        ),
                        annotations.types_of(receiver, source_code)?,
                    ))
                },
            )?
            .into_iter()
            .collect(),
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

        let Some(owners) = self
            .receiver_types_by_access
            .get(&(usage.position.start_line, usage.position.start_column))
        else {
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

    fn owner_of(&self, definition: &Definition) -> Option<&String> {
        self.owner_by_member_position.get(&(
            definition.position.start_line,
            definition.position.start_column,
        ))
    }
}

/// Where the file states a type, which is the only way to know what a receiver is.
struct Annotations {
    types_by_binding: HashMap<String, Vec<String>>,
    class_spans: Vec<NamedSpan>,
}

impl Annotations {
    fn read(source_code: &str, root_node: Node) -> Result<Self, String> {
        Ok(Self {
            types_by_binding: query::map_pairs(
                RECEIVER_TYPES,
                source_code,
                root_node,
                "binding",
                "annotated",
                |binding, annotation| {
                    Some((
                        binding.utf8_text(source_code.as_bytes()).ok()?.to_string(),
                        type_names(&annotation, source_code),
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
        })
    }

    /// The types this receiver expression may have. A union states several, and the member may be
    /// declared by any of them.
    fn types_of(&self, receiver: Node, source_code: &str) -> Option<Vec<String>> {
        match receiver.kind() {
            "identifier" => self
                .types_by_binding
                .get(receiver.utf8_text(source_code.as_bytes()).ok()?)
                .cloned(),
            "this" => self
                .enclosing_class(receiver.start_position().row + 1)
                .map(|class_name| vec![class_name]),
            // `a as First` states the type at the access itself, which is as good as an annotation
            // on the binding and better than nothing when the binding has none.
            "as_expression" | "satisfies_expression" => {
                Some(type_names(&receiver.named_child(1)?, source_code))
            }
            // `(a)`, `a!` and `-a` are the same receiver wearing a wrapper.
            "parenthesized_expression" | "non_null_expression" | "unary_expression" => {
                self.types_of(receiver.named_child(0)?, source_code)
            }
            // A chained `a.b.c` or a call's result is an expression whose type the file does not
            // state, so nothing is claimed for it.
            _ => None,
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
}

/// The type names a type expression states.
///
/// Type arguments are not descended into: `Wrapper<Reader>` states `Wrapper`, and its members are
/// the ones a receiver of that type reaches.
fn type_names(node: &Node, source_code: &str) -> Vec<String> {
    if node.kind() == "type_identifier" {
        return node
            .utf8_text(source_code.as_bytes())
            .map(|name| vec![name.to_string()])
            .unwrap_or_default();
    }

    (0..node.child_count())
        .filter_map(|index| node.child(index))
        .filter(|child| child.kind() != "type_arguments")
        .flat_map(|child| type_names(&child, source_code))
        .collect()
}
