// Public trait to abstract logging for the CLI
// Methods accept &str to keep callsites simple.
pub trait Logger: Send + Sync {
    fn info(&self, message: &str);
    fn warn(&self, message: &str);
    fn error(&self, message: &str);
}

// Default logger using stdout/stderr
pub struct StdIoLogger;

impl Logger for StdIoLogger {
    fn info(&self, message: &str) {
        println!("{message}");
    }

    fn warn(&self, message: &str) {
        eprintln!("{message}");
    }

    fn error(&self, message: &str) {
        eprintln!("{message}");
    }
}
