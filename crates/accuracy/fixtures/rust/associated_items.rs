// Associated functions, associated constants and the Self type.

struct Counter {
    count: i32,
}

impl Counter { //~ depends: Counter@3
    const START: i32 = 0;

    fn new() -> Self { //~ depends: Counter@3
        Self { count: Self::START } //~ depends: Counter@3, count@4, START@8
    }

    fn bump(&mut self) {
        self.count += 1; //~ depends: count@4
    }
}

fn main() {
    let mut c = Counter::new(); //~ depends: Counter@3, new@10
    c.bump(); //~ depends: bump@14, c@20
}
