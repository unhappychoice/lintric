struct Container<T> {
    item: T,
}

impl<T> Container<T> {
    fn new(item: T) -> Self {
        Container { item }
    }
    
    fn get(&self) -> &T {
        &self.item
    }
}

fn process<T: Clone>(container: &Container<T>) -> T {
    container.get().clone()
}

fn main() {
    let container = Container::new(42);
    let value = process(&container);
    println!("{}", value);
}