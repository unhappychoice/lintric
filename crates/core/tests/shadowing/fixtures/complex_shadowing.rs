// Complex shadowing test cases from Issue #103

fn main() {
    let x = 1;  // outer x
    {
        let x = 2;  // shadows outer x
        let y = x;  // should reference inner x (value: 2)
    }
    let z = x;  // should reference outer x (value: 1)
}

fn complex_shadowing() {
    let var = 10;           // level 1
    
    fn inner_func() {
        let var = 20;       // level 2 - shadows level 1
        
        {
            let var = 30;   // level 3 - shadows level 2
            println!("{}", var);  // should use level 3 (value: 30)
            
            {
                println!("{}", var);  // still level 3 (value: 30)
            }
        }
        
        println!("{}", var);  // back to level 2 (value: 20)
    }
    
    println!("{}", var);    // level 1 (value: 10)
    inner_func();
}

fn example(param: i32) {
    let param = param + 1;  // shadows function parameter
    
    {
        let param = param * 2;  // shadows local variable
        println!("{}", param);   // should use innermost param
    }
    
    println!("{}", param);  // should use local variable (not original parameter)
}

fn cross_scope_test() {
    let name = "outer";
    
    fn inner1() {
        let name = "inner1";
        println!("{}", name);  // inner1
    }
    
    fn inner2() {
        println!("{}", name);  // should use outer scope "outer"
    }
    
    {
        let name = "block";
        inner1();  // inner1's name doesn't affect this
        inner2();  // should still see outer "name"
        println!("{}", name);  // block
    }
    
    inner1();
    inner2();
    println!("{}", name);  // outer
}

// Four levels deep shadowing
fn deep_shadowing() {
    let deep_var = 1;  // level 1
    {
        let deep_var = 2;  // level 2
        {
            let deep_var = 3;  // level 3
            {
                let deep_var = 4;  // level 4
                {
                    let deep_var = 5;  // level 5
                    println!("{}", deep_var);  // should be 5
                }
                println!("{}", deep_var);  // should be 4
            }
            println!("{}", deep_var);  // should be 3
        }
        println!("{}", deep_var);  // should be 2
    }
    println!("{}", deep_var);  // should be 1
}

// Mutually recursive with shadowing
fn mutual_recursive_shadowing() {
    let recursive_var = "main";
    
    fn func_a(n: i32) {
        let recursive_var = "func_a";
        if n > 0 {
            func_b(n - 1);
        }
        println!("{}", recursive_var);
    }
    
    fn func_b(n: i32) {
        let recursive_var = "func_b";
        if n > 0 {
            func_a(n - 1);
        }
        println!("{}", recursive_var);
    }
    
    func_a(3);
    println!("{}", recursive_var);
}

// Generic type parameter shadowing
fn generic_shadowing<T>(param: T) -> T {
    let T = param;  // This shadows the type parameter T (this would be a compile error in real Rust, but for testing)
    T
}

// Loop shadowing patterns
fn loop_shadowing() {
    let i = 0;
    for i in 0..5 {  // shadows outer i
        let i = i * 2;  // shadows loop variable i
        println!("{}", i);
    }
    println!("{}", i);  // original i
    
    let j = 10;
    while let Some(j) = Some(j + 1) {  // shadows outer j
        if j > 15 { break; }
        let j = j * 3;  // shadows pattern j
        println!("{}", j);
    }
    println!("{}", j);  // original j
}