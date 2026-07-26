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

// A default value and a computed key are read rather than declared, so each references what it
// names instead of shadowing it.
const fallback = 9;
const key = "x";

const { x = fallback } = origin; //~ depends: fallback@25, origin@11
const { [key]: byKey } = origin; //~ depends: key@26, origin@11

function withDefault({ y = fallback }: Point): number { //~ depends: Point@6, fallback@25
    return y; //~ depends: y@31
}

const used = x + byKey + withDefault(origin); //~ depends: x@28, byKey@29, withDefault@31, origin@11
