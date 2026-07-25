// Match arms, if let and pattern bindings.

enum Value {
    Number(i32),
    Text,
}

fn describe(value: Value) -> i32 { //~ depends: Value@3
    match value { //~ depends: value@8
        Value::Number(n) => n, //~ depends: Value@3, Number@4
        Value::Text => 0, //~ depends: Value@3, Text@5
    }
}

fn main() {
    let v = Value::Number(1); //~ depends: Value@3, Number@4
    let n = describe(v); //~ depends: describe@8, v@16
    if let Value::Text = v { //~ depends: Value@3, Text@5, v@16
        let _ = n; //~ depends: n@17
    }
}
