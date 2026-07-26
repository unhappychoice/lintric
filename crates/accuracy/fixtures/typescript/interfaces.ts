// Interfaces, type aliases and implementations.

interface Shape {
    area(): number;
}

type Size = number;

class Square implements Shape { //~ depends: Shape@3
    side: Size; //~ depends: Size@7

    constructor(side: Size) { //~ depends: Size@7
        this.side = side; //~ depends: side@10, side@12
    }

    area(): number { //~ depends: area@4
        return this.side * this.side; //~ depends: side@10
    }
}

const s = new Square(2); //~ depends: Square@9
const a = s.area(); //~ depends: area@16, s@21

// An interface declares properties alongside methods, and a class field satisfying one is coupled
// to it the same way a method is.
interface Named {
    label: string;

    describe(): string;
}

class Tag implements Named { //~ depends: Named@26
    label = "tag"; //~ depends: label@27

    describe(): string { //~ depends: describe@29
        return this.label; //~ depends: label@33
    }
}
