macro_rules! my_macro {
    ($e:expr) => {
        println!("{}", $e);
    };
}

fn main() {
    my_macro!("Hello, custom macro!");
}