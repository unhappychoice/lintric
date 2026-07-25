// Operator traits and indexing.

struct Meters {
    value: i32,
}

impl std::ops::Add for Meters { //~ depends: Meters@3
    type Output = Meters; //~ depends: Meters@3

    fn add(self, other: Meters) -> Meters { //~ depends: Meters@3
        Meters { value: self.value + other.value } //~ depends: Meters@3, value@4, other@10
    }
}

fn main() {
    let a = Meters { value: 1 }; //~ depends: Meters@3, value@4
    let b = Meters { value: 2 }; //~ depends: Meters@3, value@4
    let sum = a + b; //~ depends: a@16, b@17
    let list = [sum.value, 0]; //~ depends: sum@18, value@4
    let _ = list[0]; //~ depends: list@19
}
