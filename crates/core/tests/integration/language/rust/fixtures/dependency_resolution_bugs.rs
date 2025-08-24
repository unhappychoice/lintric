use std::collections::HashMap;
use std::vec::Vec;

struct TestStruct {
    field: HashMap<String, i32>,
}

impl TestStruct {
    pub fn new(data: HashMap<String, i32>) -> Self {
        let vec_data = Vec::new();
        TestStruct {
            field: data,
        }
    }
    
    pub fn process(&self, input: String) -> Option<i32> {
        self.field.get(&input).copied()
    }
}

fn main() {
    let mut map = HashMap::new();
    map.insert("test".to_string(), 42);
    let mut vec_data = Vec::new();
    vec_data.push(1);
    let test = TestStruct::new(map);
    let result = test.process("key".to_string());
}