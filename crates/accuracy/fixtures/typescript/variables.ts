// Const bindings and arithmetic.

const a = 1;
const b = a + 2; //~ depends: a@3
const c = a + b; //~ depends: a@3, b@4

function main(): number {
    const d = c; //~ depends: c@5
    return d + 1; //~ depends: d@8
}
