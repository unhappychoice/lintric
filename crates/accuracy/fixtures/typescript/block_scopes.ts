// Bindings declared inside a block.
//
// `const` and `let` are block-scoped, so the outer `a` is what the line below the function reads.

const a = 1;

function inner(): number {
    {
        const a = 2;
        return a; //~ depends: a@9
    }
}

const outside = a; //~ depends: a@5
const result = inner(); //~ depends: inner@7

// A binding is not among the candidates for its own initializer, so `let x = x + 1` reads the
// previous `x` even when that one is in an enclosing scope.
function shadowing(): number {
    let v = 1;
    {
        let v = v + 1; //~ depends: v@20
        return v; //~ depends: v@22
    }
}

// A `for` header, a `switch` case and a `while` body each scope what they declare, so a name reused
// inside one of them is invisible outside it — and a sibling case cannot see the other's.
const held = 1;

function loops(): number {
    let sum = 0;

    for (let held = 0; held < 3; held++) {
        sum += held; //~ depends: sum@32, held@34
    }

    while (sum < 9) { //~ depends: sum@32
        const held = 2;
        sum += held; //~ depends: sum@32, held@39
    }

    return sum + held; //~ depends: sum@32, held@29
}

function branches(k: number): number {
    switch (k) { //~ depends: k@46
        case 1: {
            const held = 10;
            return held; //~ depends: held@49
        }
        default:
            return held; //~ depends: held@29
    }
}
