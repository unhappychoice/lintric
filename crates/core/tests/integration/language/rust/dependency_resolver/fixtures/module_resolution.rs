use std::collections::HashMap;

mod utils {
    pub fn helper() -> i32 {
        42
    }
    
    pub mod inner {
        pub fn deep_function() -> i32 {
            42
        }
    }
}

fn main() {
    let mut map = HashMap::new();
    map.insert("key", "value");
    
    let result = utils::helper();
    let deep_result = utils::inner::deep_function();
    
    println!("{} {}", result, deep_result);
}