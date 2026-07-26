//! Pointing `receiver.member` at the member the receiver's type declares.
//!
//! Two types may declare a member of the same name, and matching on the name alone links an access
//! to every one of them — inventing a relationship with a type the code never mentions. The
//! receiver's type is what tells them apart, and a single file knows it only where it is written
//! down. Where it is not written down, nothing is claimed.
//!
//! Both languages ask the same three questions — which type declares this member, what does this
//! access read from, and what is that receiver's type — so only the node names differ, and those
//! arrive as a `Dialect`.

use crate::models::{Definition, Usage};
use crate::query::{self, NamedSpan};
use std::collections::{HashMap, HashSet};
use tree_sitter::Node;

/// Queries locating the parts of a member access in one language.
pub struct Queries {
    /// Which type declares a member, as `@owner` and `@member`.
    pub member_owners: &'static str,
    /// What each access reads from, as `@receiver` and the `@accessed` member.
    pub accesses: &'static str,
    /// Bindings whose type the file states, as `@binding` and `@annotated`.
    pub receiver_types: &'static str,
    /// Where each type's body spans, as `@owner` and `@body`.
    pub enclosing_bodies: &'static str,
}

/// What the node names of one language mean for a receiver.
pub struct Dialect {
    pub queries: Queries,
    /// Kinds that wrap a receiver without changing what it is, such as a parenthesis or a borrow.
    pub wrappers: &'static [&'static str],
    /// Kinds naming the type the code sits inside: `this`, `self`.
    pub enclosing: &'static [&'static str],
    /// Kinds stating the receiver's type at the access, as their second named child: `a as First`.
    pub stated: &'static [&'static str],
}

/// What each member access can be narrowed to, read off the file once.
pub struct ReceiverNarrowing {
    owner_by_member_position: HashMap<(usize, usize), String>,
    /// Every member access, keyed by the span a `Usage` carries for it.
    ///
    /// The span rather than the start, because `a.to_b().shared()` nests two accesses that begin at
    /// the same token — Rust records a method call at the start of its receiver, so only the end
    /// tells the outer one from the inner.
    ///
    /// Held separately from the types below so that "an access whose receiver's type is unknown" is
    /// distinguishable from "not an access at all". The first must resolve to nothing; the second
    /// must be left entirely alone, since most usages are not member accesses.
    access_spans: HashSet<Span>,
    receiver_types_by_access: HashMap<Span, Vec<String>>,
}

/// Where an access begins and ends.
type Span = ((usize, usize), (usize, usize));

impl ReceiverNarrowing {
    pub fn new(dialect: &Dialect, source_code: &str, root_node: Node) -> Result<Self, String> {
        let annotations = Annotations::read(dialect, source_code, root_node)?;

        let accesses = query::map_pairs(
            dialect.queries.accesses,
            source_code,
            root_node,
            "receiver",
            "accessed",
            |receiver, accessed| {
                Some((
                    span(accessed),
                    annotations.types_of(dialect, receiver, source_code),
                ))
            },
        )?;

        Ok(Self {
            owner_by_member_position: query::text_by_position(
                dialect.queries.member_owners,
                source_code,
                root_node,
                "owner",
                "member",
            )?,
            access_spans: accesses.iter().map(|(span, _)| *span).collect(),
            receiver_types_by_access: accesses
                .into_iter()
                .filter_map(|(span, types)| types.map(|types| (span, types)))
                .collect(),
        })
    }

    /// Keep the candidates the receiver's type declares.
    ///
    /// A usage that is not a member access is left alone, since the receiver's type has no bearing
    /// on it. So is a single candidate: there is nothing to tell apart, and the receiver's type may
    /// well be unknowable. Several candidates with an unknown receiver type yield nothing, since
    /// every answer would be a guess and the wrong ones are edges to types the line never names.
    pub fn narrow<'a>(
        &self,
        usage: &Usage,
        candidates: Vec<&'a Definition>,
    ) -> Vec<&'a Definition> {
        let accessed = usage_span(usage);

        if candidates.len() <= 1 || !self.access_spans.contains(&accessed) {
            return candidates;
        }

        let Some(owners) = self.receiver_types_by_access.get(&accessed) else {
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
    body_spans: Vec<NamedSpan>,
}

