class MyClass {
    constructor(public value: number){}
    greet() { console.log(this.value); }
}
let instance = new MyClass(10);
instance.greet();