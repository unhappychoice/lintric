//! The TypeScript node names a receiver can wear.
//!
//! Everything about narrowing a member access to the member its receiver's type declares is shared;
//! see `crate::dependency_resolver::receiver_narrowing`.

use crate::dependency_resolver::receiver_narrowing::{Dialect, Queries};

pub const DIALECT: Dialect = Dialect {
    queries: Queries {
        member_owners: include_str!("../../../../queries/typescript/member_owners.scm"),
        accesses: include_str!("../../../../queries/typescript/member_accesses.scm"),
        receiver_types: include_str!("../../../../queries/typescript/receiver_types.scm"),
        enclosing_bodies: include_str!("../../../../queries/typescript/enclosing_classes.scm"),
    },
    // `(a)`, `a!` and `-a` are the same receiver wearing a wrapper.
    wrappers: &[
        "parenthesized_expression",
        "non_null_expression",
        "unary_expression",
    ],
    enclosing: &["this"],
    stated: &["as_expression", "satisfies_expression"],
};
