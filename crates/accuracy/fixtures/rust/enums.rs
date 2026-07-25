// Enum definitions, variants and match arms.

enum Direction {
    Left,
    Right,
}

fn flip(direction: Direction) -> Direction { //~ depends: Direction@3
    match direction { //~ depends: direction@8
        Direction::Left => Direction::Right, //~ depends: Direction@3, Left@4, Right@5
        Direction::Right => Direction::Left, //~ depends: Direction@3, Right@5, Left@4
    }
}

fn main() {
    let start = Direction::Left; //~ depends: Direction@3, Left@4
    let end = flip(start); //~ depends: flip@8, start@16
    let _ = end; //~ depends: end@17
}
