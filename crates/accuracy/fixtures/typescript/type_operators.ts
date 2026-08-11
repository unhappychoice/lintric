// Type-level operators.
//
// Each of these was checked by hand while probing and was already correct; the fixture is what keeps
// it that way.

interface Shape {
    kind: string;
    size: number;
}

type Keys = keyof Shape; //~ depends: Shape@6

type Sized = Shape["size"]; //~ depends: Shape@6

type Boxed<T = Shape> = T | null; //~ depends: Shape@6

type Renamed = { [K in Keys]: Shape[K] }; //~ depends: Keys@11, Shape@6

type Narrowed<T> = T extends Shape ? number : string; //~ depends: Shape@6

const sample: Shape = { kind: "a", size: 1 }; //~ depends: Shape@6

type SameAsSample = typeof sample; //~ depends: sample@21

type Greeting = `hello ${string}`;

interface Callable {
    (input: Shape): number; //~ depends: Shape@6
    new (seed: number): Callable; //~ depends: Callable@27
    readonly tags: readonly string[];
}

type Labelled = [first: Shape, second: number]; //~ depends: Shape@6

function invoke(c: Callable, s: Shape): number { //~ depends: Callable@27, Shape@6
    return c(s); //~ depends: c@35, s@35
}
