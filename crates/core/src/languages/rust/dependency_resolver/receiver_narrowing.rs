//! The Rust node names a receiver can wear.
//!
//! Everything about narrowing a method call to the method its receiver's type declares is shared;
//! see `crate::dependency_resolver::receiver_narrowing`.

use crate::dependency_resolver::receiver_narrowing::{Dialect, Queries};

pub const DIALECT: Dialect = Dialect {
    queries: Queries {
        member_owners: include_str!("../../../../queries/rust/method_owners.scm"),
        accesses: include_str!("../../../../queries/rust/method_calls.scm"),
        receiver_types: include_str!("../../../../queries/rust/receiver_types.scm"),
        enclosing_bodies: include_str!("../../../../queries/rust/enclosing_impls.scm"),
    },
    // `(a)`, `&a` and `*a` are the same receiver wearing a wrapper, and `a?` is the value it
    // unwraps to.
    wrappers: &[
        "parenthesized_expression",
        "reference_expression",
        "unary_expression",
        "try_expression",
    ],
    // `self` names the type whose impl the call sits inside.
    enclosing: &["self"],
    // Rust states a receiver's type at a binding rather than at the access; `a as T` casts values,
    // not references, so there is nothing to read here.
    stated: &[],
};
