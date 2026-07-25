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
