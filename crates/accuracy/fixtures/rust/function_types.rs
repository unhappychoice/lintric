// Function types, higher-ranked bounds and an extern block.
//
// Each of these was checked by hand while probing and was already correct; the fixture is what keeps
// it that way.

struct Payload {
    size: i32,
}

type Handler = fn(Payload) -> i32; //~ depends: Payload@6

fn apply(h: Handler, p: Payload) -> i32 { //~ depends: Handler@10, Payload@6
    h(p) //~ depends: h@12, p@12
}

fn boxed(f: Box<dyn Fn(Payload) -> i32>, p: Payload) -> i32 { //~ depends: Payload@6
    f(p) //~ depends: f@16, p@16
}

fn ranked<F>(f: F) -> i32
where
    F: for<'a> Fn(&'a Payload) -> i32, //~ depends: F@20, Payload@6
{
    let made = Payload { size: 1 }; //~ depends: Payload@6, size@7
    f(&made) //~ depends: f@20, made@24
}

extern "C" {
    fn external(n: i32) -> i32;
}

fn calls_external() -> i32 {
    unsafe { external(1) } //~ depends: external@29
}
