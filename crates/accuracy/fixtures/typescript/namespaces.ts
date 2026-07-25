// Namespaces and qualified access into them.

namespace shapes {
    export interface Square {
        side: number;
    }

    export function unit(): number {
        return 1;
    }
}

const square: shapes.Square = { side: 2 }; //~ depends: shapes@3, Square@4

function area(s: shapes.Square): number { //~ depends: shapes@3, Square@4
    return s.side * s.side; //~ depends: s@15, side@5
}

const total = area(square) + shapes.unit(); //~ depends: area@15, square@13, shapes@3, unit@8
