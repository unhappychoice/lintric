// Generic traits, generic supertraits and their implementations.
//
// `T` on line 10 is Extended's own parameter, so it produces no edge. It currently resolves to
// Base's instead, which this fixture records as a known spurious edge. See #215.

trait Base<T> {
    fn run(&self, value: T) -> i32; //~ depends: T@6
}

trait Extended<T>: Base<T> { //~ depends: Base@6
    fn extra(&self) -> i32;
}

struct S;

impl Extended<i32> for S { //~ depends: Extended@10, S@14
    fn run(&self, value: i32) -> i32 { //~ depends: run@7
        value //~ depends: value@17
    }

    fn extra(&self) -> i32 { //~ depends: extra@11
        1
    }
}

fn main() {
    let s = S; //~ depends: S@14
    let _ = s.extra(); //~ depends: extra@21, s@27
}
