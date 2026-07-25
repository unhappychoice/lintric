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
