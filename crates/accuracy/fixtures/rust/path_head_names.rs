// One name declared as both a module and a function.
//
// A path head names a module or a type, never a function or a local, so `Setting::LEVEL` reaches the
// module while `Setting()` reaches the function. A later segment can be anything, which is why the
// distinction is about being reached through rather than about being in a path. See #259.

mod Setting {
    pub const LEVEL: i32 = 1;
}

fn Setting() -> i32 {
    2
}

struct Holder;

impl Holder { //~ depends: Holder@15
    fn make() -> Holder { //~ depends: Holder@15
        Holder //~ depends: Holder@15
    }
}

fn read() -> i32 {
    let held = Holder::make(); //~ depends: Holder@15, make@18
    let _ = held; //~ depends: held@24
    Setting() + Setting::LEVEL //~ depends: Setting@11, Setting@7, LEVEL@8
}
