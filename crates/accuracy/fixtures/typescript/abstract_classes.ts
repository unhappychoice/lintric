// Abstract classes, abstract members and their implementations.

abstract class Shape {
    abstract area(): number;

    describe(): string {
        return String(this.area()); //~ depends: area@4
    }
}

class Square extends Shape { //~ depends: Shape@3
    side = 1;

    area(): number { //~ depends: area@4
        return this.side * this.side; //~ depends: side@12
    }
}

const s = new Square(); //~ depends: Square@11
const text = s.describe(); //~ depends: describe@6, s@19
