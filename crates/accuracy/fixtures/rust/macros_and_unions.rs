// A macro definition and a union.
//
// Each was checked by hand while probing and was already correct; the fixture is what keeps it that
// way. `macros.rs` covers invoking a macro and the captures inside a format string.

const BASE: i32 = 3;

macro_rules! twice {
    ($x:expr) => {
        $x * 2 //~ depends: $x@9
    };
}

fn expanded() -> i32 {
    twice!(BASE) //~ depends: twice@8, BASE@6
}

union Raw {
    bits: u32,
}

fn read_union(r: Raw) -> u32 { //~ depends: Raw@18
    unsafe { r.bits } //~ depends: bits@19, r@22
}

struct Holder {
    value: i32,
}

async fn fetch() -> Holder { //~ depends: Holder@26
    Holder { value: 1 } //~ depends: Holder@26, value@27
}

async fn awaited() -> i32 {
    let held = fetch().await; //~ depends: fetch@30
    held.value //~ depends: value@27, held@35
}
