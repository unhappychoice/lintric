// Constants, statics and their references.

const LIMIT: i32 = 10;

static NAME: &str = "lintric";

struct Buffer {
    size: i32,
}

fn make() -> Buffer { //~ depends: Buffer@7
    Buffer { size: LIMIT } //~ depends: Buffer@7, size@8, LIMIT@3
}

fn main() {
    let b = make(); //~ depends: make@11
    let over = b.size > LIMIT; //~ depends: b@16, size@8, LIMIT@3
    let label = NAME; //~ depends: NAME@5
    let _ = (over, label); //~ depends: over@17, label@18
}
