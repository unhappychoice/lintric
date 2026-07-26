// Every pattern form that binds a name.
//
// Each was checked by hand while probing and was already correct; the fixture is what keeps it that
// way. What a pattern reads rather than binds is covered by `patterns.rs` and `match_arms.rs`.

struct Point {
    x: i32,
    y: i32,
}

const LOW: i32 = 1;
const HIGH: i32 = 9;

fn slices(xs: &[i32]) -> i32 {
    match xs { //~ depends: xs@14
        [first, .., last] => first + last,
        [only] => *only,
        [] => 0,
    }
}

fn references(p: &Point) -> i32 { //~ depends: Point@6
    let Point { ref x, .. } = *p; //~ depends: Point@6, x@7, p@22
    *x //~ depends: x@23
}

fn mutable(mut p: Point) -> i32 { //~ depends: Point@6
    p.x += 1; //~ depends: x@7, p@27
    p.x //~ depends: x@7, p@27
}

fn ranges(v: i32) -> i32 {
    match v { //~ depends: v@32
        LOW..=HIGH => 0, //~ depends: LOW@11, HIGH@12
        bound @ 10..=20 => bound,
        1 | 2 => 3,
        _ => -1,
    }
}

fn tuples(pair: (i32, Point)) -> i32 { //~ depends: Point@6
    let (count, Point { y, .. }) = pair; //~ depends: Point@6, y@8, pair@41
    count + y //~ depends: count@42, y@42
}
