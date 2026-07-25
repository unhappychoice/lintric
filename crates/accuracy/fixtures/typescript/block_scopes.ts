// Bindings declared inside a block.
//
// `const` and `let` are block-scoped, so the outer `a` is what line 12 reads. It currently resolves
// into the block inside the function instead, recorded here as a known spurious edge. See #223.

const a = 1;

function inner(): number {
    {
        const a = 2;
        return a; //~ depends: a@10
    }
}

const outside = a; //~ depends: a@6
const result = inner(); //~ depends: inner@8
