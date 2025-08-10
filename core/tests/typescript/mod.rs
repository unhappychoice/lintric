use insta::assert_snapshot;
use lintric_core::analyze_code;
use serde_json;

#[test]
fn test_analyze_code_typescript() {
    let code = "
const x = 1;
let y = x + 2;
function foo() {
    return y;
}
"
    .trim();
    let file_path = "test.ts".to_string();
    let result = analyze_code(code, file_path.clone(), file_path.clone()).unwrap();

    assert_snapshot!(serde_json::to_string_pretty(&result).unwrap());
}

#[test]
fn test_typescript_class_method_dependency() {
    let code = r#"
class MyClass {
    constructor(public value: number){}
    greet() { console.log(this.value); }
}
let instance = new MyClass(10);
instance.greet();
"#
    .trim();
    let file_path = "test.ts".to_string();
    let result = analyze_code(code, file_path.clone(), file_path.clone()).unwrap();

    assert_snapshot!(serde_json::to_string_pretty(&result).unwrap());
}

#[test]
fn test_typescript_import_dependency() {
    let code = r#"
import { someFunction } from './module';
const result = someFunction();
"#
    .trim();
    let file_path = "test.ts".to_string();
    let result = analyze_code(code, file_path.clone(), file_path.clone()).unwrap();

    assert_snapshot!(serde_json::to_string_pretty(&result).unwrap());
}
