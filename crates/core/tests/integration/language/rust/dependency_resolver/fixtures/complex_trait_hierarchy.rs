trait Animal {
    fn name(&self) -> &str;
    fn speak(&self) -> String;
}

trait Mammal: Animal {
    fn fur_color(&self) -> &str;
}

struct Dog {
    name: String,
    fur: String,
}

impl Animal for Dog {
    fn name(&self) -> &str {
        &self.name
    }
    
    fn speak(&self) -> String {
        format!("{} barks", self.name())
    }
}

impl Mammal for Dog {
    fn fur_color(&self) -> &str {
        &self.fur
    }
}

fn print_mammal_info<T: Mammal>(animal: &T) {
    println!("{} has {} fur and says: {}", 
        animal.name(), 
        animal.fur_color(), 
        animal.speak()
    );
}

fn main() {
    let dog = Dog {
        name: "Buddy".to_string(),
        fur: "brown".to_string(),
    };
    print_mammal_info(&dog);
}