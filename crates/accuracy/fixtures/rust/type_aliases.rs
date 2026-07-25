// Type aliases and their use in signatures.

struct Inner {
    v: i32,
}

type Alias = Inner; //~ depends: Inner@3

type Pair = (Alias, Alias); //~ depends: Alias@7

fn take(a: Alias) -> i32 { //~ depends: Alias@7
    a.v //~ depends: a@11, v@4
}

fn main() {
    let inner = Inner { v: 1 }; //~ depends: Inner@3, v@4
    let _ = take(inner); //~ depends: take@11, inner@16
}
