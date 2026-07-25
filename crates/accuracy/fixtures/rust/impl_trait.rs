// `impl Trait` in argument and return position.

trait Shape {
    fn area(&self) -> i32;
}

struct Square;

impl Shape for Square { //~ depends: Shape@3, Square@7
    fn area(&self) -> i32 { //~ depends: area@4
        1
    }
}

fn make() -> impl Shape { //~ depends: Shape@3
    Square //~ depends: Square@7
}

fn measure(shape: impl Shape) -> i32 { //~ depends: Shape@3
    shape.area() //~ depends: shape@19, area@10
}

fn main() {
    let _ = measure(make()); //~ depends: measure@19, make@15
}
