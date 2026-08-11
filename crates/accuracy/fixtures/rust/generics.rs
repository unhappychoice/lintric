// Generic parameters and trait bounds.

trait Shape {
    fn area(&self) -> i32;
}

struct Square {
    side: i32,
}

impl Shape for Square { //~ depends: Shape@3, Square@7
    fn area(&self) -> i32 { //~ depends: area@4
        self.side //~ depends: side@8
    }
}

fn largest<T: Shape>(items: &[T]) -> i32 { //~ depends: Shape@3
    items.len() as i32 //~ depends: items@17
}

fn main() {
    let s = Square { side: 2 }; //~ depends: Square@7, side@8
    let _ = largest(&[s]); //~ depends: largest@17, s@22
}

struct Buffer<const N: usize> {
    data: [u8; N], //~ depends: N@26
}

impl<const N: usize> Buffer<N> { //~ depends: Buffer@26
    fn capacity(&self) -> usize {
        N //~ depends: N@30
    }
}
