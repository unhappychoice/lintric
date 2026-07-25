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
