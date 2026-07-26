// Associated constants reached through a type, a parameter and `Self`.
//
// Each of these was checked by hand while probing and was already correct; the fixture is what keeps
// it that way.

trait Limit {
    const CAP: i32;

    fn combined<T: Limit>(other: T) -> i32 { //~ depends: Limit@6
        T::CAP + Self::CAP //~ depends: T@9, CAP@7, Limit@6
    }
}

struct Small;

impl Limit for Small { //~ depends: Limit@6, Small@14
    const CAP: i32 = 1; //~ depends: CAP@7
}

fn fully_qualified() -> i32 {
    <Small as Limit>::CAP //~ depends: Small@14, Limit@6, CAP@17
}

fn plain_path() -> i32 {
    Small::CAP //~ depends: Small@14, CAP@17
}
