// Struct definitions, field access and struct literals.

struct Point {
    x: i32,
    y: i32,
}

impl Point { //~ depends: Point@3
    fn norm(&self) -> i32 {
        self.x * self.x + self.y * self.y //~ depends: x@4, y@5
    }
}

fn main() {
    let a = 1;
    let b = 2;
    let p = Point { x: a, y: b }; //~ depends: Point@3, x@4, y@5, a@15, b@16
    let n = p.norm(); //~ depends: norm@9, p@17
    let _ = n; //~ depends: n@18
}
