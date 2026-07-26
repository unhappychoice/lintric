// Enum declarations and member references.

enum Direction {
    Left,
    Right,
}

function flip(direction: Direction): Direction { //~ depends: Direction@3
    return direction === Direction.Left ? Direction.Right : Direction.Left; //~ depends: direction@8, Direction@3, Left@4, Right@5
}

const start = Direction.Left; //~ depends: Direction@3, Left@4
const end = flip(start); //~ depends: flip@8, start@12

// A member with an initializer hangs off an `enum_assignment` rather than the body, and one
// initializer may name a sibling member.
enum Level {
    Low = 1,
    High = 2,
    Same = Low, //~ depends: Low@18
}

const chosen = Level.High; //~ depends: Level@17, High@19
const echoed = Level.Same; //~ depends: Level@17, Same@20
