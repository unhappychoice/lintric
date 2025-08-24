enum Message {
    Text(String),
    Number(i32),
    Quit,
}

struct Handler {
    name: String,
}

impl Handler {
    fn new(name: String) -> Self {
        Handler { name }
    }
    
    fn handle(&self, msg: Message) {
        match msg {
            Message::Text(text) => {
                println!("{}: {}", self.name, text);
            }
            Message::Number(num) => {
                println!("{}: Number {}", self.name, num);
            }
            Message::Quit => {
                println!("{}: Quitting", self.name);
            }
        }
    }
}

fn main() {
    let handler = Handler::new("Main".to_string());
    
    handler.handle(Message::Text("Hello".to_string()));
    handler.handle(Message::Number(42));
    handler.handle(Message::Quit);
}