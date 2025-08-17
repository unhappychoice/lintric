struct MyStruct {
    value: i32,
}

impl MyStruct {
    fn my_method(&self) -> i32 {
        self.value
    }
}

fn main() {
    let s = MyStruct { value: 10 };
    s.my_method();
}
