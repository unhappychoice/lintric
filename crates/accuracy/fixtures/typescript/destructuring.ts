// Object and array destructuring, and rest patterns.
//
// A pattern's field names do not reference declared members. The shape's coupling to a type is
// recorded through the annotation naming it; see "Object shapes" in the crate README.

interface Point {
    x: number;
    y: number;
}

const origin: Point = { x: 0, y: 0 }; //~ depends: Point@6

function shift({ x, y }: Point): number { //~ depends: Point@6
    return x + y; //~ depends: x@13, y@13
}

const [first, ...rest] = [1, 2, 3];
const total = first + rest.length; //~ depends: first@17, rest@17

const { x: renamed } = origin; //~ depends: origin@11
const _ = shift(origin) + total + renamed; //~ depends: shift@13, origin@11, total@18, renamed@20
