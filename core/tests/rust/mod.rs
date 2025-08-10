use insta::assert_snapshot;
use lintric_core::analyze_code;
use serde_json;

#[test]
fn test_analyze_code_basic() {
    let code = "
let a = 1;
let b = a + 1;
"
    .trim();
    let file_path = "test.rs".to_string();
    let result = analyze_code(code, file_path.clone(), file_path.clone()).unwrap();

    assert_snapshot!(serde_json::to_string_pretty(&result).unwrap());
}

#[test]
fn test_rust_function_call_dependency() {
    let code = "
fn add(a: i32, b: i32) -> i32 {
    a + b
}
fn main() {
    let x = add(1, 2);
}
"
    .trim();
    let file_path = "test.rs".to_string();
    let result = analyze_code(code, file_path.clone(), file_path.clone()).unwrap();

    assert_snapshot!(serde_json::to_string_pretty(&result).unwrap());
}

#[test]
fn test_rust_struct_field_access_dependency() {
    let code = r#"
struct Point { x: i32, y: i32 }
fn main() {
    let p = Point { x: 1, y: 2 };
    let val = p.x;
}
"#
    .trim();
    let file_path = "test.rs".to_string();
    let result = analyze_code(code, file_path.clone(), file_path.clone()).unwrap();

    assert_snapshot!(serde_json::to_string_pretty(&result).unwrap());
}

#[test]
fn test_rust_use_statements_dependency() {
    let code = r#"
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
"#
    .trim();
    let file_path = "test.rs".to_string();
    let result = analyze_code(code, file_path.clone(), file_path.clone()).unwrap();

    assert_snapshot!(serde_json::to_string_pretty(&result).unwrap());
}
