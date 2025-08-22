fn main() {
    let y = x + 1;  // x not yet defined - should not create dependency
    let x = 42;
}

fn proper_order() {
    let x = 42;
    let y = x + 1;  // x already defined - should create dependency
}

fn block_scoping() {
    let outer = 10;
    {
        let inner = outer + 5;  // outer is accessible
    }
    // inner is not accessible here (but we don't test this case in dependency resolution)
}

fn helper() -> i32 {
    42
}

fn function_scoping() {
    let result = helper();  // helper defined before main
}

fn func_a() {
    let local_var = 10;
}

fn func_b() {
    // local_var not accessible from here, but since it's not used, no dependency should be created
    let x = 5;
}