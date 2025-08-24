fn main() {
    helper();
    
    let result = is_even(4);
    println!("{}", result);
}

fn helper() {
    println!("Helper function");
}

fn is_even(n: i32) -> bool {
    if n == 0 {
        true
    } else {
        is_odd(n - 1)
    }
}

fn is_odd(n: i32) -> bool {
    if n == 0 {
        false
    } else {
        is_even(n - 1)
    }
}