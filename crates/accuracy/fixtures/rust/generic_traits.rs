// Generic traits, generic supertraits and their implementations.
//
// `T` on lines 10 and 27 is the local parameter, so those references produce no edge. They
// currently resolve to Base's instead, recorded here as known spurious edges. See #215.

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

// A third `T`, on a generic function: both references are to the local parameter.
fn identity<T>(value: T) -> T {
    value //~ depends: value@27
}

fn main() {
    let s = S; //~ depends: S@14
    let _ = s.extra(); //~ depends: extra@21, s@32
    let _ = identity(1); //~ depends: identity@27
}