impl Annotations {
    fn read(dialect: &Dialect, source_code: &str, root_node: Node) -> Result<Self, String> {
        Ok(Self {
            types_by_binding: merged(query::map_pairs(
                dialect.queries.receiver_types,
                source_code,
                root_node,
                "binding",
                "annotated",
                |binding, annotation| {
                    Some((
                        binding.utf8_text(source_code.as_bytes()).ok()?.to_string(),
                        stated_type_names(&annotation, source_code),
                    ))
                },
            )?),
            body_spans: query::text_by_span(
                dialect.queries.enclosing_bodies,
                source_code,
                root_node,
                "owner",
                "body",
            )?,
        })
    }

    /// The types this receiver expression may have. A union states several, and the member may be
    /// declared by any of them.
    fn types_of(
        &self,
        dialect: &Dialect,
        receiver: Node,
        source_code: &str,
    ) -> Option<Vec<String>> {
        let kind = receiver.kind();

        if kind == "identifier" {
            return self
                .types_by_binding
                .get(receiver.utf8_text(source_code.as_bytes()).ok()?)
                .cloned();
        }
        if dialect.enclosing.contains(&kind) {
            return self
                .enclosing_type(receiver.start_position().row + 1)
                .map(|owner| vec![owner]);
        }
        // A cast states the type at the access itself, which is as good as an annotation on the
        // binding and better than nothing when the binding has none.
        if dialect.stated.contains(&kind) {
            return Some(type_names(&receiver.named_child(1)?, source_code));
        }
        if dialect.wrappers.contains(&kind) {
            return self.types_of(dialect, receiver.named_child(0)?, source_code);
        }

        // A chained `a.b.c` or a call's result is an expression whose type the file does not state,
        // so nothing is claimed for it.
        None
    }

    /// The innermost body containing the line, so a nested type wins over the one it sits in.
    fn enclosing_type(&self, line: usize) -> Option<String> {
        self.body_spans
            .iter()
            .filter(|(_, (start, end))| *start <= line && line <= *end)
            .min_by_key(|(_, (start, end))| end - start)
            .map(|(owner, _)| owner.clone())
    }
}

/// Every type a binding is stated to have, rather than whichever the last match happened to state.
///
/// `let bound: Second = s;` states one through the annotation and one through the initializer, and
/// keeping only one of them was losing whichever the query matched first. Each is genuinely stated at
/// that line, and a name that is not a type matches no owner.
fn merged(pairs: Vec<(String, Vec<String>)>) -> HashMap<String, Vec<String>> {
    pairs
        .into_iter()
        .fold(HashMap::new(), |mut merged, (binding, types)| {
            merged.entry(binding).or_default().extend(types);
            merged
        })
}

fn span(node: Node) -> Span {
    (
        (
            node.start_position().row + 1,
            node.start_position().column + 1,
        ),
        (node.end_position().row + 1, node.end_position().column + 1),
    )
}

fn usage_span(usage: &Usage) -> Span {
    (
        (usage.position.start_line, usage.position.start_column),
        (usage.position.end_line, usage.position.end_column),
    )
}

/// The type names something stated as a type, reading a bare name as one.
///
/// `let n = Numbers;` states the type by naming a unit struct, which the grammar gives as an
/// `identifier` rather than a `type_identifier`. It cannot tell that from reading a variable, and
/// neither can this — a name that turns out to be a variable matches no type and rules nothing out.
fn stated_type_names(node: &Node, source_code: &str) -> Vec<String> {
    let names = type_names(node, source_code);

    match (names.is_empty(), node.kind()) {
        (true, "identifier") => node
            .utf8_text(source_code.as_bytes())
            .map(|name| vec![name.to_string()])
            .unwrap_or_default(),
        _ => names,
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
