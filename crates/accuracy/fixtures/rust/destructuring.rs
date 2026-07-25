// Destructuring in let bindings and match arms.

struct Point {
    x: i32,
    y: i32,
}

fn main() {
    let pair = (1, 2);
    let (first, second) = pair; //~ depends: pair@9
    let sum = first + second; //~ depends: first@10, second@10

    let p = Point { x: 3, y: 4 }; //~ depends: Point@3, x@4, y@5
    let Point { x, y } = p; //~ depends: Point@3, x@4, y@5, p@13
    let area = x * y; //~ depends: x@14, y@14

    let _ = (sum, area); //~ depends: sum@11, area@15
}
