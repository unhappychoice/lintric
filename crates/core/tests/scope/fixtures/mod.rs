pub const BASIC_SCOPE_TEST_RUST: &str = include_str!("basic_scope_test.rs");

pub const COMPLEX_TYPESCRIPT_SCOPE: &str = r#"
class MyClass {
    private privateField: number = 0;
    public publicField: string = "hello";
    
    constructor(param: number) {
        this.privateField = param;
    }
    
    public method(arg: string): void {
        const localVar = this.privateField;
        
        function innerFunction() {
            const innerVar = arg; // Captures outer parameter
            console.log(innerVar);
        }
        
        if (localVar > 0) {
            const blockVar = "block scope";
            innerFunction();
        }
    }
}

interface MyInterface {
    interfaceMethod(param: boolean): string;
}

function globalFunction() {
    const funcVar = new MyClass(42);
    return funcVar.publicField;
}
"#;

pub const RUST_IMPL_TRAIT_SCOPE: &str = r#"
struct MyStruct {
    field: i32,
}

impl MyStruct {
    fn new(value: i32) -> Self {
        Self { field: value }
    }
    
    fn get_field(&self) -> i32 {
        self.field
    }
}

trait MyTrait {
    fn trait_method(&self) -> String;
}

impl MyTrait for MyStruct {
    fn trait_method(&self) -> String {
        format!("Value: {}", self.get_field())
    }
}
"#;
