namespace Utils {
    export function helper(): number {
        return 42;
    }
    
    export class Calculator {
        add(a: number, b: number): number {
            return a + b;
        }
    }
    
    export namespace Inner {
        export function deepFunction(): number {
            return helper() + 1;
        }
    }
}

function main() {
    const result = Utils.helper();
    const calc = new Utils.Calculator();
    const sum = calc.add(1, 2);
    const deep = Utils.Inner.deepFunction();
    
    console.log(result, sum, deep);
}