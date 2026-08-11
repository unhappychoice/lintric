// Trait objects behind references and boxes.
//
// `&dyn Shape` states the trait and nothing else, so `shape.area()` reaches the trait's declaration
// rather than any implementor's — pointing at Square's would claim a relationship with a type this
// line never names, and would be a guess as soon as a second implementor existed.

trait Shape {
    fn area(&self) -> i32;
}

struct Square;

impl Shape for Square { //~ depends: Shape@7, Square@11
    fn area(&self) -> i32 { //~ depends: area@8
        1
    }
}

fn measure(shape: &dyn Shape) -> i32 { //~ depends: Shape@7
    shape.area() //~ depends: shape@19, area@8
}

fn boxed() -> Box<dyn Shape> { //~ depends: Shape@7
    Box::new(Square) //~ depends: Square@11
}

fn main() {
    let s = Square; //~ depends: Square@11
    let _ = measure(&s); //~ depends: measure@19, s@28
    let _ = boxed(); //~ depends: boxed@23
}
