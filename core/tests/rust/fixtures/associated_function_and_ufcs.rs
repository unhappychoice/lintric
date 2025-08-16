struct MyStruct;

impl MyStruct {
    fn new() -> MyStruct {
        MyStruct
    }
}

trait MyTrait {
    fn my_function();
}

struct MyType;

impl MyTrait for MyType {
    fn my_function() {
        println!("Hello from MyType!");
    }
}

fn main() {
    let _s = MyStruct::new();
    MyType::my_function();
    <MyType as MyTrait>::my_function();
}
