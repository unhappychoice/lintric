// `impl Trait` in argument and return position.
//
// `shape: impl Shape` states the trait and nothing else, so `shape.area()` reaches the trait's
// declaration rather than any implementor's — pointing at Square's would claim a relationship with a
// type this line never names, and would be a guess as soon as a second implementor existed.

trait Shape {
    fn area(&self) -> i32;
}

struct Square;

impl Shape for Square { //~ depends: Shape@7, Square@11
    fn area(&self) -> i32 { //~ depends: area@8
        1
    }
}

fn make() -> impl Shape { //~ depends: Shape@7
    Square //~ depends: Square@11
}

fn measure(shape: impl Shape) -> i32 { //~ depends: Shape@7
    shape.area() //~ depends: shape@23, area@8
}

fn main() {
    let _ = measure(make()); //~ depends: measure@23, make@19
}
