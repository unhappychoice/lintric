macro_rules! create_function {
    ($name:ident, $return_type:ty) => {
        fn $name() -> $return_type {
            Default::default()
        }
    };
}

macro_rules! impl_display {
    ($type:ty) => {
        impl std::fmt::Display for $type {
            fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                write!(f, "Custom Display")
            }
        }
    };
}

create_function!(get_number, i32);
create_function!(get_string, String);

struct CustomType;

impl_display!(CustomType);

fn main() {
    let num = get_number();
    let s = get_string();
    let custom = CustomType;
    
    println!("{} {} {}", num, s, custom);
}