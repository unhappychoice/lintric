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

// An abstract class satisfies a contract while declaring one of its own, so its members reference
// what the interface declared as well as being referenced by the class below.
interface Tagged {
    tag: string;

    render(): string;
}

abstract class Partial implements Tagged { //~ depends: Tagged@24
    abstract tag: string; //~ depends: tag@25

    render(): string { //~ depends: render@27
        return this.tag; //~ depends: tag@31
    }
}

class Concrete extends Partial { //~ depends: Partial@30
    tag = "concrete"; //~ depends: tag@31
}

const rendered = new Concrete().render(); //~ depends: Concrete@38, render@33
