import { SomeClass } from './some_module';
import * as Utils from './utils';
import DefaultExport from './default_module';

interface Point {
    x: number;
    y: number;
}

function add(a: number, b: number): number {
    const result = a + b;
    return result;
}

function main() {
    const p1: Point = { x: 1, y: 2 };
    const p2: Point = { x: 3, y: 4 };

    const p3: Point = {
        x: add(p1.x, p2.x),
        y: add(p1.y, p2.y),
    };

    const p4 = (() => {
        const p5: Point = { x: 5, y: 6 };
        const p6 = p5;
        return p6;
    })();

    let x = 1;
    let y = x + 1;
    let z = y + x;

    console.log(p3);
    console.log(p4);
}

const instance = new SomeClass();
const result = Utils.helperFunction();
DefaultExport.init();

function anotherFunction() {
    console.log(instance, result);
}
