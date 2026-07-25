// Trait objects behind references and boxes.

trait Shape {
    fn area(&self) -> i32;
}

struct Square;

impl Shape for Square { //~ depends: Shape@3, Square@7
    fn area(&self) -> i32 { //~ depends: area@4
        1
    }
}

fn measure(shape: &dyn Shape) -> i32 { //~ depends: Shape@3
    shape.area() //~ depends: shape@15, area@10
}

fn boxed() -> Box<dyn Shape> { //~ depends: Shape@3
    Box::new(Square) //~ depends: Square@7
}

fn main() {
    let s = Square; //~ depends: Square@7
    let _ = measure(&s); //~ depends: measure@15, s@24
    let _ = boxed(); //~ depends: boxed@19
}
