struct Point {
    x: i32,
    y: i32,
}

fn add(a: i32, b: i32) -> i32 {
    let result = a + b;
    result
}

fn main() {
    let p1 = Point { x: 1, y: 2 };
    let p2 = Point { x: 3, y: 4 };

    let p3 = Point {
        x: add(p1.x, p2.x),
        y: add(p1.y, p2.y),
    };

    let p4 = {
        let p5 = Point { x: 5, y: 6 };
        let p6 = p5;
        p6
    };

    let x = 1;
    let y = x + 1;
    let z = y + x;

    println!("{:?}", p3);
    println!("{:?}", p4);
}
