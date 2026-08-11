// Import aliases, nested groups and glob imports.

mod shapes {
    pub struct Square {
        pub side: i32,
    }

    pub struct Circle {
        pub radius: i32,
    }

    pub fn unit() -> i32 {
        1
    }
}

use shapes::Square as Boxy; //~ depends: shapes@3, Square@4

use shapes::{Circle, unit}; //~ depends: shapes@3, Circle@8, unit@12

fn main() {
    let b = Boxy { side: 1 }; //~ depends: Boxy@17, side@5
    let c = Circle { radius: 2 }; //~ depends: Circle@19, radius@9
    let _ = b.side + c.radius + unit(); //~ depends: b@22, side@5, c@23, radius@9, unit@19
}
