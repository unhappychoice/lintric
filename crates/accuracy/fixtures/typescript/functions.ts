// Function declarations, parameters and nested calls.

function double(n: number): number {
    return n * 2; //~ depends: n@3
}

function quadruple(n: number): number {
    return double(double(n)); //~ depends: double@3, n@7
}

const x = 3;
const y = quadruple(x); //~ depends: quadruple@7, x@11
const z = double(y); //~ depends: double@3, y@12
