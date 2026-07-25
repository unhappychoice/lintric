// Object and array destructuring, and rest patterns.

interface Point {
    x: number;
    y: number;
}

const origin: Point = { x: 0, y: 0 }; //~ depends: Point@3

function shift({ x, y }: Point): number { //~ depends: Point@3, x@4, y@5
    return x + y; //~ depends: x@10, y@10
}

const [first, ...rest] = [1, 2, 3];
const total = first + rest.length; //~ depends: first@14, rest@14

const { x: renamed } = origin; //~ depends: x@4, origin@8
const _ = shift(origin) + total + renamed; //~ depends: shift@10, origin@8, total@15, renamed@17
