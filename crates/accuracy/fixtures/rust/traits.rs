// Trait declarations and their implementations.

trait Shape {
    fn area(&self) -> i32;
    fn scaled(&self, factor: i32) -> i32;
}

struct Square {
    side: i32,
}

impl Shape for Square { //~ depends: Shape@3, Square@8
    fn area(&self) -> i32 { //~ depends: area@4
        self.side * self.side //~ depends: side@9
    }

    fn scaled(&self, factor: i32) -> i32 { //~ depends: scaled@5
        self.area() * factor //~ depends: area@13, factor@17
    }
}

fn main() {
    let s = Square { side: 4 }; //~ depends: Square@8, side@9
    let a = s.scaled(2); //~ depends: scaled@17, s@23
    let _ = a; //~ depends: a@24
}
