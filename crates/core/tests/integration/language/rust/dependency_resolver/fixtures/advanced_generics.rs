trait Clone {
    fn clone(&self) -> Self;
}

trait Display {
    fn display(&self) -> String;
}

fn process_item<T: Clone + Display>(item: &T) -> String {
    let cloned = item.clone();
    cloned.display()
}

struct Item {
    value: i32,
}

impl Clone for Item {
    fn clone(&self) -> Self {
        Item { value: self.value }
    }
}

impl Display for Item {
    fn display(&self) -> String {
        format!("Item({})", self.value)
    }
}

fn main() {
    let item = Item { value: 42 };
    let result = process_item(&item);
    println!("{}", result);
}