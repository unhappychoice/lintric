struct Calculator {
    value: i32,
}

impl Calculator {
    fn new() -> Self {
        Calculator { value: 0 }
    }
    
    fn add(&mut self, n: i32) {
        self.value += n;
    }
    
    fn get_value(&self) -> i32 {
        self.value
    }
}

trait Display {
    fn display(&self) -> String;
}

impl Display for Calculator {
    fn display(&self) -> String {
        format!("Calculator({})", self.value)
    }
}

fn main() {
    let mut calc = Calculator::new();
    calc.add(5);
    println!("{}", calc.get_value());
    println!("{}", calc.display());
}