function main() {
    // Function hoisting - should create dependency even though helper is defined later
    const result = helper();
    
    // Variable forward reference - should NOT create dependency (let/const are not hoisted)
    const y = x + 1;
    const x = 42;
}

// Helper function defined after main - but should still be accessible due to hoisting
function helper(): number {
    return 42;
}

// Test interface hoisting
function useInterface() {
    const obj: MyInterface = { field: 10 };
}

interface MyInterface {
    field: number;
}

// Test class hoisting
function useClass() {
    const instance = new MyClass();
}

class MyClass {
    constructor() {}
}

// Test type hoisting
function useType(): MyType {
    return 42;
}

type MyType = number;

// Test enum hoisting
function useEnum() {
    const value = MyEnum.Value1;
}

enum MyEnum {
    Value1,
    Value2
}