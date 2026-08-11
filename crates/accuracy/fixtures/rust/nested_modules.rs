// Nested modules and paths reaching out of them.

const VERSION: i32 = 1;

mod outer {
    pub const SCALE: i32 = 2;

    pub mod inner {
        pub fn scaled() -> i32 {
            super::SCALE * crate::VERSION //~ depends: SCALE@6, VERSION@3
        }
    }

    pub fn double() -> i32 {
        inner::scaled() * 2 //~ depends: inner@8, scaled@9
    }
}

fn main() {
    let a = outer::double(); //~ depends: outer@5, double@14
    let b = outer::inner::scaled(); //~ depends: outer@5, inner@8, scaled@9
    let _ = a + b; //~ depends: a@20, b@21
}
