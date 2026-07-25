// Declarations inherited from a supertrait.

trait Base {
    fn run(&self) -> i32;
}

trait Extended: Base { //~ depends: Base@3
    fn extra(&self) -> i32;
}

struct S;

impl Extended for S { //~ depends: Extended@7, S@11
    fn run(&self) -> i32 { //~ depends: run@4
        1
    }

    fn extra(&self) -> i32 { //~ depends: extra@8
        2
    }
}
