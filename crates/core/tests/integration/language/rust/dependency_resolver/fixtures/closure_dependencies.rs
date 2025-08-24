fn main() {
    let captured = 42;
    let mut mutable_capture = 10;
    
    let closure = || {
        println!("{}", captured);
        mutable_capture += 1;
        println!("{}", mutable_capture);
    };
    
    closure();
    
    let move_closure = move || {
        println!("Moved: {}", captured);
    };
    
    move_closure();
}