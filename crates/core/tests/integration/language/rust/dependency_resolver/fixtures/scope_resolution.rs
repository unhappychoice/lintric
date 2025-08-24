fn outer() {
    let outer_var = 42;
    
    fn inner() {
        println!("{}", outer_var);
    }
    
    {
        let block_var = 20;
        println!("{} {}", outer_var, block_var);
    }
    
    inner();
}

fn main() {
    let x = 10;
    
    {
        let y = 20;
        println!("{} {}", x, y);
    }
    
    println!("{}", x);
}