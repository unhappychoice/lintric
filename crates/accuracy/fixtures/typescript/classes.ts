// Class fields, constructor parameters and methods.

class Point {
    x: number;
    y: number;

    constructor(x: number, y: number) {
        this.x = x; //~ depends: x@4, x@7
        this.y = y; //~ depends: y@5, y@7
    }

    norm(): number {
        return this.x * this.x + this.y * this.y; //~ depends: x@4, y@5
    }
}

const p = new Point(1, 2); //~ depends: Point@3
const n = p.norm(); //~ depends: norm@12, p@17
