fn main() {
    // Function hoisting - should create dependency even though helper is defined later
    let result = helper();
    
    // Variable forward reference - should NOT create dependency
    let y = x + 1;
    let x = 42;
}

// Helper function defined after main - but should still be accessible due to hoisting
fn helper() -> i32 {
    42
}

// Test struct hoisting
fn use_struct() {
    let instance = MyStruct { field: 10 };
}

struct MyStruct {
    field: i32,
}

// Test enum hoisting  
fn use_enum() {
    let value = MyEnum::Variant1;
}

enum MyEnum {
    Variant1,
    Variant2,
}

// Test type hoisting
fn use_type() -> MyType {
    42
}

type MyType = i32;