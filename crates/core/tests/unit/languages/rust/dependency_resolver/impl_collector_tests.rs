use lintric_core::languages::rust::dependency_resolver::impl_collector::RustImplCollector;
use tree_sitter::Parser;

fn setup_rust_parser() -> Parser {
    let mut parser = Parser::new();
    parser
        .set_language(&tree_sitter_rust::language())
        .expect("Error loading Rust grammar");
    parser
}

#[test]
fn test_impl_collector_creation() {
    let _collector = RustImplCollector::new();
    // Basic test to ensure collector can be created
}

#[test]
fn test_basic_impl_block_collection() {
    let source_code = r#"
struct MyStruct {
    value: i32,
}

impl MyStruct {
    fn new(value: i32) -> Self {
        Self { value }
    }
    
    fn get_value(&self) -> i32 {
        self.value
    }
    
    fn set_value(&mut self, new_value: i32) {
        self.value = new_value;
    }
}
    "#;

    let mut collector = RustImplCollector::new();
    let mut parser = setup_rust_parser();
    let tree = parser.parse(source_code, None).unwrap();

    let impl_blocks = collector
        .collect_impl_blocks(source_code, tree.root_node())
        .unwrap();

    assert!(!impl_blocks.is_empty(), "Should find impl blocks");
    assert_eq!(impl_blocks.len(), 1, "Should find exactly one impl block");

    let impl_block = &impl_blocks[0];
    assert_eq!(impl_block.methods.len(), 3, "Should find 3 methods");
}

#[test]
fn test_trait_definition_collection() {
    let source_code = r#"
trait Display {
    fn display(&self) -> String;
    fn print(&self) {
        println!("{}", self.display());
    }
}

trait Debug {
    fn debug(&self) -> String;
}

trait Clone {
    fn clone(&self) -> Self;
}
    "#;

    let mut collector = RustImplCollector::new();
    let mut parser = setup_rust_parser();
    let tree = parser.parse(source_code, None).unwrap();

    let traits = collector
        .collect_traits(source_code, tree.root_node())
        .unwrap();

    assert!(!traits.is_empty(), "Should find trait definitions");
    assert_eq!(traits.len(), 3, "Should find exactly 3 traits");

    let trait_names: Vec<_> = traits.iter().map(|t| &t.name).collect();
    assert!(
        trait_names.contains(&&"Display".to_string()),
        "Should find Display trait"
    );
    assert!(
        trait_names.contains(&&"Debug".to_string()),
        "Should find Debug trait"
    );
    assert!(
        trait_names.contains(&&"Clone".to_string()),
        "Should find Clone trait"
    );
}

#[test]
fn test_trait_impl_collection() {
    let source_code = r#"
struct User {
    name: String,
    age: u32,
}

trait Display {
    fn display(&self) -> String;
}

impl Display for User {
    fn display(&self) -> String {
        format!("{} ({})", self.name, self.age)
    }
}

impl Clone for User {
    fn clone(&self) -> Self {
        Self {
            name: self.name.clone(),
            age: self.age,
        }
    }
}
    "#;

    let mut collector = RustImplCollector::new();
    let mut parser = setup_rust_parser();
    let tree = parser.parse(source_code, None).unwrap();

    let trait_impls = collector
        .collect_trait_impl_blocks(source_code, tree.root_node())
        .unwrap();

    assert!(!trait_impls.is_empty(), "Should find trait implementations");
    assert_eq!(
        trait_impls.len(),
        2,
        "Should find exactly 2 trait implementations"
    );
}

#[test]
fn test_associated_function_vs_method() {
    let source_code = r#"
struct Point {
    x: f64,
    y: f64,
}

impl Point {
    // Associated function (no self)
    fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }
    
    // Associated function (no self)
    fn origin() -> Self {
        Self { x: 0.0, y: 0.0 }
    }
    
    // Method (has &self)
    fn distance(&self, other: &Point) -> f64 {
        ((self.x - other.x).powi(2) + (self.y - other.y).powi(2)).sqrt()
    }
    
    // Method (has &mut self)
    fn move_by(&mut self, dx: f64, dy: f64) {
        self.x += dx;
        self.y += dy;
    }
}
    "#;

    let mut collector = RustImplCollector::new();
    let mut parser = setup_rust_parser();
    let tree = parser.parse(source_code, None).unwrap();

    let impl_blocks = collector
        .collect_impl_blocks(source_code, tree.root_node())
        .unwrap();

    assert_eq!(impl_blocks.len(), 1, "Should find exactly one impl block");
    assert_eq!(
        impl_blocks[0].methods.len(),
        4,
        "Should find 4 methods/functions"
    );
}

#[test]
fn test_multiple_impl_blocks_same_type() {
    let source_code = r#"
struct Vector {
    data: Vec<i32>,
}

// First impl block
impl Vector {
    fn new() -> Self {
        Self { data: Vec::new() }
    }
    
    fn push(&mut self, value: i32) {
        self.data.push(value);
    }
}

// Second impl block for the same type
impl Vector {
    fn len(&self) -> usize {
        self.data.len()
    }
    
    fn is_empty(&self) -> bool {
        self.data.is_empty()
    }
}
    "#;

    let mut collector = RustImplCollector::new();
    let mut parser = setup_rust_parser();
    let tree = parser.parse(source_code, None).unwrap();

    let impl_blocks = collector
        .collect_impl_blocks(source_code, tree.root_node())
        .unwrap();

    assert_eq!(impl_blocks.len(), 2, "Should find exactly 2 impl blocks");
    assert_eq!(
        impl_blocks[0].methods.len(),
        2,
        "First impl block should have 2 methods"
    );
    assert_eq!(
        impl_blocks[1].methods.len(),
        2,
        "Second impl block should have 2 methods"
    );
}

#[test]
fn test_complex_trait_hierarchy() {
    let source_code = r#"
struct Rectangle {
    width: f64,
    height: f64,
}

trait Shape {
    fn area(&self) -> f64;
    fn perimeter(&self) -> f64;
}

trait Drawable {
    fn draw(&self);
}

trait Printable: Drawable {
    fn print_info(&self);
}

impl Shape for Rectangle {
    fn area(&self) -> f64 {
        self.width * self.height
    }
    
    fn perimeter(&self) -> f64 {
        2.0 * (self.width + self.height)
    }
}

impl Drawable for Rectangle {
    fn draw(&self) {
        println!("Drawing rectangle {}x{}", self.width, self.height);
    }
}

impl Printable for Rectangle {
    fn print_info(&self) {
        println!("Rectangle: area={}, perimeter={}", self.area(), self.perimeter());
    }
}
    "#;

    let mut collector = RustImplCollector::new();
    let mut parser = setup_rust_parser();
    let tree = parser.parse(source_code, None).unwrap();

    let traits = collector
        .collect_traits(source_code, tree.root_node())
        .unwrap();
    let trait_impls = collector
        .collect_trait_impl_blocks(source_code, tree.root_node())
        .unwrap();

    assert_eq!(traits.len(), 3, "Should find 3 trait definitions");
    assert_eq!(trait_impls.len(), 3, "Should find 3 trait implementations");
}
