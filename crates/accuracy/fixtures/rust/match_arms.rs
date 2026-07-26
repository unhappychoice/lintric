// Each match arm scopes the names its pattern binds.
//
// Two arms are alternative branches, so one arm's binding is invisible in another — a name reused
// across arms is a new binding each time, not a reference to the earlier one.
//
// A guard is the pattern's `condition:` field, so it sits among the pattern's own children while
// reading the names the pattern binds rather than binding any itself.
//
// A bare identifier in a pattern is a binding only when it names nothing: `Idle` and `LIMIT` below
// name a variant and a constant, so they are references.
//
// The last arm binds and reads on one line, which is a dependency the analyzer deliberately drops,
// so it carries no annotation.

enum Status {
    Idle,
    Busy,
}

use Status::Idle; //~ depends: Status@15, Idle@16

const LIMIT: i32 = 5;

fn threshold() -> i32 {
    LIMIT //~ depends: LIMIT@22
}

fn describe(s: Status) -> i32 { //~ depends: Status@15
    match s { //~ depends: s@28
        Idle => 0, //~ depends: Idle@20
        Status::Busy => 1, //~ depends: Status@15, Busy@17
    }
}

fn classify(v: i32) -> i32 {
    match v { //~ depends: v@35
        LIMIT => 0, //~ depends: LIMIT@22
        n
            if n > threshold() => //~ depends: n@38, threshold@24
        {
            n //~ depends: n@38
        }
        n => -n,
    }
}
