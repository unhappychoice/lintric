pub fn main() {
    let global_var = 10;
    
    fn outer_function() {
        let outer_var = 20;
        
        fn inner_function() {
            let inner_var = 30;
            println!("{}", outer_var); // Should resolve to outer scope
        }
        
        {
            let block_var = 40;
            println!("{}", global_var); // Should resolve to global scope
            inner_function(); // Function call
        }
    }
    
    mod inner_module {
        pub fn module_function() {
            let module_var = 50;
        }
        
        fn private_function() {
            let private_var = 60;
        }
    }
    
    outer_function();
}