mod my_module {
    pub struct MyStruct;
    pub fn my_function() {}
    pub const MY_CONST: i32 = 1;
}

use my_module::MyStruct;
use my_module::{my_function, MY_CONST};
use my_module::*;
use my_module as mm;

fn main() {
    let s = MyStruct;
    my_function();
    let c = MY_CONST;
    let s2 = mm::MyStruct;
}