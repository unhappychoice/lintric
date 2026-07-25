// Declaration forms whose own name is not a use of itself.
//
// Each name below collides deliberately with a top-level declaration of the same name. A name that
// declares must not resolve to the colliding one; see #233.

function inner(): number {
    return 0;
}

class Inner {
    tag = "outer";
}

function* stream(): Iterable<number> {
    yield 1;
}

const asExpression = function inner(): number {
    return 1;
};

const classExpression = class Inner {
    tag = "inner";
};

function overloaded(x: string): string;
function overloaded(x: string): string {
    return x; //~ depends: x@27
}

const consumed = [inner(), new Inner(), stream(), asExpression, classExpression, overloaded("a")]; //~ depends: inner@6, Inner@10, stream@14, asExpression@18, classExpression@22, overloaded@27
