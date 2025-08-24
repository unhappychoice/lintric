class Calculator {
    private value: number = 0;
    
    add(n: number): void {
        this.value += n;
    }
    
    getValue(): number {
        return this.value;
    }
}

interface Drawable {
    draw(): void;
}

class Circle implements Drawable {
    radius: number;
    
    constructor(radius: number) {
        this.radius = radius;
    }
    
    draw(): void {
        console.log(`Drawing circle with radius ${this.radius}`);
    }
}

function main() {
    const calc = new Calculator();
    calc.add(5);
    console.log(calc.getValue());
    
    const circle = new Circle(10);
    circle.draw();
}