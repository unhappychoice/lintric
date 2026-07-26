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
